use alert_evidence_envelope::{api_router, create_state};
use anyhow::Context;
use axum::{
    extract::Request,
    http::{header, HeaderName, HeaderValue},
    middleware::{self, Next},
    response::Response,
    Router,
};
use std::{env, net::SocketAddr};
use tokio::signal;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "alert_evidence_envelope=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/envelopes.db?mode=rwc".into());
    if database_url.starts_with("sqlite:data/") {
        std::fs::create_dir_all("data")?;
    }
    if env::var("ENVELOPE_SIGNING_KEY").is_err() {
        tracing::warn!("ENVELOPE_SIGNING_KEY is unset; using an insecure development key");
    }
    let state = create_state(&database_url).await?;
    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "dist".into());
    let index = format!("{static_dir}/index.html");
    let governor = GovernorConfigBuilder::default()
        .per_millisecond(5)
        .burst_size(300)
        .finish()
        .expect("valid rate limit configuration");
    let app = Router::new()
        .merge(api_router(state))
        .fallback_service(ServeDir::new(&static_dir).not_found_service(ServeFile::new(index)))
        .layer(GovernorLayer::new(governor))
        .layer(middleware::from_fn(cache_policy))
        .layer(SetResponseHeaderLayer::if_not_present(HeaderName::from_static("x-content-type-options"), HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::if_not_present(HeaderName::from_static("referrer-policy"), HeaderValue::from_static("no-referrer")))
        .layer(SetResponseHeaderLayer::if_not_present(HeaderName::from_static("content-security-policy"), HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in")));
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .context("PORT must be a valid TCP port")?;
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(address).await?;
    info!(%address, "alert evidence envelope listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    Ok(())
}

async fn cache_policy(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    let value =
        if path.starts_with("/assets/") || path.starts_with("/fonts/") || path == "/favicon.svg" {
            "public, max-age=31536000, immutable"
        } else if path.starts_with("/api/") || path == "/health" {
            "no-store"
        } else {
            "no-cache"
        };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    response
}

async fn shutdown() {
    let ctrl_c = async { signal::ctrl_c().await.expect("install Ctrl+C handler") };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
