mod config;
mod index;
mod pep_691;
mod states;

use axum::{routing::get, Router};
use config::Config;
use index::{download_package, pypi_index, pypi_package};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = envy::from_env::<Config>().unwrap();
    let port = config.port.unwrap_or(4000);

    let state = states::AppState::new(config);

    let app = Router::new()
        .route("/", get(pypi_index))
        .route("/:package/download/*filename", get(download_package))
        .route("/:package/", get(pypi_package))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::Server::bind(&format!("0.0.0.0:{}", port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
