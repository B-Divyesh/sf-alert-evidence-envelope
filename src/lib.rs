use anyhow::Context;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::Write,
    path::{Path as FsPath, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use subtle::ConstantTimeEq;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tower_http::{
    compression::CompressionLayer, limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer, trace::TraceLayer,
};
use url::Url;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub client: Client,
    pub signing_key: Arc<Vec<u8>>,
    pub admin_token: Arc<String>,
    pub inbound_token: Arc<String>,
    pub upstream_token: Option<Arc<String>>,
    pub destination_token: Option<Arc<String>>,
    pub destination_url_override: Option<Arc<String>>,
    demo_sessions: Arc<tokio::sync::RwLock<HashMap<String, chrono::DateTime<Utc>>>>,
    snapshot: Option<Arc<SnapshotPaths>>,
    persistence_lock: Arc<tokio::sync::Mutex<()>>,
}

#[derive(Debug)]
struct SnapshotPaths {
    database: PathBuf,
    durable: PathBuf,
}

/// Loads an operator token or generates a persistent 256-bit hexadecimal token.
/// The value is written with mode 600 and is never returned by an API or log.
pub fn load_or_generate_token(
    token_path: &FsPath,
    supplied: Option<&str>,
) -> anyhow::Result<(String, SigningKeySource)> {
    if let Some(token) = supplied {
        anyhow::ensure!(
            token.len() >= 32,
            "access tokens must contain at least 32 characters"
        );
        return Ok((token.to_owned(), SigningKeySource::Supplied));
    }

    match std::fs::read_to_string(token_path) {
        Ok(token) => {
            let token = token.trim().to_owned();
            anyhow::ensure!(token.len() >= 64, "persisted access token is invalid");
            return Ok((token, SigningKeySource::Persisted));
        }
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            return Err(error)
                .with_context(|| format!("read access token at {}", token_path.display()));
        }
        Err(_) => {}
    }

    let parent = token_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("access token path must have a parent directory"))?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create access-token directory {}", parent.display()))?;
    let mut random = [0_u8; 32];
    getrandom::fill(&mut random).context("generate an access token")?;
    let token = hex::encode(random);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.open(token_path) {
        Ok(mut file) => {
            file.write_all(token.as_bytes())
                .with_context(|| format!("write access token at {}", token_path.display()))?;
            file.sync_all()
                .with_context(|| format!("sync access token at {}", token_path.display()))?;
            Ok((token, SigningKeySource::Generated))
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let persisted = std::fs::read_to_string(token_path).with_context(|| {
                format!(
                    "read concurrently-created access token at {}",
                    token_path.display()
                )
            })?;
            let persisted = persisted.trim().to_owned();
            anyhow::ensure!(persisted.len() >= 64, "persisted access token is invalid");
            Ok((persisted, SigningKeySource::Persisted))
        }
        Err(error) => {
            Err(error).with_context(|| format!("create access token at {}", token_path.display()))
        }
    }
}

/// Records whether the HMAC key was explicitly configured or safely recovered
/// from the local instance's persistent storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningKeySource {
    Supplied,
    Persisted,
    Generated,
}

impl SigningKeySource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Supplied => "supplied",
            Self::Persisted => "persisted",
            Self::Generated => "generated",
        }
    }
}

/// Loads an operator-provided key or creates a unique, persistent 256-bit
/// key for an instance. The generated file contains raw key bytes and is never
/// returned in logs or API responses.
pub fn load_or_generate_signing_key(
    key_path: &FsPath,
    supplied: Option<&str>,
) -> anyhow::Result<(Vec<u8>, SigningKeySource)> {
    if let Some(key) = supplied {
        validate_signing_key(key.as_bytes())?;
        return Ok((key.as_bytes().to_vec(), SigningKeySource::Supplied));
    }

    match std::fs::read(key_path) {
        Ok(key) => {
            validate_signing_key(&key)?;
            return Ok((key, SigningKeySource::Persisted));
        }
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            return Err(error)
                .with_context(|| format!("read signing key at {}", key_path.display()));
        }
        Err(_) => {}
    }

    let parent = key_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("signing key path must have a parent directory"))?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create signing-key directory {}", parent.display()))?;
    let mut key = vec![0_u8; 32];
    getrandom::fill(&mut key).context("generate a signing key")?;

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.open(key_path) {
        Ok(mut file) => {
            file.write_all(&key)
                .with_context(|| format!("write signing key at {}", key_path.display()))?;
            file.sync_all()
                .with_context(|| format!("sync signing key at {}", key_path.display()))?;
            Ok((key, SigningKeySource::Generated))
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let persisted = std::fs::read(key_path).with_context(|| {
                format!(
                    "read concurrently-created signing key at {}",
                    key_path.display()
                )
            })?;
            validate_signing_key(&persisted)?;
            Ok((persisted, SigningKeySource::Persisted))
        }
        Err(error) => {
            Err(error).with_context(|| format!("create signing key at {}", key_path.display()))
        }
    }
}

