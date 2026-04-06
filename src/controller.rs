// controller.rs — AiController: Facade over the LLM engine.
//
// Knows only the fs-manager-ai AiEngine trait — never the concrete implementation.

use std::sync::{Arc, Mutex};

use fs_manager_ai::{AiEngine, EngineStatus, LlmConfig, LlmEngine, LlmModel};

use crate::conversation::{
    ConversationStore, ConversationTurn, InMemoryConversationStore, TurnRole,
};
use crate::model::{AiModel, KnownModel};

// ── AiController ─────────────────────────────────────────────────────────────

/// Shared controller — cheaply cloneable (Arc-backed).
#[derive(Clone)]
pub struct AiController {
    state: Arc<Mutex<AiModel>>,
    conversation: Arc<dyn ConversationStore>,
}

impl AiController {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AiModel::new())),
            conversation: Arc::new(InMemoryConversationStore::new()),
        }
    }

    /// Create a controller with an explicit conversation store.
    #[must_use]
    pub fn with_conversation(store: impl ConversationStore + 'static) -> Self {
        Self {
            state: Arc::new(Mutex::new(AiModel::new())),
            conversation: Arc::new(store),
        }
    }

    /// Append a user message and the AI's reply to the conversation history.
    pub fn record_exchange(&self, user_msg: &str, ai_reply: &str) {
        self.conversation
            .append(ConversationTurn::now(TurnRole::User, user_msg));
        self.conversation
            .append(ConversationTurn::now(TurnRole::Assistant, ai_reply));
    }

    /// Return the full conversation history.
    pub fn history(&self) -> Vec<ConversationTurn> {
        self.conversation.history()
    }

    /// Clear the conversation history.
    pub fn clear_history(&self) {
        self.conversation.clear();
    }

    /// Snapshot of the current model state.
    #[must_use]
    pub fn snapshot(&self) -> AiModel {
        self.state.lock().unwrap().clone()
    }

    /// List all known models.
    #[must_use]
    pub fn list_models(&self) -> Vec<KnownModel> {
        KnownModel::all()
    }

    /// Start the LLM engine with the given model id.
    ///
    /// Returns `Ok(port)` on success or an error string.
    pub fn start(&self, model_id: &str) -> Result<u16, String> {
        let llm_model = Self::model_from_id(model_id)?;
        let config = LlmConfig {
            model: llm_model,
            ..LlmConfig::default()
        };
        let engine = LlmEngine::new(
            config,
            LlmEngine::default_binary(),
            LlmEngine::default_data_dir(),
        );
        engine.start().map_err(|e| e.to_string())?;

        let EngineStatus::Running { port } = engine.status() else {
            return Err("engine did not start".into());
        };

        let mut state = self.state.lock().unwrap();
        state.set_running(port);
        state.active_model = Some(model_id.to_string());
        Ok(port)
    }

    /// Stop the LLM engine.
    pub fn stop(&self) -> Result<(), String> {
        let snapshot = self.snapshot();
        let model_id = snapshot.active_model.as_deref().ok_or("no active model")?;

        let llm_model = Self::model_from_id(model_id)?;
        let config = LlmConfig {
            model: llm_model,
            ..LlmConfig::default()
        };
        let engine = LlmEngine::new(
            config,
            LlmEngine::default_binary(),
            LlmEngine::default_data_dir(),
        );
        engine.stop().map_err(|e| e.to_string())?;

        let mut state = self.state.lock().unwrap();
        state.set_stopped();
        state.active_model = None;
        Ok(())
    }

    /// Send a chat question to the running LLM engine and return the response.
    ///
    /// Calls the OpenAI-compatible `/v1/chat/completions` endpoint on the local
    /// engine port.  Returns an error string when the engine is not running or
    /// the HTTP call fails.
    pub fn chat(&self, question: &str, context: &str) -> Result<String, String> {
        let api_url = self
            .snapshot()
            .api_url()
            .ok_or_else(|| "AI engine is not running".to_string())?;

        let prompt = if context.is_empty() {
            question.to_owned()
        } else {
            format!("Context: {context}\n\nQuestion: {question}")
        };

        let body = serde_json::json!({
            "model": "local",
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 512
        });

        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(format!("{api_url}/chat/completions"))
            .json(&body)
            .send()
            .map_err(|e| format!("AI request failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("AI error: {e}"))?;

        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("AI response parse error: {e}"))?;

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "Unexpected AI response format".to_string())
    }

    fn model_from_id(id: &str) -> Result<LlmModel, String> {
        match id {
            "qwen3-4b" => Ok(LlmModel::Qwen3_4B),
            "qwen3-8b" => Ok(LlmModel::Qwen3_8B),
            "qwen2.5-coder-7b" => Ok(LlmModel::Qwen25Coder7B),
            other => Err(format!("unknown model: {other}")),
        }
    }
}

impl Default for AiController {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_controller_is_stopped() {
        let ctrl = AiController::new();
        let snap = ctrl.snapshot();
        assert!(!snap.running);
    }

    #[test]
    fn list_models_returns_entries() {
        let ctrl = AiController::new();
        assert!(!ctrl.list_models().is_empty());
    }

    #[test]
    fn unknown_model_returns_error() {
        let ctrl = AiController::new();
        let result = ctrl.start("nonexistent-model");
        assert!(result.is_err());
    }
}
