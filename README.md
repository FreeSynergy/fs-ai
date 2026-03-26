# fs-ai

FreeSynergy AI — local LLM engine manager.

Part of the [FreeSynergy](https://github.com/FreeSynergy) platform.

## Purpose

Manages local AI inference engines (currently: Mistral.rs). Lets users start/stop
LLM engines, select models, and automatically configures editor integrations
(Continue.dev).

## Workspace

| Crate | Purpose |
|---|---|
| `fs-manager-ai` | Backend: engine lifecycle, model catalogue, process management |
| `fs-ai` | UI: Dioxus desktop app |

## Architecture

- `AiEngine` trait — common interface for all engine types
- `LlmEngine` — Mistral.rs backend (PID-file based process management)
- `LlmModel` + `ModelSpec` catalogue — data-driven model registry
- `AiManagerApp` — Dioxus UI component

## Build

```bash
cargo build                   # default: desktop feature
```

## Dependencies

- **fs-libs** (`../fs-libs/`) — `fs-i18n`
- **fs-desktop** (`../fs-desktop/vendor/dioxus-desktop`) — patched Dioxus desktop