fn validate_signing_key(key: &[u8]) -> anyhow::Result<()> {
    anyhow::ensure!(
        key.len() >= 32,
        "ENVELOPE_SIGNING_KEY must contain at least 32 bytes"
    );
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub id: String,
    pub name: String,
    pub source_url: Option<String>,
    pub destination_url: Option<String>,
    pub destination_kind: String,
    pub query_pointer: String,
    pub evidence_pointer: String,
    pub service_pointer: String,
    pub error_pointer: String,
    pub time_pointer: String,
    pub redact_fields: Vec<String>,
    pub max_items: usize,
    pub max_bytes: usize,
    pub enabled: bool,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            id: "primary".into(),
            name: "Primary incident route".into(),
            source_url: None,
            destination_url: None,
            destination_kind: "json".into(),
            query_pointer: "/query".into(),
            evidence_pointer: "/evidence".into(),
            service_pointer: "/service".into(),
            error_pointer: "/error".into(),
            time_pointer: "/startsAt".into(),
            redact_fields: vec![
                "authorization".into(),
                "password".into(),
                "token".into(),
                "email".into(),
                "cookie".into(),
            ],
            max_items: 20,
            max_bytes: 32_768,
            enabled: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvelopeSummary {
    pub service: String,
    pub error_signature: String,
    pub first_seen: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvidenceEnvelope {
    pub schema: String,
    pub id: String,
    pub created_at: String,
    pub channel: String,
    pub summary: EnvelopeSummary,
    pub query_fingerprint: String,
    pub evidence: Vec<Value>,
    pub evidence_items: usize,
    pub evidence_bytes: usize,
    pub truncated: bool,
    pub redacted_fields: Vec<String>,
    pub source_signature_preserved: bool,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct RelayResult {
    pub status: String,
    pub delivery: String,
    pub envelope: EvidenceEnvelope,
}

#[derive(Debug, Deserialize, Clone)]
struct PreviewRequest {
    alert: Value,
    evidence: Option<Vec<Value>>,
    redact_fields: Option<Vec<String>>,
    max_items: Option<usize>,
    max_bytes: Option<usize>,
}

#[derive(Debug, Serialize)]
struct DemoSession {
    id: String,
    expires_at: String,
}

#[derive(Debug, Serialize)]
struct DeliveryRecord {
    id: String,
    service: String,
    status: String,
    fingerprint: String,
    created_at: String,
    evidence_items: i64,
    evidence_bytes: i64,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("channel was not found")]
    NotFound,
    #[error("upstream evidence could not be fetched: {0}")]
    Upstream(String),
    #[error("delivery failed: {0}")]
    Delivery(String),
    #[error("internal service error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
            Self::Delivery(_) => StatusCode::BAD_GATEWAY,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let public_message = self.to_string();
        (status, Json(json!({ "error": public_message }))).into_response()
    }
}

pub async fn create_state(
    database_url: &str,
    signing_key: Vec<u8>,
    admin_token: String,
    inbound_token: String,
) -> anyhow::Result<AppState> {
    create_state_with_snapshot(database_url, signing_key, admin_token, inbound_token, None).await
}

pub async fn create_state_with_snapshot(
    database_url: &str,
    signing_key: Vec<u8>,
    admin_token: String,
    inbound_token: String,
    snapshot: Option<(PathBuf, PathBuf)>,
) -> anyhow::Result<AppState> {
    let snapshot = snapshot.map(|(database, durable)| SnapshotPaths { database, durable });
    if let Some(paths) = &snapshot {
        restore_database_snapshot(paths).await?;
    }
    // Azure Files is an SMB filesystem. A single SQLite connection avoids
    // cross-connection contention. In production the live database remains
    // local and committed copies are written to the mounted durable share.
    let options = SqliteConnectOptions::from_str(database_url)?
        .journal_mode(SqliteJournalMode::Delete)
        .busy_timeout(Duration::from_secs(30));
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    migrate(&db).await?;
    seed_default(&db).await?;
    let state = AppState {
        db,
        client: Client::builder()
            .timeout(Duration::from_secs(8))
            .user_agent("alert-evidence-envelope/0.1")
            .build()?,
        signing_key: Arc::new(signing_key),
        admin_token: Arc::new(admin_token),
        inbound_token: Arc::new(inbound_token),
        upstream_token: std::env::var("UPSTREAM_BEARER_TOKEN").ok().map(Arc::new),
        destination_token: std::env::var("DESTINATION_BEARER_TOKEN").ok().map(Arc::new),
        destination_url_override: std::env::var("DESTINATION_URL").ok().map(Arc::new),
        demo_sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        snapshot: snapshot.map(Arc::new),
        persistence_lock: Arc::new(tokio::sync::Mutex::new(())),
    };
    persist_database_snapshot(&state).await?;
    Ok(state)
}

async fn restore_database_snapshot(paths: &SnapshotPaths) -> anyhow::Result<()> {
    if !paths.durable.exists() {
        return Ok(());
    }
    if let Some(parent) = paths.database.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    for suffix in ["-journal", "-wal", "-shm"] {
        let stale = PathBuf::from(format!("{}{}", paths.database.display(), suffix));
        let _ = tokio::fs::remove_file(stale).await;
    }
    let contents = tokio::fs::read(&paths.durable)
        .await
        .with_context(|| format!("read SQLite snapshot {}", paths.durable.display()))?;
    let mut database = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&paths.database)
        .await
        .with_context(|| format!("open restored database {}", paths.database.display()))?;
    database.write_all(&contents).await?;
    database.sync_all().await?;
    Ok(())
}

async fn persist_database_snapshot(state: &AppState) -> anyhow::Result<()> {
    let Some(paths) = &state.snapshot else {
        return Ok(());
    };
    let parent = paths
        .durable
        .parent()
        .ok_or_else(|| anyhow::anyhow!("snapshot path must have a parent directory"))?;
    tokio::fs::create_dir_all(parent).await?;
    let temporary = paths.durable.with_extension("db.next");
    let contents = tokio::fs::read(&paths.database)
        .await
        .with_context(|| format!("read SQLite database {}", paths.database.display()))?;
    let mut temporary_file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)
        .await
        .with_context(|| format!("open durable snapshot {}", temporary.display()))?;
    temporary_file.write_all(&contents).await?;
    temporary_file.sync_all().await?;
    drop(temporary_file);
    tokio::fs::rename(&temporary, &paths.durable)
        .await
        .with_context(|| format!("publish durable snapshot {}", paths.durable.display()))?;
    Ok(())
}

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/config", get(get_config).put(put_config))
        .route("/api/v1/history", get(history))
        .route("/api/v1/preview", post(preview))
        .route("/api/v1/relay/{channel}", post(relay))
        .route("/api/v1/demo/sessions", post(create_demo_session))
        .route(
            "/api/v1/demo/sessions/{session}",
            axum::routing::delete(delete_demo_session),
        )
        .route(
            "/api/v1/demo/sessions/{session}/preview",
            post(demo_preview),
        )
        .with_state(state)
        .layer(RequestBodyLimitLayer::new(262_144))
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in; frame-ancestors 'none'"),
        ))
        .layer(TraceLayer::new_for_http())
}

