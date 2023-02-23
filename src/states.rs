use axum::extract::FromRef;
use hyper::{client::HttpConnector, Body, Client};
use hyper_rustls::HttpsConnector;

use crate::config::Config;

pub type TlsClient = hyper::client::Client<HttpsConnector<HttpConnector>, Body>;

// Reference: https://docs.rs/axum/latest/axum/extract/struct.State.html#substates
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http_client: TlsClient,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .build();
        let http_client = Client::builder().build::<_, hyper::Body>(https);

        Self {
            config,
            http_client,
        }
    }
}

impl FromRef<AppState> for Config {
    fn from_ref(app_state: &AppState) -> Config {
        app_state.config.clone()
    }
}

impl FromRef<AppState> for TlsClient {
    fn from_ref(app_state: &AppState) -> TlsClient {
        app_state.http_client.clone()
    }
}
