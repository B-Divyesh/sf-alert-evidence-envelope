use alert_evidence_envelope::{api_router, create_state};
use anyhow::Context;
use axum::Router;
use std::{env, net::SocketAddr};
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "alert_evidence_envelope=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/envelopes.db?mode=rwc".into());
    if database_url.starts_with("sqlite:data/") { std::fs::create_dir_all("data")?; }
    if env::var("ENVELOPE_SIGNING_KEY").is_err() {
        tracing::warn!("ENVELOPE_SIGNING_KEY is unset; using an insecure development key");
    }
    let state = create_state(&database_url).await?;
    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "dist".into());
    let index = format!("{static_dir}/index.html");
    let app = Router::new()
        .merge(api_router(state))
        .fallback_service(ServeDir::new(&static_dir).not_found_service(ServeFile::new(index)));
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "8080".into()).parse().context("PORT must be a valid TCP port")?;
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(address).await?;
    info!(%address, "alert evidence envelope listening");
    axum::serve(listener, app).with_graceful_shutdown(shutdown()).await?;
    Ok(())
}

async fn shutdown() {
    let ctrl_c = async { signal::ctrl_c().await.expect("install Ctrl+C handler") };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate()).expect("install SIGTERM handler").recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
