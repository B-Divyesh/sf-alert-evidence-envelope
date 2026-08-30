use alert_evidence_envelope::{
    api_router, create_state, health, load_or_generate_signing_key, load_or_generate_token,
};
use anyhow::Context;
use axum::{
    extract::Request,
    http::{header, HeaderName, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Json, Router,
};
use serde_json::json;
use std::{env, net::SocketAddr, path::PathBuf};
use tokio::signal;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorError,
    GovernorLayer,
};
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

    let data_dir = env::var("DATA_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        if std::path::Path::new("/data").is_dir() {
            PathBuf::from("/data")
        } else {
            PathBuf::from("data")
        }
    });
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        format!(
            "sqlite:{}?mode=rwc",
            data_dir.join("envelopes.db").display()
        )
    });
    if let Ok(database_path) = sqlite_file_path(&database_url) {
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let legacy_snapshot = data_dir.join("envelopes.snapshot.db");
        if restore_legacy_snapshot_once(&database_path, &legacy_snapshot)? {
            info!("migrated legacy SQLite snapshot to the durable database");
        }
    }
    info!(
        database_source = if env::var_os("DATABASE_URL").is_some() {
            "supplied"
        } else {
            "generated default"
        },
        data_dir = %data_dir.display(),
        "runtime database configured"
    );
    let signing_key_path = env::var("SIGNING_KEY_FILE")
        .unwrap_or_else(|_| data_dir.join("envelope-signing.key").display().to_string());
    let supplied_signing_key = env::var("ENVELOPE_SIGNING_KEY").ok();
    let (signing_key, signing_key_source) = load_or_generate_signing_key(
        std::path::Path::new(&signing_key_path),
        supplied_signing_key.as_deref(),
    )?;
    info!(
        signing_key_source = signing_key_source.label(),
        "runtime signing key configured"
    );
    let admin_token_path = env::var("ADMIN_TOKEN_FILE")
        .unwrap_or_else(|_| data_dir.join("admin.token").display().to_string());
    let (admin_token, admin_token_source) = load_or_generate_token(
        std::path::Path::new(&admin_token_path),
        env::var("ADMIN_TOKEN").ok().as_deref(),
    )?;
    info!(
        admin_token_source = admin_token_source.label(),
        admin_token_path, "admin access configured"
    );
    let inbound_token_path = env::var("INBOUND_TOKEN_FILE")
        .unwrap_or_else(|_| data_dir.join("inbound.token").display().to_string());
    let (inbound_token, inbound_token_source) = load_or_generate_token(
        std::path::Path::new(&inbound_token_path),
        env::var("INBOUND_TOKEN").ok().as_deref(),
    )?;
    info!(
        inbound_token_source = inbound_token_source.label(),
        inbound_token_path, "inbound relay access configured"
    );
    let state = create_state(&database_url, signing_key, admin_token, inbound_token).await?;
    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "dist".into());
    let index = format!("{static_dir}/index.html");
    let governor = GovernorConfigBuilder::default()
        .per_millisecond(50)
        .burst_size(40)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .expect("valid rate limit configuration");
    let app = Router::new()
        .route("/health", get(health))
        .merge(api_router(state).layer(GovernorLayer::new(governor).error_handler(
            |error: GovernorError| {
                let message = match error {
                    GovernorError::TooManyRequests { .. } => {
                        "Request limit reached. Retry in one second."
                    }
                    _ => "The request could not be rate limited.",
                };
                let mut response =
                    (StatusCode::TOO_MANY_REQUESTS, Json(json!({ "error": message })))
                        .into_response();
                response.headers_mut().insert(
                    header::RETRY_AFTER,
                    HeaderValue::from_static("1"),
                );
                response
            },
        )))
        // Legal pages must also work for direct links, crawlers, and clients
        // without JavaScript; serve their complete static documents explicitly.
        .route_service(
            "/privacy",
            get_service(ServeFile::new(format!("{static_dir}/privacy.html"))),
        )
        .route_service(
            "/terms",
            get_service(ServeFile::new(format!("{static_dir}/terms.html"))),
        )
        .route_service("/demo", get_service(ServeFile::new(index)))
        .fallback_service(
            ServeDir::new(&static_dir).not_found_service(axum::routing::get(not_found)),
        )
        .layer(middleware::from_fn(cache_policy))
        .layer(SetResponseHeaderLayer::if_not_present(HeaderName::from_static("x-content-type-options"), HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::if_not_present(HeaderName::from_static("x-frame-options"), HeaderValue::from_static("DENY")))
        .layer(SetResponseHeaderLayer::if_not_present(HeaderName::from_static("referrer-policy"), HeaderValue::from_static("no-referrer")))
        .layer(SetResponseHeaderLayer::if_not_present(HeaderName::from_static("strict-transport-security"), HeaderValue::from_static("max-age=63072000; includeSubDomains")))
        .layer(SetResponseHeaderLayer::if_not_present(HeaderName::from_static("content-security-policy"), HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in; frame-ancestors 'none'")));
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

fn sqlite_file_path(database_url: &str) -> anyhow::Result<PathBuf> {
    let path = database_url
        .strip_prefix("sqlite:")
        .and_then(|value| value.split('?').next())
        .filter(|value| !value.is_empty() && *value != ":memory:")
        .context("DATABASE_URL must use a file-backed SQLite path")?;
    Ok(PathBuf::from(path))
}

fn restore_legacy_snapshot_once(
    database: &std::path::Path,
    legacy: &std::path::Path,
) -> anyhow::Result<bool> {
    if database.exists() || !legacy.exists() || database == legacy {
        return Ok(false);
    }
    std::fs::copy(legacy, database).with_context(|| {
        format!(
            "migrate legacy SQLite snapshot {} to {}",
            legacy.display(),
            database.display()
        )
    })?;
    Ok(true)
}

async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Html(include_str!("../frontend/static/404.html")),
    )
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

#[cfg(test)]
mod tests {
    use super::restore_legacy_snapshot_once;

    #[test]
    fn legacy_snapshot_is_migrated_only_when_the_database_is_absent() {
        let directory = tempfile::tempdir().unwrap();
        let legacy = directory.path().join("envelopes.snapshot.db");
        let database = directory.path().join("envelopes.db");
        std::fs::write(&legacy, b"legacy database").unwrap();

        assert!(restore_legacy_snapshot_once(&database, &legacy).unwrap());
        assert_eq!(std::fs::read(&database).unwrap(), b"legacy database");
        std::fs::write(&legacy, b"new snapshot").unwrap();
        assert!(!restore_legacy_snapshot_once(&database, &legacy).unwrap());
        assert_eq!(std::fs::read(&database).unwrap(), b"legacy database");
    }
}
