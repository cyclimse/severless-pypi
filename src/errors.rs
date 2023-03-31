use axum::response::{IntoResponse, Response};
use hyper::StatusCode;

pub enum ServiceError {
    GenericError {
        status: Option<hyper::StatusCode>,
        message: Option<String>,
    },
    HyperError {
        error: hyper::Error,
    },
    PackageNotFound {
        package: String,
    },
}

impl From<hyper::Error> for ServiceError {
    fn from(error: hyper::Error) -> Self {
        ServiceError::HyperError { error }
    }
}

impl From<hyper::StatusCode> for ServiceError {
    fn from(status: hyper::StatusCode) -> Self {
        ServiceError::GenericError {
            status: Some(status),
            message: status.canonical_reason().map(str::to_owned),
        }
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        match self {
            ServiceError::GenericError { status, message } => (
                status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                message.unwrap_or_default(),
            )
                .into_response(),
            ServiceError::HyperError { error } => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
            ServiceError::PackageNotFound { package } => {
                let message = format!("Package {package} not found!");
                (StatusCode::NOT_FOUND, message).into_response()
            }
        }
    }
}
