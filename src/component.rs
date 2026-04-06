// component.rs — AiComponent: chat interface for the AI assistant.
//
// Design Pattern: Facade (AiController wraps fs-manager-ai)
//   AiComponent renders a chat input + conversation history.
//   It only appears when the "ai.chat" capability is registered in fs-registry.
//
// Capability check: ComponentCtx.config["ai_chat_available"] == "true"
//   Missing or "false" → capability-unavailable notice is shown instead.
//
// Data source: AiController (via gRPC in production, stub here).
// Writes: Bus-Events ("ai.chat.send" topic).

use fs_render::component::{ButtonStyle, ComponentCtx, ComponentTrait, LayoutElement, TextSize};
use fs_render::layout::SlotKind;

use crate::keys;

// ── ConversationEntry stub ────────────────────────────────────────────────────

/// A single conversation turn shown in the component.
#[derive(Debug, Clone)]
struct ConversationEntry {
    /// `true` = user, `false` = assistant.
    from_user: bool,
    text: String,
}

impl ConversationEntry {
    fn row(&self) -> LayoutElement {
        let role_icon = if self.from_user {
            "fs:icons/user"
        } else {
            "fs:icons/ai"
        };
        LayoutElement::Row {
            children: vec![
                LayoutElement::Icon {
                    name: role_icon.into(),
                    size: 16,
                },
                LayoutElement::Text {
                    content: self.text.clone(),
                    size: TextSize::Body,
                    color: None,
                },
            ],
            gap: 6,
        }
    }
}

// ── AiComponent ───────────────────────────────────────────────────────────────

/// Chat interface for the AI assistant.
///
/// # Wiring (ComponentCtx.config)
///
/// | key | value |
/// |-----|-------|
/// | `"ai_chat_available"` | `"true"` when the `"ai.chat"` capability is registered |
/// | `"ai_busy"` | `"true"` while the engine is processing a request |
/// | `"conversation"` | JSON-serialised `Vec<{from_user:bool, text:String}>` (optional) |
///
/// In production the shell populates these via `fs-registry` gRPC queries.
/// The component is hidden (no render) when `ai_chat_available != "true"`.
pub struct AiComponent {
    id: &'static str,
}

impl AiComponent {
    /// Create a new AI chat component.
    pub fn new() -> Self {
        Self { id: "ai-chat" }
    }

    fn capability_unavailable() -> Vec<LayoutElement> {
        vec![LayoutElement::Text {
            content: "ai-chat-capability-unavailable".into(),
            size: TextSize::Body,
            color: None,
        }]
    }

    fn stub_conversation() -> Vec<ConversationEntry> {
        // Placeholder conversation — replaced by live data from AiController.
        vec![ConversationEntry {
            from_user: false,
            text: "ai-chat-welcome".into(),
        }]
    }
}

impl Default for AiComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentTrait for AiComponent {
    fn component_id(&self) -> &str {
        self.id
    }

    fn name_key(&self) -> &'static str {
        keys::TITLE
    }

    fn description_key(&self) -> &'static str {
        "ai-chat-desc"
    }

    fn slot_preference(&self) -> SlotKind {
        SlotKind::Sidebar
    }

    fn min_width(&self) -> u32 {
        220
    }

    fn render(&self, ctx: &ComponentCtx) -> Vec<LayoutElement> {
        let available = ctx.config.get("ai_chat_available").map(String::as_str) == Some("true");

        if !available {
            return Self::capability_unavailable();
        }

        let busy = ctx.config.get("ai_busy").map(String::as_str) == Some("true");

        let conversation = Self::stub_conversation();

        let mut elements = vec![
            LayoutElement::Text {
                content: keys::TITLE.into(),
                size: TextSize::Label,
                color: None,
            },
            LayoutElement::Separator { label_key: None },
        ];

        // Conversation history
        let history_items: Vec<LayoutElement> =
            conversation.iter().map(ConversationEntry::row).collect();
        elements.push(LayoutElement::List {
            items: history_items,
            scrollable: true,
        });

        elements.push(LayoutElement::Separator { label_key: None });

        // Input area
        if busy {
            elements.push(LayoutElement::Spinner);
        } else {
            elements.push(LayoutElement::TextInput {
                placeholder_key: "ai-chat-input-placeholder".into(),
                value: String::new(),
                on_change_action: "ai.chat.input_changed".into(),
            });
            elements.push(LayoutElement::Button {
                label_key: "ai-chat-send".into(),
                action: "ai.chat.send".into(),
                style: ButtonStyle::Primary,
            });
        }

        elements
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fs_render::layout::{ShellKind, SlotKind};

    fn ctx_available(busy: bool) -> ComponentCtx {
        let mut ctx = ComponentCtx::test(ShellKind::Sidebar, SlotKind::Sidebar);
        ctx.config.insert("ai_chat_available".into(), "true".into());
        ctx.config
            .insert("ai_busy".into(), if busy { "true" } else { "false" }.into());
        ctx
    }

    #[test]
    fn component_id() {
        let c = AiComponent::new();
        assert_eq!(c.component_id(), "ai-chat");
    }

    #[test]
    fn slot_preference_is_sidebar() {
        let c = AiComponent::new();
        assert_eq!(c.slot_preference(), SlotKind::Sidebar);
    }

    #[test]
    fn render_unavailable_when_no_capability() {
        let c = AiComponent::new();
        let ctx = ComponentCtx::test(ShellKind::Sidebar, SlotKind::Sidebar);
        let els = c.render(&ctx);
        assert_eq!(els.len(), 1);
        assert!(matches!(&els[0], LayoutElement::Text { content, .. }
            if content.contains("unavailable")));
    }

    #[test]
    fn render_available_shows_input() {
        let c = AiComponent::new();
        let ctx = ctx_available(false);
        let els = c.render(&ctx);
        let has_input = els
            .iter()
            .any(|e| matches!(e, LayoutElement::TextInput { .. }));
        assert!(has_input);
    }

    #[test]
    fn render_busy_shows_spinner_not_input() {
        let c = AiComponent::new();
        let ctx = ctx_available(true);
        let els = c.render(&ctx);
        let has_spinner = els.iter().any(|e| matches!(e, LayoutElement::Spinner));
        assert!(has_spinner);
        let has_input = els
            .iter()
            .any(|e| matches!(e, LayoutElement::TextInput { .. }));
        assert!(!has_input);
    }

    #[test]
    fn render_available_shows_conversation_list() {
        let c = AiComponent::new();
        let ctx = ctx_available(false);
        let els = c.render(&ctx);
        let has_list = els.iter().any(|e| matches!(e, LayoutElement::List { .. }));
        assert!(has_list);
    }
}
