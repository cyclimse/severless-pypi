use crate::pep_691::{self, IndexProject};
use crate::states::AppState;
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
) -> Result<Response<Body>, StatusCode> {
    forward_to_index(&mut req, &state.config.pypi_index);
    state
        .http_client
        .request(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn pypi_package(
    State(state): State<AppState>,
    Path(path): Path<String>,
    mut req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    forward_to_index(&mut req, &state.config.pypi_index);
    let resp = state
        .http_client
        .request(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !resp.status().is_success() {
        info!("Received unsuppported call to API");
        return Ok(resp);
    }

    let content_type = resp
        .headers()
        .get(http::header::CONTENT_TYPE)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if content_type
        != pep_691::SUPPORTED_CONTENT_TYPE
            .parse::<HeaderValue>()
            .unwrap()
    {
        info!("ee unsuppported call to API");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let (parts, body) = resp.into_parts();
    let body = hyper::body::aggregate(body).await.unwrap();
    let mut d = GzDecoder::new(body.reader());

    let mut s = String::new();
    d.read_to_string(&mut s).unwrap();

    let mut project: pep_691::IndexProject = match serde_json::from_str(&s) {
        Ok(project) => project,
        Err(e) => {
            error!(target: "app_events", "App Error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    redirect_to_reserved(&mut project);

    let body = serde_json::to_vec(&project).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Response::builder()
        .header(http::header::CONTENT_TYPE, pep_691::SUPPORTED_CONTENT_TYPE)
        .body(Body::from(body))
        .unwrap())
}

pub fn redirect_to_reserved(project: &mut IndexProject) {
    let mut versions = std::collections::HashMap::new();

    for file in project.files.clone() {
        if file.filename.ends_with(".tar.gz") {
            let version = file
                .filename
                .split("-")
                .nth(1)
                .unwrap()
                .strip_suffix(".tar.gz")
                .unwrap()
                .to_string();
            versions.insert(version, file.url.clone());
        }
    }

    println!("{:?}", versions);

    for file in &mut project.files {
        if file.filename.ends_with(".whl") {
            let version = file.filename.split("-").nth(1).unwrap();
            let url = Uri::try_from(file.url.clone()).unwrap();
            let path: Vec<&str> = url.path().split('/').collect();

            let archive = versions.get(version).unwrap();

            file.url = format!("download/{}?archive={}", path.last().unwrap(), archive);
        }
    }
}

#[derive(Deserialize)]
pub struct Archive {
    archive: String,
}

#[derive(Serialize)]
pub struct WorkerRequest {
    project: String,
    archive_url: String,
}

pub async fn download_package(
    State(state): State<AppState>,
    archive: Query<Archive>,
    Path((package, filename)): Path<(String, String)>,
) -> Result<Response<Body>, StatusCode> {
    println!("{}", package);
    println!("{}", filename);

    let worker_request = WorkerRequest {
        project: package,
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

    println!("{:?}", req);

    state
        .http_client
        .request(req)
        .await
        .map_err(|_| StatusCode::NOT_IMPLEMENTED)?;

    todo!()
}
