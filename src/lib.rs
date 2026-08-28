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
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::{collections::HashSet, sync::Arc, time::Duration};
use subtle::ConstantTimeEq;
use thiserror::Error;
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
    pub admin_token: Option<Arc<String>>,
    pub inbound_token: Option<Arc<String>>,
    pub upstream_token: Option<Arc<String>>,
    pub destination_token: Option<Arc<String>>,
    pub destination_url_override: Option<Arc<String>>,
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

#[derive(Debug, Deserialize)]
struct PreviewRequest {
    alert: Value,
    evidence: Option<Vec<Value>>,
    redact_fields: Option<Vec<String>>,
    max_items: Option<usize>,
    max_bytes: Option<usize>,
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
    #[error("configuration is not authorized")]
    Unauthorized,
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
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
            Self::Delivery(_) => StatusCode::BAD_GATEWAY,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let public_message = self.to_string();
        (status, Json(json!({ "error": public_message }))).into_response()
    }
}

pub async fn create_state(database_url: &str) -> anyhow::Result<AppState> {
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    migrate(&db).await?;
    seed_default(&db).await?;
    Ok(AppState {
        db,
        client: Client::builder()
            .timeout(Duration::from_secs(8))
            .user_agent("alert-evidence-envelope/0.1")
            .build()?,
        signing_key: Arc::new(
            std::env::var("ENVELOPE_SIGNING_KEY")
                .unwrap_or_else(|_| "development-only-change-me".into())
                .into_bytes(),
        ),
        admin_token: std::env::var("ADMIN_TOKEN").ok().map(Arc::new),
        inbound_token: std::env::var("INBOUND_TOKEN").ok().map(Arc::new),
        upstream_token: std::env::var("UPSTREAM_BEARER_TOKEN").ok().map(Arc::new),
        destination_token: std::env::var("DESTINATION_BEARER_TOKEN").ok().map(Arc::new),
        destination_url_override: std::env::var("DESTINATION_URL").ok().map(Arc::new),
    })
}

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/config", get(get_config).put(put_config))
        .route("/api/v1/history", get(history))
        .route("/api/v1/preview", post(preview))
        .route("/api/v1/relay/{channel}", post(relay))
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
            HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in"),
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

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "build": option_env!("BUILD_SHA").unwrap_or("development")
    }))
}

fn authorize(headers: &HeaderMap, state: &AppState) -> Result<(), ApiError> {
    let Some(expected) = &state.admin_token else {
        return Ok(());
    };
    let supplied = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    if supplied.as_bytes().ct_eq(expected.as_bytes()).into() {
        Ok(())
    } else {
        Err(ApiError::Unauthorized)
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
    Json(mut config): Json<ChannelConfig>,
) -> Result<Json<ChannelConfig>, ApiError> {
    authorize(&headers, &state)?;
    config.id = "primary".into();
    validate_config(&config)?;
    sqlx::query("INSERT INTO channels (id, config, updated_at) VALUES (?, ?, ?) ON CONFLICT(id) DO UPDATE SET config=excluded.config, updated_at=excluded.updated_at")
        .bind(&config.id)
        .bind(serde_json::to_string(&config).map_err(|e| ApiError::Internal(e.into()))?)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;
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
    Json(request): Json<PreviewRequest>,
) -> Result<Json<EvidenceEnvelope>, ApiError> {
    authorize(&headers, &state)?;
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

async fn relay(
    State(state): State<AppState>,
    Path(channel): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<RelayResult>), ApiError> {
    if let Some(expected) = &state.inbound_token {
        let supplied = headers
            .get("x-envelope-token")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        if !bool::from(supplied.as_bytes().ct_eq(expected.as_bytes())) {
            return Err(ApiError::Unauthorized);
        }
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
    record_delivery(&state.db, &config.id, &envelope, status).await?;
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
    db: &SqlitePool,
    channel: &str,
    envelope: &EvidenceEnvelope,
    status: &str,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO deliveries (id, channel_id, service, status, fingerprint, created_at, evidence_items, evidence_bytes) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&envelope.id).bind(channel).bind(&envelope.summary.service).bind(status)
        .bind(&envelope.query_fingerprint).bind(&envelope.created_at)
        .bind(envelope.evidence_items as i64).bind(envelope.evidence_bytes as i64)
        .execute(db).await.map_err(|e| ApiError::Internal(e.into()))?;
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

    #[tokio::test]
    async fn http_routes_complete_a_local_relay_workflow() {
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
        let state = create_state(&database_url).await.unwrap();
        let app = api_router(state);

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
}