async fn migrate(db: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS channels (
            id TEXT PRIMARY KEY, config TEXT NOT NULL, updated_at TEXT NOT NULL
         )",
    )
    .execute(db)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS deliveries (
            id TEXT PRIMARY KEY, channel_id TEXT NOT NULL, service TEXT NOT NULL,
            status TEXT NOT NULL, fingerprint TEXT NOT NULL, created_at TEXT NOT NULL,
            evidence_items INTEGER NOT NULL, evidence_bytes INTEGER NOT NULL
         )",
    )
    .execute(db)
    .await?;
    Ok(())
}

async fn seed_default(db: &SqlitePool) -> anyhow::Result<()> {
    let config = ChannelConfig::default();
    sqlx::query("INSERT OR IGNORE INTO channels (id, config, updated_at) VALUES (?, ?, ?)")
        .bind(&config.id)
        .bind(serde_json::to_string(&config)?)
        .bind(Utc::now().to_rfc3339())
        .execute(db)
        .await?;
    Ok(())
}

const DEVELOPMENT_BUILD_ID: &str = "development";

fn build_identity(compiled_identity: Option<&'static str>) -> &'static str {
    compiled_identity
        .filter(|identity| !identity.trim().is_empty())
        .unwrap_or(DEVELOPMENT_BUILD_ID)
}

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "build": build_identity(option_env!("BUILD_SHA"))
    }))
}

fn authorize(headers: &HeaderMap, state: &AppState) -> Result<(), ApiError> {
    let supplied = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    if supplied
        .as_bytes()
        .ct_eq(state.admin_token.as_bytes())
        .into()
    {
        Ok(())
    } else {
        Err(ApiError::Unauthorized(
            "admin token required; read the persisted token on the relay host".into(),
        ))
    }
}

async fn get_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ChannelConfig>, ApiError> {
    authorize(&headers, &state)?;
    Ok(Json(load_config(&state.db, "primary").await?))
}

async fn put_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ChannelConfig>, ApiError> {
    authorize(&headers, &state)?;
    let mut config: ChannelConfig = serde_json::from_slice(&body)
        .map_err(|_| ApiError::BadRequest("request body must be valid channel JSON".into()))?;
    config.id = "primary".into();
    validate_config(&config)?;
    let _persistence = state.persistence_lock.lock().await;
    sqlx::query("INSERT INTO channels (id, config, updated_at) VALUES (?, ?, ?) ON CONFLICT(id) DO UPDATE SET config=excluded.config, updated_at=excluded.updated_at")
        .bind(&config.id)
        .bind(serde_json::to_string(&config).map_err(|e| ApiError::Internal(e.into()))?)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;
    persist_database_snapshot(&state)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(config))
}

fn validate_config(config: &ChannelConfig) -> Result<(), ApiError> {
    if config.name.trim().is_empty() || config.name.len() > 80 {
        return Err(ApiError::BadRequest(
            "channel name must contain 1–80 characters".into(),
        ));
    }
    if !(1..=100).contains(&config.max_items) {
        return Err(ApiError::BadRequest(
            "evidence item cap must be between 1 and 100".into(),
        ));
    }
    if !(1_024..=131_072).contains(&config.max_bytes) {
        return Err(ApiError::BadRequest(
            "evidence byte cap must be between 1 KB and 128 KB".into(),
        ));
    }
    if config.redact_fields.len() > 100 || config.redact_fields.iter().any(|v| v.len() > 80) {
        return Err(ApiError::BadRequest("redaction policy is too large".into()));
    }
    for pointer in [
        &config.query_pointer,
        &config.evidence_pointer,
        &config.service_pointer,
        &config.error_pointer,
        &config.time_pointer,
    ] {
        if !pointer.starts_with('/') {
            return Err(ApiError::BadRequest(
                "JSON pointers must start with /".into(),
            ));
        }
    }
    for raw in [&config.source_url, &config.destination_url]
        .into_iter()
        .flatten()
    {
        let parsed =
            Url::parse(raw).map_err(|_| ApiError::BadRequest("endpoint URL is invalid".into()))?;
        let local = matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
        if parsed.scheme() != "https" && !local {
            return Err(ApiError::BadRequest(
                "endpoint URLs must use HTTPS (localhost is allowed for development)".into(),
            ));
        }
    }
    if !matches!(
        config.destination_kind.as_str(),
        "json" | "slack" | "email-webhook"
    ) {
        return Err(ApiError::BadRequest(
            "destination kind must be json, slack, or email-webhook".into(),
        ));
    }
    Ok(())
}

