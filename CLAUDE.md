# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ZSM (Zoxide Session Manager) is a Zellij plugin written in Rust that integrates zoxide (smart directory navigation) with Zellij session management. It compiles to WebAssembly and runs inside Zellij.

## Build Commands

```bash
# Build for development (debug mode)
cargo build --target wasm32-wasip1

# Build for release
cargo build --target wasm32-wasip1 --release

# Add WASM target if not installed
rustup target add wasm32-wasip1
```

Output locations:
- Debug: `target/wasm32-wasip1/debug/zsm.wasm`
- Release: `target/wasm32-wasip1/release/zsm.wasm`

## Development Workflow

Start the plugin development layout (includes hot-reload keybinding):
```bash
zellij -l zellij.kdl
```
- Press `Alt+R` to reload the plugin after rebuilding
- Re-launch plugin manually: `zellij action launch-or-focus-plugin file:target/wasm32-wasip1/debug/zsm.wasm`

Alternative with watchexec:
```bash
watchexec --exts rs -- 'cargo build --target wasm32-wasip1; zellij action start-or-reload-plugin file:target/wasm32-wasip1/debug/zsm.wasm'
```

## Architecture

### Core Modules

- **`main.rs`** - Plugin entry point. Implements `ZellijPlugin` trait, handles Zellij events (key input, permissions, session updates), and processes zoxide output. Contains smart session naming logic.

- **`state.rs`** - `PluginState` struct holds all plugin state: config, session manager, zoxide directories, search engine, and UI state. Orchestrates key handling between screens (Main vs NewSession).

- **`config.rs`** - Plugin configuration parsed from Zellij layout options (default_layout, session_separator, show_resurrectable_sessions, show_all_sessions, base_paths).

### Session Module (`session/`)

- **`manager.rs`** - `SessionManager` handles session operations: tracking existing/resurrectable sessions, generating incremented names (e.g., `project.2`), and executing actions (switch/delete sessions).

- **`types.rs`** - `SessionItem` enum (ExistingSession, ResurrectableSession, Directory) and `SessionAction` enum for session operations.

### Zoxide Module (`zoxide/`)

- **`directory.rs`** - `ZoxideDirectory` struct: ranking score, directory path, generated session name.

- **`search.rs`** - `SearchEngine` for fuzzy-finding directories/sessions using `fuzzy-matcher` crate.

### UI Module (`ui/`)

- **`renderer.rs`** - `PluginRenderer` handles all terminal rendering. Renders main list, new session screen, search results, and deletion confirmations.

- **`components.rs`** - UI color utilities.

- **`theme.rs`** - Theme/palette handling.

### Other

- **`new_session_info.rs`** - State for new session creation screen (name input, folder selection, layout selection).

## Key Concepts

**Smart Session Naming**: The plugin generates session names from directory paths, handling conflicts (adds parent context), nested directories, and truncation (max 29 chars due to Unix socket path limits).

**Two-Screen UI**: Main screen shows directory list with fuzzy search; NewSession screen handles session name/folder/layout configuration.

**Filepicker Integration**: Communicates with Zellij's filepicker plugin via `pipe_message_to_plugin` for folder selection.

**Session Stability**: Zellij sends inconsistent `SessionUpdate` events that can omit sessions temporarily. The `SessionManager` uses stability tracking with a missing-count threshold (3 updates) before removing sessions from the UI, preventing flickering.

## Testing

Tests must run on the **native target**, not WASM (WASM binaries can't execute directly):

```bash
# Run tests (uses native target automatically when not cross-compiling)
cargo test

# Or explicitly specify target
cargo test --target aarch64-apple-darwin
```

Note: `SessionInfo` from `zellij-tile` has many required fields. See `manager.rs` test helper `make_session()` for how to construct test instances.

## CI

GitHub Actions workflow (`.github/workflows/ci.yml`) runs on PRs and pushes to main:
- `test` - Runs unit tests on native target
- `build-wasm` - Verifies WASM compilation
- `clippy` - Lints with `-D warnings`
- `fmt` - Checks formatting
