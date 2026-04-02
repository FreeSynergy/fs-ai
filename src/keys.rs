// keys.rs — FTL key name constants for fs-ai.
//
// All user-visible strings are translated via fs-i18n.
// The matching .ftl files live at:
//   fs-i18n/locales/{lang}/ai.ftl
//
// Use these constants wherever a localised string is needed.

// ── App ───────────────────────────────────────────────────────────────────────

pub const TITLE: &str = "ai-title";
pub const ENGINES_TITLE: &str = "ai-engines-title";
pub const MODEL_LABEL: &str = "ai-model-label";

// ── Engine status ─────────────────────────────────────────────────────────────

pub const STATUS_STOPPED: &str = "ai-status-stopped";
pub const STATUS_STARTING: &str = "ai-mistral-status-starting";

// ── Buttons ───────────────────────────────────────────────────────────────────

pub const BTN_START: &str = "ai-start-engine";
pub const BTN_STOP: &str = "ai-stop-engine";
pub const BTN_REFRESH: &str = "ai-mistral-btn-refresh";

// ── Mistral ───────────────────────────────────────────────────────────────────

pub const MISTRAL_TITLE: &str = "ai-mistral-title";
pub const MISTRAL_DESCRIPTION: &str = "ai-mistral-description";
pub const MISTRAL_STATUS_RUNNING: &str = "ai-mistral-status-running";
pub const MISTRAL_STATUS_STOPPED: &str = "ai-mistral-status-stopped";
pub const MISTRAL_RAM_INFO: &str = "ai-mistral-ram-info";
pub const MISTRAL_BINARY_MISSING: &str = "ai-mistral-binary-missing";

// ── Editor integration ────────────────────────────────────────────────────────

pub const EDITOR_INTEGRATION_TITLE: &str = "ai-editor-integration-title";

// ── Errors ────────────────────────────────────────────────────────────────────

pub const ERROR: &str = "ai-error";
pub const ERROR_DETAIL: &str = "ai-error-detail";
