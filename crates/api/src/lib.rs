// API functionality will be implemented here

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Health
#[derive(Serialize)]
struct HealthResponse<'a> {
    status: &'a str,
}

// Basic graph query (placeholder)
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub question: Option<String>,
    pub max_hops: Option<u8>,
    pub include_code: Option<bool>,
    pub include_quality_metrics: Option<bool>,
    pub version: Option<String>,
}

// LLM-optimized context request
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmFocus {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct LlmContextRequest {
    pub focus: LlmFocus,
    pub version: Option<String>,
    pub token_budget: Option<u32>,
    pub max_hops: Option<u8>,
    pub top_k: Option<u32>,
}

// Explain-change request
#[derive(Debug, Deserialize)]
pub struct ExplainChangeRequest {
    pub diff: Option<String>,
    pub from_version: Option<String>,
    pub to_version: Option<String>,
    pub include_tests: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/query", post(query))
        .route("/api/v1/query/llm-context", post(llm_context))
        .route("/api/v1/query/explain-change", post(explain_change))
}

async fn health() -> Json<HealthResponse<'static>> {
    Json(HealthResponse { status: "ok" })
}

async fn query(Json(_req): Json<QueryRequest>) -> Json<Value> {
    // TODO: wire to graph/query engine
    Json(json!({
        "query_id": "q_placeholder",
        "context": { "primary_entities": [], "relationships": [], "quality_metrics": {} },
        "version_info": { "version_id": _req.version.unwrap_or_else(|| "latest".into()) }
    }))
}

async fn llm_context(Json(req): Json<LlmContextRequest>) -> Json<Value> {
    // TODO: implement ranking + schema packaging
    Json(json!({
        "version_id": req.version.unwrap_or_else(|| "latest".into()),
        "focus": { "type": req.focus.r#type, "id": req.focus.id },
        "primary": [],
        "relations": [],
        "quality": {},
        "connascence": {},
        "narrative": "",
        "limits": { "token_budget": req.token_budget.unwrap_or(12000), "truncation": "tail" }
    }))
}

async fn explain_change(Json(_req): Json<ExplainChangeRequest>) -> Json<Value> {
    // TODO: implement diff mapping to affected subgraph
    Json(json!({
        "summary": { "changed_files": 0, "affected_entities": 0, "risk": "unknown", "reasons": [] },
        "subgraph": { "primary": [], "relations": [] },
        "narrative": "",
        "tests_impacted": []
    }))
}