async fn load_config(db: &SqlitePool, id: &str) -> Result<ChannelConfig, ApiError> {
    let row = sqlx::query("SELECT config FROM channels WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?
        .ok_or(ApiError::NotFound)?;
    serde_json::from_str(row.get("config")).map_err(|e| ApiError::Internal(e.into()))
}

async fn history(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<DeliveryRecord>>, ApiError> {
    authorize(&headers, &state)?;
    let rows = sqlx::query("SELECT id, service, status, fingerprint, created_at, evidence_items, evidence_bytes FROM deliveries ORDER BY created_at DESC LIMIT 20")
        .fetch_all(&state.db)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;
    Ok(Json(
        rows.into_iter()
            .map(|r| DeliveryRecord {
                id: r.get("id"),
                service: r.get("service"),
                status: r.get("status"),
                fingerprint: r.get("fingerprint"),
                created_at: r.get("created_at"),
                evidence_items: r.get("evidence_items"),
                evidence_bytes: r.get("evidence_bytes"),
            })
            .collect(),
    ))
}

async fn preview(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<EvidenceEnvelope>, ApiError> {
    authorize(&headers, &state)?;
    let request: PreviewRequest = serde_json::from_slice(&body)
        .map_err(|_| ApiError::BadRequest("request body must be valid preview JSON".into()))?;
    let mut config = load_config(&state.db, "primary").await?;
    if let Some(fields) = request.redact_fields {
        config.redact_fields = fields;
    }
    if let Some(cap) = request.max_items {
        config.max_items = cap;
    }
    if let Some(cap) = request.max_bytes {
        config.max_bytes = cap;
    }
    validate_config(&config)?;
    let evidence = request
        .evidence
        .or_else(|| extract_evidence(&request.alert, &config.evidence_pointer))
        .unwrap_or_default();
    Ok(Json(build_envelope(
        &request.alert,
        evidence,
        &config,
        &state.signing_key,
        false,
    )?))
}

async fn create_demo_session(State(state): State<AppState>) -> Json<DemoSession> {
    let now = Utc::now();
    let expires_at = now + chrono::Duration::hours(24);
    let id = Uuid::new_v4().to_string();
    let mut sessions = state.demo_sessions.write().await;
    sessions.retain(|_, expiry| *expiry > now);
    sessions.insert(id.clone(), expires_at);
    Json(DemoSession {
        id,
        expires_at: expires_at.to_rfc3339(),
    })
}

async fn delete_demo_session(
    State(state): State<AppState>,
    Path(session): Path<String>,
) -> StatusCode {
    state.demo_sessions.write().await.remove(&session);
    StatusCode::NO_CONTENT
}

async fn demo_preview(
    State(state): State<AppState>,
    Path(session): Path<String>,
    body: Bytes,
) -> Result<Json<EvidenceEnvelope>, ApiError> {
    let now = Utc::now();
    let mut sessions = state.demo_sessions.write().await;
    sessions.retain(|_, expiry| *expiry > now);
    if !sessions.contains_key(&session) {
        return Err(ApiError::NotFound);
    }
    drop(sessions);
    let request: PreviewRequest = serde_json::from_slice(&body)
        .map_err(|_| ApiError::BadRequest("request body must be valid preview JSON".into()))?;
    let mut config = ChannelConfig::default();
    if let Some(fields) = request.redact_fields {
        config.redact_fields = fields;
    }
    if let Some(cap) = request.max_items {
        config.max_items = cap;
    }
    if let Some(cap) = request.max_bytes {
        config.max_bytes = cap;
    }
    validate_config(&config)?;
    let evidence = request
        .evidence
        .or_else(|| extract_evidence(&request.alert, &config.evidence_pointer))
        .unwrap_or_default();
    Ok(Json(build_envelope(
        &request.alert,
        evidence,
        &config,
        &state.signing_key,
        false,
    )?))
}

async fn relay(
    State(state): State<AppState>,
    Path(channel): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<RelayResult>), ApiError> {
    let supplied = headers
        .get("x-envelope-token")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if !bool::from(supplied.as_bytes().ct_eq(state.inbound_token.as_bytes())) {
        return Err(ApiError::Unauthorized(
            "inbound relay token required in x-envelope-token".into(),
        ));
    }
    let config = load_config(&state.db, &channel).await?;
    if !config.enabled {
        return Err(ApiError::BadRequest("channel is paused".into()));
    }
    let alert: Value = serde_json::from_slice(&body)
        .map_err(|_| ApiError::BadRequest("request body must be valid JSON".into()))?;
    let provider_signature = headers
        .get("x-signature")
        .or_else(|| headers.get("x-grafana-alerting-signature"))
        .or_else(|| headers.get("x-signoz-signature"))
        .and_then(|v| v.to_str().ok());
    let evidence = if config.source_url.is_some() {
        fetch_evidence(&state, &config, &alert).await?
    } else {
        extract_evidence(&alert, &config.evidence_pointer).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "no evidence found at {}; configure a source URL or include bounded evidence",
                config.evidence_pointer
            ))
        })?
    };
    let envelope = build_envelope(
        &alert,
        evidence,
        &config,
        &state.signing_key,
        provider_signature.is_some(),
    )?;
    let has_destination = destination(&state, &config).is_some();
    let delivery = if has_destination {
        deliver(&state, &config, &envelope, provider_signature).await
    } else {
        Ok("not configured; returned to caller".into())
    };
    let (status, delivery_label) = match &delivery {
        Ok(label) if has_destination => ("delivered", label.clone()),
        Ok(label) => ("created", label.clone()),
        Err(error) => ("failed", error.to_string()),
    };
    record_delivery(&state, &config.id, &envelope, status).await?;
    delivery?;
    Ok((
        StatusCode::ACCEPTED,
        Json(RelayResult {
            status: status.into(),
            delivery: delivery_label,
            envelope,
        }),
    ))
}

fn extract_evidence(alert: &Value, pointer: &str) -> Option<Vec<Value>> {
    match alert.pointer(pointer)? {
        Value::Array(items) => Some(items.clone()),
        Value::Object(map) => map
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .or_else(|| Some(vec![Value::Object(map.clone())])),
        item if !item.is_null() => Some(vec![item.clone()]),
        _ => None,
    }
}

