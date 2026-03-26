# CLAUDE.md – fs-ai

## What is this?

FreeSynergy AI — local LLM engine manager (Mistral.rs backend + Dioxus UI).

## Workspace

- `crates/fs-manager-ai/` — backend: engine lifecycle, model catalogue
- `crates/fs-ai/` — UI: Dioxus desktop app

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md
- After every feature: commit directly

## Quality Gates (before every commit)

```
1. Design Pattern (Traits, Object hierarchy)
2. Structs + Traits — no impl code yet
3. cargo check
4. Impl (OOP)
5. cargo clippy --all-targets -- -D warnings
6. cargo fmt --check
7. Unit tests (min. 1 per public module)
8. cargo test
9. commit + push
```

Every lib.rs / main.rs must have:
```rust
#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings)]
```

## Architecture

- `AiEngine` trait — common interface for all engine types
- `LlmEngine` — Mistral.rs backend (PID-file process management)
- `ModelSpec` catalogue — data-driven, single source of truth
- `AiManagerApp` — Dioxus root component

## Dependencies

- `fs-i18n` from `../fs-libs/`
- `dioxus-desktop` patched from `../fs-desktop/vendor/dioxus-desktop`
