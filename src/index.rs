use crate::controller;
use crate::errors::{self, ServiceError};
use crate::pep_691;
use crate::states::AppState;
use aws_sdk_s3::error::{GetObjectError, GetObjectErrorKind};
use axum::extract::Query;
use flate2::read::GzDecoder;

use axum::{
    extract::{Path, State},
    http::{self, uri::Uri, HeaderValue, Request, Response},
};
use hyper::{body::Buf, Body, StatusCode};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use tracing::{error, info};

pub fn forward_to_index(req: &mut Request<Body>, pypi_index: &str) {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("https://{}/simple{}", pypi_index, path_query);
    req.headers_mut()
        .insert(http::header::HOST, pypi_index.parse().unwrap());

    *req.uri_mut() = Uri::try_from(uri).unwrap();
}

pub async fn pypi_index(
    State(state): State<AppState>,
    mut req: Request<Body>,
) -> Result<Response<Body>, errors::ServiceError> {
    forward_to_index(&mut req, &state.config.pypi_index);
    let resp = state.http_client.request(req).await?;
    Ok(resp)
}

pub async fn pypi_package(
    State(state): State<AppState>,
    Path(path): Path<String>, // Path to the package eg. /numpy
    mut req: Request<Body>,
) -> Result<Response<Body>, errors::ServiceError> {
    forward_to_index(&mut req, &state.config.pypi_index);
    let resp = state.http_client.request(req).await?;

    if !resp.status().is_success() {
        let index = &state.config.pypi_index;
        let status = resp.status().to_string();
        error!("Received status {status} from PyPI index {index} for package {path}");
        if resp.status() == StatusCode::NOT_FOUND {
            return Err(ServiceError::PackageNotFound { package: path });
        }
        return Err(resp.status().into());
    }

    let content_type = resp
        .headers()
        .get(http::header::CONTENT_TYPE)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if content_type != HeaderValue::from_static(pep_691::SUPPORTED_CONTENT_TYPE) {
        error!(
            "Received unsupported request with Content-Type {}",
            content_type.to_str().unwrap_or_default()
        );
        return Err(ServiceError::GenericError {
            status: Some(StatusCode::BAD_REQUEST),
            message: Some("This index only supports pep_691 client requests. TIP: are you using the latest pip version?".to_owned()),
        });
    }

    let (parts, body) = resp.into_parts();
    let body = hyper::body::aggregate(body).await.unwrap();
    let mut s = String::new();

    if let Some(encoding) = parts.headers.get(http::header::CONTENT_ENCODING) {
        if encoding == HeaderValue::from_static("gzip") {
            let mut d = GzDecoder::new(body.reader());
            d.read_to_string(&mut s).unwrap();
        } else {
            error!(
                "Unsupported PyPI response encoding {}",
                encoding.to_str().unwrap_or_default()
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into());
        }
    } else {
        body.reader().read_to_string(&mut s).unwrap();
    }

    let mut project: pep_691::IndexProject = match serde_json::from_str(&s) {
        Ok(project) => project,
        Err(e) => {
            error!("Could not serialize PyPI registry response: {e}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    };

    controller::redirect_to_reserved(&mut project);

    let body = serde_json::to_vec(&project).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Response::builder()
        .header(http::header::CONTENT_TYPE, pep_691::SUPPORTED_CONTENT_TYPE)
        .body(Body::from(body))
        .unwrap())
}

#[derive(Deserialize)]
pub struct Archive {
    archive: String,
}

#[derive(Serialize)]
pub struct WorkerRequest {
    project: String,
    filename: String,
    archive_url: String,
}

pub async fn download_package(
    State(state): State<AppState>,
    archive: Query<Archive>,
    Path((package, filename)): Path<(String, String)>,
) -> Result<Response<Body>, ServiceError> {
    info!(package = package, filename = filename);

    // Check if package is present in S3
    let key = format!("{}/{}", package, filename);
    match state
        .s3_client
        .get_object()
        .bucket(&state.config.s3_bucket)
        .key(&key)
        .send()
        .await
    {
        Ok(res) => {
            info!("Found file {filename} for {package} in s3 bucket");
            let bytes = res.body.collect().await.unwrap();
            let resp = Response::builder()
                .status(http::StatusCode::OK)
                .header(http::header::CONTENT_TYPE, "application/zip")
                .body(Body::from(bytes.to_vec()))
                .unwrap();
            return Ok(resp);
        }
        Err(sdk_err) => match sdk_err.into_service_error() {
            GetObjectError {
                kind: GetObjectErrorKind::NoSuchKey(_),
                ..
            } => {}
            err => {
                let bucket = &state.config.s3_bucket;
                error!("Got {:?} while calling get object in {} on key {}", err, bucket, key);
                return Err(ServiceError::GenericError {
                    status: Some(StatusCode::INTERNAL_SERVER_ERROR),
                    message: Some(format!("Error while looking for file {filename}")),
                });
            }
        },
    }

    let worker_request = WorkerRequest {
        project: package.clone(),
        filename: filename.clone(),
        archive_url: archive.archive.to_string(),
    };

    let body =
        serde_json::to_vec(&worker_request).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let req = Request::builder()
        .method(http::method::Method::POST)
        .uri(state.config.worker_url)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap();

    state
        .http_client
        .request(req)
        .await
        .map_err(|_| StatusCode::NOT_IMPLEMENTED)?;

    let stream = state
        .s3_client
        .get_object()
        .bucket(state.config.s3_bucket)
        .key(format!("{}/{}", package, filename))
        .send()
        .await
        .unwrap()
        .body
        .collect()
        .await
        .unwrap();

    Ok(Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "application/zip")
        .body(Body::from(stream.to_vec()))
        .unwrap())
}