async fn fetch_evidence(
    state: &AppState,
    config: &ChannelConfig,
    alert: &Value,
) -> Result<Vec<Value>, ApiError> {
    let mut url = Url::parse(config.source_url.as_deref().unwrap())
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    let query = scalar_at(alert, &config.query_pointer).unwrap_or_default();
    url.query_pairs_mut()
        .append_pair("q", &query)
        .append_pair("limit", &config.max_items.to_string());
    let mut request = state.client.get(url);
    if let Some(token) = &state.upstream_token {
        request = request.bearer_auth(token.as_str());
    }
    let mut response = request
        .send()
        .await
        .map_err(|e| ApiError::Upstream(clean_network_error(&e.to_string())))?;
    if !response.status().is_success() {
        return Err(ApiError::Upstream(format!(
            "source returned HTTP {}",
            response.status().as_u16()
        )));
    }
    let response_cap = config.max_bytes.saturating_mul(4).clamp(262_144, 524_288);
    if response
        .content_length()
        .is_some_and(|size| size > response_cap as u64)
    {
        return Err(ApiError::Upstream(format!(
            "source response exceeded the {} byte fetch cap",
            response_cap
        )));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| ApiError::Upstream(clean_network_error(&e.to_string())))?
    {
        if body.len() + chunk.len() > response_cap {
            return Err(ApiError::Upstream(format!(
                "source response exceeded the {} byte fetch cap",
                response_cap
            )));
        }
        body.extend_from_slice(&chunk);
    }
    let value: Value = serde_json::from_slice(&body)
        .map_err(|_| ApiError::Upstream("source response was not JSON".into()))?;
    let data = value
        .get("data")
        .or_else(|| value.get("results"))
        .unwrap_or(&value);
    match data {
        Value::Array(items) => Ok(items.clone()),
        Value::Object(_) => Ok(vec![data.clone()]),
        _ => Err(ApiError::Upstream(
            "source response did not contain evidence objects".into(),
        )),
    }
}

fn build_envelope(
    alert: &Value,
    mut evidence: Vec<Value>,
    config: &ChannelConfig,
    key: &[u8],
    source_signature_preserved: bool,
) -> Result<EvidenceEnvelope, ApiError> {
    let original_count = evidence.len();
    evidence.truncate(config.max_items);
    let fields: HashSet<String> = config
        .redact_fields
        .iter()
        .map(|v| v.to_ascii_lowercase())
        .collect();
    for item in &mut evidence {
        redact_value(item, &fields);
    }
    let mut truncated = original_count > evidence.len();
    while !evidence.is_empty()
        && serde_json::to_vec(&evidence)
            .map_err(|e| ApiError::Internal(e.into()))?
            .len()
            > config.max_bytes
    {
        evidence.pop();
        truncated = true;
    }
    let evidence_bytes = serde_json::to_vec(&evidence)
        .map_err(|e| ApiError::Internal(e.into()))?
        .len();
    let service = scalar_at(alert, &config.service_pointer)
        .or_else(|| find_scalar(alert, &["service", "service_name", "app"]))
        .unwrap_or_else(|| "unknown service".into());
    let error_signature = scalar_at(alert, &config.error_pointer)
        .or_else(|| find_scalar(alert, &["error", "message", "title", "alertname"]))
        .or_else(|| {
            evidence
                .first()
                .and_then(|v| find_scalar(v, &["error", "message", "exception"]))
        })
        .unwrap_or_else(|| "No error signature supplied".into());
    let first_seen = scalar_at(alert, &config.time_pointer)
        .or_else(|| find_scalar(alert, &["startsAt", "timestamp", "time", "first_seen"]))
        .or_else(|| {
            evidence
                .first()
                .and_then(|v| find_scalar(v, &["timestamp", "time", "ts"]))
        })
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    let query = scalar_at(alert, &config.query_pointer).unwrap_or_default();
    let query_fingerprint = hex::encode(Sha256::digest(format!(
        "{}\n{}",
        config.source_url.as_deref().unwrap_or("embedded"),
        query
    )))[..16]
        .to_string();
    let mut envelope = EvidenceEnvelope {
        schema: "alert-evidence-envelope/v1".into(),
        id: Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        channel: config.id.clone(),
        summary: EnvelopeSummary {
            service: clip(&service, 120),
            error_signature: clip(&error_signature, 240),
            first_seen: clip(&first_seen, 80),
        },
        query_fingerprint,
        evidence_items: evidence.len(),
        evidence_bytes,
        evidence,
        truncated,
        redacted_fields: config.redact_fields.clone(),
        source_signature_preserved,
        signature: String::new(),
    };
    let unsigned = serde_json::to_vec(&envelope).map_err(|e| ApiError::Internal(e.into()))?;
    let mut mac =
        Hmac::<Sha256>::new_from_slice(key).map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;
    mac.update(&unsigned);
    envelope.signature = format!("hmac-sha256={}", hex::encode(mac.finalize().into_bytes()));
    Ok(envelope)
}

fn redact_value(value: &mut Value, fields: &HashSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if fields.contains(&key.to_ascii_lowercase()) {
                    *child = Value::String("[REDACTED]".into());
                } else {
                    redact_value(child, fields);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_value(item, fields);
            }
        }
        _ => {}
    }
}

fn scalar_at(value: &Value, pointer: &str) -> Option<String> {
    value.pointer(pointer).and_then(value_to_string)
}

fn find_scalar(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(found) = map
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(key))
                    .and_then(|(_, v)| value_to_string(v))
                {
                    return Some(found);
                }
            }
            map.values().find_map(|v| find_scalar(v, keys))
        }
        Value::Array(items) => items.iter().find_map(|v| find_scalar(v, keys)),
        _ => None,
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(v) => Some(v.clone()),
        Value::Number(v) => Some(v.to_string()),
        Value::Bool(v) => Some(v.to_string()),
        _ => None,
    }
}

