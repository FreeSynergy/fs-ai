// rest.rs — REST + OpenAPI routes for fs-ai.

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::controller::AiController;
use crate::model::KnownModel;

// ── OpenAPI doc ───────────────────────────────────────────────────────────────

#[allow(clippy::needless_for_each)]
#[derive(OpenApi)]
#[openapi(
    paths(list_models, get_status, start_engine, stop_engine, chat),
    components(schemas(KnownModel, EngineStatusBody, StartBody, ChatBody, ChatResponse))
)]
pub struct ApiDoc;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct EngineStatusBody {
    pub running: bool,
    pub port: Option<u16>,
    pub api_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChatBody {
    /// The user question.
    pub question: String,
    /// Optional context string (e.g. active app or screen).
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatResponse {
    pub answer: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StartBody {
    /// Model ID to start (e.g. qwen3-4b).
    pub model_id: String,
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router(ctrl: AiController) -> Router {
    Router::new()
        .route("/ai/models", get(list_models))
        .route("/ai/status", get(get_status))
        .route("/ai/start", post(start_engine))
        .route("/ai/stop", post(stop_engine))
        .route("/ai/chat", post(chat))
        .with_state(ctrl)
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// List available AI models.
#[utoipa::path(get, path = "/ai/models", responses((status = 200, body = Vec<KnownModel>)))]
async fn list_models(State(ctrl): State<AiController>) -> Json<Vec<KnownModel>> {
    Json(ctrl.list_models())
}

/// Get current engine status.
#[utoipa::path(get, path = "/ai/status", responses((status = 200, body = EngineStatusBody)))]
async fn get_status(State(ctrl): State<AiController>) -> Json<EngineStatusBody> {
    let snap = ctrl.snapshot();
    Json(EngineStatusBody {
        running: snap.running,
        port: snap.port,
        api_url: snap.api_url(),
    })
}

/// Start the LLM engine.
#[utoipa::path(
    post,
    path = "/ai/start",
    request_body = StartBody,
    responses((status = 200), (status = 400))
)]
async fn start_engine(State(ctrl): State<AiController>, Json(body): Json<StartBody>) -> StatusCode {
    match ctrl.start(&body.model_id) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

/// Stop the LLM engine.
#[utoipa::path(post, path = "/ai/stop", responses((status = 200)))]
async fn stop_engine(State(ctrl): State<AiController>) -> StatusCode {
    let _ = ctrl.stop();
    StatusCode::OK
}

/// Send a question to the LLM engine and return the answer.
///
/// The engine must be running (`POST /ai/start`) before calling this endpoint.
/// Returns HTTP 503 when the engine is not available.
#[utoipa::path(
    post,
    path = "/ai/chat",
    request_body = ChatBody,
    responses(
        (status = 200, body = ChatResponse),
        (status = 503, description = "Engine not running")
    )
)]
async fn chat(
    State(ctrl): State<AiController>,
    Json(body): Json<ChatBody>,
) -> (StatusCode, Json<ChatResponse>) {
    match ctrl.chat(&body.question, &body.context) {
        Ok(answer) => (
            StatusCode::OK,
            Json(ChatResponse {
                answer,
                ok: true,
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ChatResponse {
                answer: String::new(),
                ok: false,
                error: Some(e),
            }),
        ),
    }
}
