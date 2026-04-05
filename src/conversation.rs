// conversation.rs — ConversationStore trait (Strategy Pattern).
//
// Design Pattern: Strategy — swap persistence backends for chat history.
//
//   ConversationStore (trait)
//     ├── InMemoryConversationStore  — ephemeral; default + tests
//     └── TomlConversationStore      — persists to ~/.config/fsn/ai-history.toml

use serde::{Deserialize, Serialize};

// ── Domain types ──────────────────────────────────────────────────────────────

/// Speaker role of a conversation turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TurnRole {
    User,
    Assistant,
}

impl TurnRole {
    pub fn label(&self) -> &'static str {
        match self {
            Self::User => "You",
            Self::Assistant => "AI",
        }
    }
}

/// One exchange in the conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: TurnRole,
    pub content: String,
    /// ISO 8601 timestamp (UTC).
    pub timestamp: String,
}

impl ConversationTurn {
    /// Create a new turn stamped with the current UTC time.
    #[must_use]
    pub fn now(role: TurnRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Persisted container for the conversation history.
#[derive(Default, Serialize, Deserialize)]
struct ConversationHistory {
    #[serde(default)]
    turns: Vec<ConversationTurn>,
}

impl ConversationHistory {
    fn path() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        std::path::PathBuf::from(home)
            .join(".config")
            .join("fsn")
            .join("ai-history.toml")
    }

    fn load() -> Self {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str::<Self>(&content).unwrap_or_default()
    }

    fn save(&self) -> Result<(), String> {
        let path = Self::path();
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}

// ── ConversationStore (Strategy) ──────────────────────────────────────────────

/// Pluggable persistence strategy for conversation history.
pub trait ConversationStore: Send + Sync {
    /// Append a new turn to the history.
    fn append(&self, turn: ConversationTurn);

    /// Return the full conversation history in order.
    fn history(&self) -> Vec<ConversationTurn>;

    /// Clear all turns.
    fn clear(&self);

    /// Number of turns stored.
    fn len(&self) -> usize {
        self.history().len()
    }

    /// Returns `true` when no turns have been recorded yet.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ── InMemoryConversationStore ─────────────────────────────────────────────────

/// Ephemeral in-memory store.  State is lost when the process exits.
#[derive(Default)]
pub struct InMemoryConversationStore {
    turns: std::sync::Mutex<Vec<ConversationTurn>>,
}

impl InMemoryConversationStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConversationStore for InMemoryConversationStore {
    fn append(&self, turn: ConversationTurn) {
        self.turns.lock().unwrap().push(turn);
    }

    fn history(&self) -> Vec<ConversationTurn> {
        self.turns.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.turns.lock().unwrap().clear();
    }
}

// ── TomlConversationStore ─────────────────────────────────────────────────────

/// Durable store that persists to `~/.config/fsn/ai-history.toml`.
///
/// The file is read once at construction; every mutation flushes to disk.
pub struct TomlConversationStore {
    turns: std::sync::Mutex<Vec<ConversationTurn>>,
}

impl TomlConversationStore {
    /// Load history from disk.  Missing or invalid files start with an empty history.
    #[must_use]
    pub fn new() -> Self {
        let turns = ConversationHistory::load().turns;
        Self {
            turns: std::sync::Mutex::new(turns),
        }
    }

    fn flush(guard: &[ConversationTurn]) {
        let history = ConversationHistory {
            turns: guard.to_vec(),
        };
        if let Err(e) = history.save() {
            tracing::warn!("ConversationStore: failed to persist history: {e}");
        }
    }
}

impl Default for TomlConversationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationStore for TomlConversationStore {
    fn append(&self, turn: ConversationTurn) {
        let mut guard = self.turns.lock().unwrap();
        guard.push(turn);
        Self::flush(&guard);
    }

    fn history(&self) -> Vec<ConversationTurn> {
        self.turns.lock().unwrap().clone()
    }

    fn clear(&self) {
        let mut guard = self.turns.lock().unwrap();
        guard.clear();
        Self::flush(&guard);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> InMemoryConversationStore {
        InMemoryConversationStore::new()
    }

    #[test]
    fn starts_empty() {
        assert!(store().is_empty());
    }

    #[test]
    fn append_and_history() {
        let s = store();
        s.append(ConversationTurn::now(TurnRole::User, "Hello"));
        assert_eq!(s.len(), 1);
        assert_eq!(s.history()[0].role, TurnRole::User);
    }

    #[test]
    fn multiple_turns_preserved_in_order() {
        let s = store();
        s.append(ConversationTurn::now(TurnRole::User, "Question"));
        s.append(ConversationTurn::now(TurnRole::Assistant, "Answer"));
        let h = s.history();
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].role, TurnRole::User);
        assert_eq!(h[1].role, TurnRole::Assistant);
    }

    #[test]
    fn clear_empties_history() {
        let s = store();
        s.append(ConversationTurn::now(TurnRole::User, "Foo"));
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn turn_role_labels() {
        assert_eq!(TurnRole::User.label(), "You");
        assert_eq!(TurnRole::Assistant.label(), "AI");
    }
}