fn clip(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

fn destination<'a>(state: &'a AppState, config: &'a ChannelConfig) -> Option<&'a str> {
    state
        .destination_url_override
        .as_deref()
        .map(String::as_str)
        .or(config.destination_url.as_deref())
}

async fn deliver(
    state: &AppState,
    config: &ChannelConfig,
    envelope: &EvidenceEnvelope,
    original_signature: Option<&str>,
) -> Result<String, ApiError> {
    let Some(url) = destination(state, config) else {
        return Err(ApiError::Delivery(
            "destination is not configured; the signed envelope was returned to the caller".into(),
        ));
    };
    let payload = if config.destination_kind == "slack" {
        json!({ "text": format!("Evidence sealed · {}\n{}\nFirst seen {} · {} items · fingerprint {}", envelope.summary.service, envelope.summary.error_signature, envelope.summary.first_seen, envelope.evidence_items, envelope.query_fingerprint) })
    } else {
        serde_json::to_value(envelope).map_err(|e| ApiError::Internal(e.into()))?
    };
    let mut request = state
        .client
        .post(url)
        .json(&payload)
        .header("x-evidence-envelope-signature", &envelope.signature);
    if let Some(signature) = original_signature {
        request = request.header("x-original-provider-signature", signature);
    }
    if let Some(token) = &state.destination_token {
        request = request.bearer_auth(token.as_str());
    }
    let response = request
        .send()
        .await
        .map_err(|e| ApiError::Delivery(clean_network_error(&e.to_string())))?;
    if !response.status().is_success() {
        return Err(ApiError::Delivery(format!(
            "destination returned HTTP {}",
            response.status().as_u16()
        )));
    }
    Ok(config.destination_kind.clone())
}

fn clean_network_error(raw: &str) -> String {
    raw.split('?')
        .next()
        .unwrap_or("network request failed")
        .chars()
        .take(180)
        .collect()
}

async fn record_delivery(
    state: &AppState,
    channel: &str,
    envelope: &EvidenceEnvelope,
    status: &str,
) -> Result<(), ApiError> {
    let _persistence = state.persistence_lock.lock().await;
    sqlx::query("INSERT INTO deliveries (id, channel_id, service, status, fingerprint, created_at, evidence_items, evidence_bytes) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&envelope.id).bind(channel).bind(&envelope.summary.service).bind(status)
        .bind(&envelope.query_fingerprint).bind(&envelope.created_at)
        .bind(envelope.evidence_items as i64).bind(envelope.evidence_bytes as i64)
        .execute(&state.db).await.map_err(|e| ApiError::Internal(e.into()))?;
    sqlx::query("DELETE FROM deliveries WHERE id NOT IN (SELECT id FROM deliveries ORDER BY created_at DESC LIMIT 20)")
        .execute(&state.db)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;
    persist_database_snapshot(state)
        .await
        .map_err(ApiError::Internal)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    #[test]
    fn recursively_redacts_and_bounds_evidence() {
        // @claim:bounded-redacted-signed
        let alert = json!({"service":"checkout", "error":"card declined", "startsAt":"2026-08-27T12:00:00Z", "query":"service=checkout"});
        let evidence = vec![
            json!({"message":"failed", "user":{"email":"a@example.com"}, "token":"secret"}),
            json!({"message":"second"}),
        ];
        let config = ChannelConfig {
            max_items: 1,
            ..Default::default()
        };
        let output = build_envelope(&alert, evidence, &config, b"test-key", true).unwrap();
        assert_eq!(output.summary.service, "checkout");
        assert_eq!(output.evidence_items, 1);
        assert!(output.truncated);
        assert_eq!(output.evidence[0]["user"]["email"], "[REDACTED]");
        assert_eq!(output.evidence[0]["token"], "[REDACTED]");
        assert!(output.signature.starts_with("hmac-sha256="));
        assert!(output.source_signature_preserved);
    }

    #[test]
    fn rejects_unsafe_remote_urls() {
        let mut config = ChannelConfig {
            source_url: Some("http://logs.example.com/query".into()),
            ..Default::default()
        };
        assert!(validate_config(&config).is_err());
        config.source_url = Some("http://localhost:9090/query".into());
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn clips_by_serialized_byte_size() {
        let config = ChannelConfig {
            max_bytes: 1024,
            ..Default::default()
        };
        let evidence = vec![
            json!({"message":"x".repeat(2000)}),
            json!({"message":"small"}),
        ];
        let output = build_envelope(&json!({}), evidence, &config, b"key", false).unwrap();
        assert_eq!(output.evidence_items, 0);
        assert!(output.truncated);
        assert!(output.evidence_bytes <= 1024);
    }

    #[test]
    fn build_identity_never_exposes_an_empty_compile_time_value() {
        assert_eq!(build_identity(None), DEVELOPMENT_BUILD_ID);
        assert_eq!(build_identity(Some("")), DEVELOPMENT_BUILD_ID);
        assert_eq!(build_identity(Some("   \t")), DEVELOPMENT_BUILD_ID);
        assert_eq!(
            build_identity(Some("0123456789abcdef0123456789abcdef01234567")),
            "0123456789abcdef0123456789abcdef01234567"
        );
    }

    #[test]
    fn signing_key_is_generated_once_and_can_be_overridden() {
        let directory = tempfile::tempdir().unwrap();
        let key_path = directory.path().join("instance.key");

        let (generated, source) = load_or_generate_signing_key(&key_path, None).unwrap();
        assert_eq!(source, SigningKeySource::Generated);
        assert_eq!(generated.len(), 32);

        let (persisted, source) = load_or_generate_signing_key(&key_path, None).unwrap();
        assert_eq!(source, SigningKeySource::Persisted);
        assert_eq!(persisted, generated);

        let override_key = "a".repeat(32);
        let (supplied, source) =
            load_or_generate_signing_key(&key_path, Some(&override_key)).unwrap();
        assert_eq!(source, SigningKeySource::Supplied);
        assert_eq!(supplied, override_key.as_bytes());
        assert_eq!(std::fs::read(key_path).unwrap(), generated);
    }

    #[test]
    fn access_tokens_are_generated_once_and_can_be_overridden() {
        let directory = tempfile::tempdir().unwrap();
        let token_path = directory.path().join("admin.token");

        let (generated, source) = load_or_generate_token(&token_path, None).unwrap();
        assert_eq!(source, SigningKeySource::Generated);
        assert_eq!(generated.len(), 64);

        let (persisted, source) = load_or_generate_token(&token_path, None).unwrap();
        assert_eq!(source, SigningKeySource::Persisted);
        assert_eq!(persisted, generated);

        let supplied = "z".repeat(32);
        let (loaded, source) = load_or_generate_token(&token_path, Some(&supplied)).unwrap();
        assert_eq!(source, SigningKeySource::Supplied);
        assert_eq!(loaded, supplied);
    }

    #[test]
    fn signing_key_rejects_short_values() {
        let directory = tempfile::tempdir().unwrap();
        assert!(
            load_or_generate_signing_key(&directory.path().join("key"), Some("too-short")).is_err()
        );
    }

    #[tokio::test]
    async fn claim_provider_signature_is_preserved_across_relay() {
        // @claim:provider-signature
        type Capture =
            Arc<tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<(HeaderMap, Value)>>>>;
        async fn capture(
            State(capture): State<Capture>,
            headers: HeaderMap,
            Json(body): Json<Value>,
        ) -> StatusCode {
            if let Some(sender) = capture.lock().await.take() {
                let _ = sender.send((headers, body));
            }
            StatusCode::NO_CONTENT
        }
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let capture_state: Capture = Arc::new(tokio::sync::Mutex::new(Some(sender)));
        let sink = Router::new()
            .route("/capture", post(capture))
            .with_state(capture_state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let destination = format!("http://{}/capture", listener.local_addr().unwrap());
        let sink_task = tokio::spawn(async move { axum::serve(listener, sink).await.unwrap() });

        let file = tempfile::NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", file.path().display());
        let state = create_state(
            &database_url,
            b"test-signing-key-with-at-least-32-bytes".to_vec(),
            "test-admin-token-with-at-least-32-characters".into(),
            "test-inbound-token-with-at-least-32-characters".into(),
        )
        .await
        .unwrap();
        let app = Router::new()
            .route("/health", get(health))
            .merge(api_router(state));

        let health = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(health.status(), StatusCode::OK);

        let config = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/config")
                    .header(
                        "authorization",
                        "Bearer test-admin-token-with-at-least-32-characters",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(config.status(), StatusCode::OK);

        let updated = ChannelConfig {
            name: "Changed route".into(),
            destination_url: Some(destination),
            ..Default::default()
        };
        let request = Request::builder()
            .method("PUT")
            .uri("/api/v1/config")
            .header("content-type", "application/json")
            .header(
                "authorization",
                "Bearer test-admin-token-with-at-least-32-characters",
            )
            .body(Body::from(serde_json::to_vec(&updated).unwrap()))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let preview_body =
            json!({"alert":{"service":"api","error":"boom","evidence":[{"token":"secret"}]}})
                .to_string();
        let preview = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/preview")
                    .header("content-type", "application/json")
                    .header(
                        "authorization",
                        "Bearer test-admin-token-with-at-least-32-characters",
                    )
                    .body(Body::from(preview_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(preview.status(), StatusCode::OK);
        let bytes = to_bytes(preview.into_body(), 100_000).await.unwrap();
        let preview_json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(preview_json["evidence"][0]["token"], "[REDACTED]");

        let relay_body = json!({"service":"api","error":"boom","startsAt":"2026-08-27T00:00:00Z","evidence":[{"message":"failed"}]}).to_string();
        let relay = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/relay/primary")
                    .header("content-type", "application/json")
                    .header("x-signature", "provider-sig")
                    .header(
                        "x-envelope-token",
                        "test-inbound-token-with-at-least-32-characters",
                    )
                    .body(Body::from(relay_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(relay.status(), StatusCode::ACCEPTED);
        let (forwarded_headers, forwarded) = tokio::time::timeout(Duration::from_secs(2), receiver)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(forwarded["summary"]["service"], "api");
        assert_eq!(
            forwarded_headers
                .get("x-original-provider-signature")
                .unwrap(),
            "provider-sig"
        );
        assert!(forwarded_headers
            .get("x-evidence-envelope-signature")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("hmac-sha256="));

        let history = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/history")
                    .header(
                        "authorization",
                        "Bearer test-admin-token-with-at-least-32-characters",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(history.status(), StatusCode::OK);
        let bytes = to_bytes(history.into_body(), 100_000).await.unwrap();
        let history_json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(history_json.as_array().unwrap().len(), 1);

        sink_task.abort();
    }

    #[tokio::test]
    async fn unauthorized_requests_are_rejected_before_body_parsing() {
        // @claim:protected-real-apis
        let file = tempfile::NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", file.path().display());
        let state = create_state(
            &database_url,
            b"test-signing-key-with-at-least-32-bytes".to_vec(),
            "test-admin-token-with-at-least-32-characters".into(),
            "test-inbound-token-with-at-least-32-characters".into(),
        )
        .await
        .unwrap();
        let app = api_router(state.clone());

        for (method, uri) in [
            ("GET", "/api/v1/config"),
            ("GET", "/api/v1/history"),
            ("PUT", "/api/v1/config"),
            ("POST", "/api/v1/preview"),
            ("POST", "/api/v1/relay/primary"),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(uri)
                        .header("content-type", "application/json")
                        .body(Body::from("{"))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "{method} {uri}"
            );
        }
    }

    #[tokio::test]
    async fn demo_session_is_ephemeral_and_cannot_reach_real_history() {
        // @claim:isolated-demo
        let file = tempfile::NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", file.path().display());
        let state = create_state(
            &database_url,
            b"test-signing-key-with-at-least-32-bytes".to_vec(),
            "test-admin-token-with-at-least-32-characters".into(),
            "test-inbound-token-with-at-least-32-characters".into(),
        )
        .await
        .unwrap();
        let app = api_router(state);
        let session = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/demo/sessions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(session.status(), StatusCode::OK);
        let bytes = to_bytes(session.into_body(), 10_000).await.unwrap();
        let session: Value = serde_json::from_slice(&bytes).unwrap();
        let id = session["id"].as_str().unwrap();

        let sample = json!({
            "alert": {
                "service": "checkout-api",
                "error": "payment authorization timed out",
                "startsAt": "2026-08-27T14:32:08Z",
                "evidence": [{"email": "customer@example.com", "token": "secret"}]
            }
        });
        let preview = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/demo/sessions/{id}/preview"))
                    .header("content-type", "application/json")
                    .body(Body::from(sample.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(preview.status(), StatusCode::OK);
        let bytes = to_bytes(preview.into_body(), 100_000).await.unwrap();
        let preview: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(preview["summary"]["service"], "checkout-api");
        assert_eq!(preview["evidence"][0]["email"], "[REDACTED]");

        let history = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/history")
                    .header(
                        "authorization",
                        "Bearer test-admin-token-with-at-least-32-characters",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = to_bytes(history.into_body(), 10_000).await.unwrap();
        let history: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(history, json!([]));
    }

    #[tokio::test]
    async fn claim_raw_payload_is_not_persisted() {
        // @claim:raw-not-retained
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("metadata.db");
        let database_url = format!("sqlite:{}?mode=rwc", database_path.display());
        let state = create_state(
            &database_url,
            b"test-signing-key-with-at-least-32-bytes".to_vec(),
            "test-admin-token-with-at-least-32-characters".into(),
            "test-inbound-token-with-at-least-32-characters".into(),
        )
        .await
        .unwrap();
        let db = state.db.clone();
        let app = api_router(state);
        let private_email = "private-marker-83@example.test";
        let private_token = "secret-marker-a91f";
        let relay_body = json!({
            "service": "retention-check",
            "error": "private error marker 714",
            "startsAt": "2026-08-30T00:00:00Z",
            "evidence": [{"message": "private evidence marker 552", "email": private_email, "token": private_token}]
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/relay/primary")
                    .header("content-type", "application/json")
                    .header(
                        "x-envelope-token",
                        "test-inbound-token-with-at-least-32-characters",
                    )
                    .body(Body::from(relay_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let stored: (String, String) = sqlx::query_as(
            "SELECT service, fingerprint FROM deliveries WHERE service = 'retention-check'",
        )
        .fetch_one(&db)
        .await
        .unwrap();
        assert_eq!(stored.0, "retention-check");
        assert_eq!(stored.1.len(), 16);
        db.close().await;
        let database_bytes = std::fs::read(database_path).unwrap();
        for forbidden in [
            private_email,
            private_token,
            "private evidence marker 552",
            "private error marker 714",
        ] {
            assert!(!database_bytes
                .windows(forbidden.len())
                .any(|window| window == forbidden.as_bytes()));
        }
    }

    #[tokio::test]
    async fn persistent_sqlite_serializes_connections_for_mounted_storage() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", file.path().display());
        let state = create_state(
            &database_url,
            b"test-signing-key-with-at-least-32-bytes".to_vec(),
            "test-admin-token-with-at-least-32-characters".into(),
            "test-inbound-token-with-at-least-32-characters".into(),
        )
        .await
        .unwrap();

        let held = state.db.acquire().await.unwrap();
        assert!(
            state.db.try_acquire().is_none(),
            "the persistent SQLite pool must expose only one connection"
        );
        drop(held);
        let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&state.db)
            .await
            .unwrap();
        let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(journal_mode, "delete");
        assert_eq!(busy_timeout, 30_000);
    }

    #[tokio::test]
    async fn durable_snapshot_restores_committed_route_state() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("runtime.db");
        let durable = directory
            .path()
            .join("mounted")
            .join("envelopes.snapshot.db");
        let database_url = format!("sqlite:{}?mode=rwc", database.display());
        let state = create_state_with_snapshot(
            &database_url,
            b"test-signing-key-with-at-least-32-bytes".to_vec(),
            "test-admin-token-with-at-least-32-characters".into(),
            "test-inbound-token-with-at-least-32-characters".into(),
            Some((database.clone(), durable.clone())),
        )
        .await
        .unwrap();
        let config = ChannelConfig {
            name: "Persisted after revision replacement".into(),
            ..Default::default()
        };
        let response = api_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/config")
                    .header("content-type", "application/json")
                    .header(
                        "authorization",
                        "Bearer test-admin-token-with-at-least-32-characters",
                    )
                    .body(Body::from(serde_json::to_vec(&config).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(std::fs::metadata(&durable).unwrap().len() > 0);
        state.db.close().await;
        std::fs::remove_file(&database).unwrap();

        let restored = create_state_with_snapshot(
            &database_url,
            b"test-signing-key-with-at-least-32-bytes".to_vec(),
            "test-admin-token-with-at-least-32-characters".into(),
            "test-inbound-token-with-at-least-32-characters".into(),
            Some((database, durable)),
        )
        .await
        .unwrap();
        assert_eq!(
            load_config(&restored.db, "primary").await.unwrap().name,
            "Persisted after revision replacement"
        );
    }

    #[tokio::test]
    async fn claim_history_keeps_only_the_latest_twenty_records() {
        // @claim:history-limit
        let file = tempfile::NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", file.path().display());
        let state = create_state(
            &database_url,
            b"test-signing-key-with-at-least-32-bytes".to_vec(),
            "test-admin-token-with-at-least-32-characters".into(),
            "test-inbound-token-with-at-least-32-characters".into(),
        )
        .await
        .unwrap();
        let app = api_router(state.clone());
        for index in 0..25 {
            let body = json!({
                "service": format!("service-{index}"),
                "error": "bounded failure",
                "startsAt": format!("2026-08-30T00:00:{index:02}Z"),
                "evidence": [{"message": "safe"}]
            });
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/relay/primary")
                        .header("content-type", "application/json")
                        .header(
                            "x-envelope-token",
                            "test-inbound-token-with-at-least-32-characters",
                        )
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::ACCEPTED);
        }
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/history")
                    .header(
                        "authorization",
                        "Bearer test-admin-token-with-at-least-32-characters",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = to_bytes(response.into_body(), 100_000).await.unwrap();
        let history: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(history.as_array().unwrap().len(), 20);
        let retained: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM deliveries")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(retained, 20);
    }
}
