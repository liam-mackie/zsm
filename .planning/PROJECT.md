# ZSM Usability Improvements

## What This Is

Usability polish for ZSM (Zellij Session Manager) — a Zellij plugin that integrates zoxide with session management. This round focuses on visual refinement and interaction improvements to make the session selector feel natural and pleasant.

## Core Value

The session selector should feel instant and intuitive — MRU ordering, clean visual hierarchy, and responsive feedback.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. Inferred from existing codebase. -->

- ✓ Session list with existing and resurrectable sessions — existing
- ✓ Zoxide directory integration with smart session naming — existing
- ✓ Fuzzy search across sessions and directories — existing
- ✓ New session creation with folder and layout selection — existing
- ✓ Session stability tracking (prevents UI flicker) — existing
- ✓ Quick-switch to previous session — existing
- ✓ Configuration via Zellij layout options — existing

### Active

<!-- Current scope. Building toward these. -->

- [ ] Pleasant color palette — replace harsh pink/magenta with subtle, muted colors for live sessions
- [ ] MRU ordering — sessions ordered by most recent switch, with config option (MRU vs alphabetic), default MRU
- [ ] Tabular directory alignment — session names and directories in clean columns
- [ ] Updated title — "ZSM - Zellij Session Manager"
- [ ] Responsive session deletion — immediate UI feedback when killing sessions (reduce lag)
- [ ] Two-row help text — navigation keys on row 1, action keys on row 2, grouped logically

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- New features — focus on polish, not new capabilities
- Settings screen — config stays in layout file, no in-plugin settings UI
- Keybinding changes — existing shortcuts must be preserved

## Context

ZSM is a WASM plugin running inside Zellij's sandbox. The UI renders via terminal escape sequences through zellij-tile. Current issues identified from screenshot review:

- Pink/magenta bullets (`○`) for live sessions clash with cyan session names
- Sessions sorted alphabetically, not by usage frequency
- Directory paths left-ragged after variable-length session names
- Help text at bottom wraps awkwardly with mixed key hints
- Session deletion has noticeable lag before list updates

Architecture (from codebase map):
- UI layer: `src/ui/renderer.rs`, `src/ui/theme.rs`, `src/ui/components.rs`
- State layer: `src/state.rs` — handles ordering and item combination
- Session layer: `src/session/manager.rs` — stability tracking affects deletion responsiveness
- Config layer: `src/config.rs` — will need new `sort_order` option

## Constraints

- **WASM sandbox**: Cannot write directly to filesystem; must shell out for persistence
- **29-char session names**: Unix socket path limit constrains display width
- **Zellij event timing**: SessionUpdate events can be inconsistent; existing stability tracking must be preserved

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| MRU as default sort | User expectation matches shell history behavior | — Pending |
| Config option for sort | Some users may prefer alphabetical | — Pending |
| Muted color palette | Reduce visual noise, improve readability | — Pending |

---
*Last updated: 2026-01-13 after initialization*
