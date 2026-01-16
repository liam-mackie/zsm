# Architecture

**Analysis Date:** 2026-01-13

## Pattern Overview

**Overall:** Plugin-based architecture with layered state management and event-driven processing

**Key Characteristics:**
- Zellij plugin using WASM (WebAssembly) with WASI sandbox
- Event-driven: Zellij sends events → Plugin processes → State updates → UI renders
- Single executable with two-screen UI (Main list, NewSession creation)
- Stateless request handling within plugin instance

## Layers

**Plugin Layer:**
- Purpose: Zellij plugin trait implementation, event dispatcher
- Contains: `ZellijPlugin` trait methods (`load`, `update`, `pipe`, `render`)
- Location: `src/main.rs`
- Depends on: State layer for coordination
- Used by: Zellij runtime

**State Layer:**
- Purpose: Central state management, business logic coordination
- Contains: `PluginState` struct, key handlers, item combination logic
- Location: `src/state.rs`
- Depends on: Config, Session, Zoxide, NewSessionInfo, UI modules
- Used by: Plugin layer

**Session Layer:**
- Purpose: Zellij session operations, lifecycle management, stability tracking
- Contains: `SessionManager`, `SessionItem`, `SessionAction`
- Location: `src/session/manager.rs`, `src/session/types.rs`
- Depends on: No internal dependencies (isolated)
- Used by: State layer

**Zoxide Layer:**
- Purpose: Directory discovery and fuzzy search
- Contains: `ZoxideDirectory`, `SearchEngine`
- Location: `src/zoxide/directory.rs`, `src/zoxide/search.rs`
- Depends on: Session types (for `SessionItem`)
- Used by: State layer

**UI Layer:**
- Purpose: Terminal rendering
- Contains: `PluginRenderer`, `Theme`, `Colors`, components
- Location: `src/ui/renderer.rs`, `src/ui/theme.rs`, `src/ui/components.rs`
- Depends on: Session types, State for display data
- Used by: Plugin layer (render method)

**Configuration Layer:**
- Purpose: Zellij layout-based configuration parsing
- Contains: `Config` struct
- Location: `src/config.rs`
- Depends on: None
- Used by: State layer

## Data Flow

**Plugin Initialization:**

1. `ZellijPlugin::load()` in `src/main.rs:16` → Registers plugin with Zellij
2. Requests permissions (RunCommands, ReadApplicationState, ChangeApplicationState, MessageAndLaunchOtherPlugins)
3. Subscribes to events (ModeUpdate, SessionUpdate, Key, RunCommandResult, PermissionRequestResult)

**Permission Grant & Zoxide Query:**

1. User receives PermissionRequestResult → `update()` in `src/main.rs:50`
2. `fetch_zoxide_directories()` runs `zoxide query -l -s` command
3. Results returned via RunCommandResult event

**Zoxide Output Processing:**

1. `process_zoxide_output()` in `src/main.rs:161` parses score+path lines
2. `generate_smart_session_names()` creates context-aware names (`src/main.rs:197-235`)
3. Names conflict-resolved with parent context and truncated to 29 chars
4. `update_zoxide_directories()` in `src/state.rs:102` stores directories

**Session Updates:**

1. Zellij sends SessionUpdate event with current and resurrectable sessions
2. `update_sessions()` in `src/state.rs:57` uses stability tracking (MISSING_THRESHOLD=3)
3. Prevents UI flickering from Zellij's inconsistent event timing

**User Interaction:**

1. Key event in `update()` → `handle_key()` in `src/state.rs:108`
2. Routes to main screen or new session screen handlers
3. Actions trigger session switches, deletions, or new session creation

**State Management:**
- File-based state for previous session: `/tmp/zsm-previous-session`
- Each plugin instance has isolated state (per Zellij session)
- State does not transfer between sessions automatically

## Key Abstractions

**SessionManager:**
- Purpose: Orchestrates Zellij session operations with stability tracking
- Examples: `update_sessions_stable()`, `execute_action()`, `generate_incremented_name()`
- Location: `src/session/manager.rs`
- Pattern: Stability threshold (MISSING_THRESHOLD=3) prevents UI flicker from inconsistent Zellij events

**SessionItem:**
- Purpose: Represents displayable items in the session list
- Examples: `ExistingSession`, `ResurrectableSession`, `Directory`
- Location: `src/session/types.rs`
- Pattern: Enum with variant data

**SearchEngine:**
- Purpose: Fuzzy matching for search functionality
- Examples: `search()`, `update_search()`, `get_results()`
- Location: `src/zoxide/search.rs`
- Pattern: Uses `SkimMatcherV2` from `fuzzy-matcher` crate

**PluginState:**
- Purpose: Central orchestrator holding all plugin state
- Examples: `combined_items()`, `display_items()`, `handle_key()`
- Location: `src/state.rs`
- Pattern: Singleton state container, coordinates between all layers

## Entry Points

**Plugin Entry:**
- Location: `src/main.rs:13` - `register_plugin!(PluginState)` macro
- Triggers: Zellij loads plugin WASM binary
- Responsibilities: Register plugin with Zellij runtime

**ZellijPlugin Implementation:**
- Location: `src/main.rs:15-152`
- Triggers: Zellij events (permissions, keys, session updates)
- Responsibilities: Event dispatch, zoxide command execution, rendering

**Renderer Entry:**
- Location: `src/ui/renderer.rs:14`
- Triggers: Zellij render call
- Responsibilities: Determine active screen, render UI, display overlays

## Error Handling

**Strategy:** Uses `Option<T>` for nullable values, UI error display for user feedback

**Patterns:**
- Validation errors shown via `.set_error()` method (`src/state.rs:26`)
- Error cleared on next keypress
- Permission denial shows error, prevents zoxide fetch
- Invalid session names blocked with descriptive messages

## Cross-Cutting Concerns

**Logging:**
- Plugin logs to Zellij's plugin log output
- No external logging framework

**Validation:**
- Session name validation at creation time (`src/state.rs:614-622`)
- Max 108 bytes, no `/` characters
- Path validation for zoxide results

**Smart Session Naming:**
- Complex algorithm spanning `src/main.rs:197-502`
- Conflict detection, context-aware naming, truncation to 29 chars
- Respects Unix socket path limits

**WASM Sandbox Handling:**
- Direct filesystem writes use sandboxed paths
- Shelling out via `run_command` for persistent file operations
- Inter-plugin communication via `pipe_message_to_plugin`

---

*Architecture analysis: 2026-01-13*
*Update when major patterns change*
