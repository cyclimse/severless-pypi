use std::time::Duration;

use aws_credential_types::Credentials;
use aws_smithy_client::http_connector::ConnectorSettings;
use aws_smithy_client::hyper_ext;
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
    pub s3_client: aws_sdk_s3::Client,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .build();
        let http_client = Client::builder().build::<_, hyper::Body>(https.clone());

        let smithy_connector = hyper_ext::Adapter::builder()
            // Optionally set things like timeouts as well
            .connector_settings(
                ConnectorSettings::builder()
                    .connect_timeout(Duration::from_secs(5))
                    .build(),
            )
            .build(https);

        let creds = Credentials::from_keys(
            config.scw_access_key.clone(),
            config.scw_secret_key.clone(),
            None,
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .http_connector(smithy_connector)
            .credentials_provider(creds)
            .region(aws_sdk_s3::Region::new(
                config.scw_default_region.to_string(),
            ))
            .endpoint_url(config.s3_endpoint.clone())
            .force_path_style(true)
            .build();
        let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

        Self {
            config,
            http_client,
            s3_client,
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
