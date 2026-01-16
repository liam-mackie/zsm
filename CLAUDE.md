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

- **`state.rs`** - `PluginState` struct holds all plugin state: config, session manager, zoxide directories, search engine, and UI state. Orchestrates key handling between screens (Main, NewSession, Rename).

- **`config.rs`** - Plugin configuration parsed from Zellij layout options (default_layout, session_separator, show_resurrectable_sessions, show_all_sessions, base_paths, sort_order).

### Session Module (`session/`)

- **`manager.rs`** - `SessionManager` handles session operations: tracking existing/resurrectable sessions, generating incremented names (e.g., `project.2`), and executing actions (switch/delete sessions).

- **`types.rs`** - `SessionItem` enum (ExistingSession, ResurrectableSession, Directory) and `SessionAction` enum for session operations.

### Zoxide Module (`zoxide/`)

- **`directory.rs`** - `ZoxideDirectory` struct: ranking score, directory path, generated session name.

- **`search.rs`** - `SearchEngine` for fuzzy-finding directories/sessions using `fuzzy-matcher` crate.

### UI Module (`ui/`)

- **`renderer.rs`** - `PluginRenderer` handles all terminal rendering. Renders main list, new session screen, search results, and deletion confirmations.

- **`components.rs`** - UI color utilities.

- **`theme.rs`** - Theme/palette handling. See "Color System" below.

### Other

- **`new_session_info.rs`** - State for new session creation screen (name input, folder selection, layout selection).

## Color System

Zellij plugins use `Text::color_range(index, range)` where `index` (0-3) maps to the user's theme **emphasis colors**:

| Index | Color (default theme) | Usage in ZSM |
|-------|----------------------|--------------|
| 0 | Orange | Warnings, errors |
| 1 | Cyan | Available sessions |
| 2 | Green | Current/active session |
| 3 | Pink/Magenta | Highlights |

**Limitations:**
- Only 4 colors available (emphasis_0 through emphasis_3)
- No "dim" or opacity options — use default foreground for muted text
- Actual colors depend on user's Zellij theme

## Key Concepts

**Smart Session Naming**: The plugin generates session names from directory paths, handling conflicts (adds parent context), nested directories, and truncation (max 29 chars due to Unix socket path limits).

**Three-Screen UI**: Main screen shows directory list with fuzzy search; NewSession screen handles session name/folder/layout configuration; Rename screen allows renaming the current session.

**Filepicker Integration**: Communicates with Zellij's filepicker plugin via `pipe_message_to_plugin` for folder selection.

**Session Stability**: Zellij sends inconsistent `SessionUpdate` events that can omit sessions temporarily. The `SessionManager` uses stability tracking with a missing-count threshold (3 updates) before removing sessions from the UI, preventing flickering.

**MRU Ordering**: Sessions sorted by most recent switch. Timestamps persisted to `/tmp/zsm-mru-timestamps` via `run_command`. Configurable via `sort_order` option (`mru` or `alphabetical`).

**Optimistic UI Updates**: Delete and rename operations update local state immediately before sending commands to Zellij, providing instant feedback. If the operation fails, the session reappears on the next `SessionUpdate`.

## Keybindings

**Main Screen:**
| Key | Action |
|-----|--------|
| `↑/↓` | Navigate list |
| `Enter` | Switch to session / Create new |
| `Ctrl+Enter` | Quick create with default layout |
| `Alt+r` | Rename current session |
| `Ctrl+r` | Reload zoxide directories |
| `Del` | Delete selected session |
| `Esc` | Exit (or clear search) |
| Type | Fuzzy search |

**Rename Screen:**
| Key | Action |
|-----|--------|
| `Enter` | Confirm rename |
| `Esc` | Cancel |

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
- `test` - Runs unit tests on native target (must explicitly specify `--target x86_64-unknown-linux-gnu` because `.cargo/config.toml` defaults to wasm32-wasip1)
- `build-wasm` - Verifies WASM compilation
- `clippy` - Lints with `-D warnings`
- `fmt` - Checks formatting

## WASM Sandbox Limitations

Zellij plugins run in a WASI sandbox with restricted filesystem access:

- **Cannot directly write to real filesystem** - `std::fs::write()` writes to a sandboxed virtual filesystem
- **Mapped paths** (per [Zellij docs](https://zellij.dev/documentation/plugin-api-file-system.html)):
  - `/host` - Working directory of last focused terminal
  - `/data` - Plugin-specific folder (but NOT shared across sessions despite docs)
  - `/tmp` - Sandboxed temp directory (NOT the real /tmp)

**To persist data across sessions**, use `run_command` to shell out:
```rust
run_command(&["sh", "-c", "echo 'data' > /tmp/myfile"], context);
```

## Plugin Instance Behavior

**Each Zellij session has its own plugin instance** with separate state:
- Switching from session A to B means interacting with B's plugin instance
- State does not transfer between sessions automatically
- To share state, must use external storage (files via `run_command`)

**Plugin reloading**:
- `zellij action start-or-reload-plugin "file:/path/to/plugin.wasm"` - Reloads in current session only
- Must reload separately in each session, or close/reopen the plugin pane
- Closing the pane (not just hiding) and reopening loads the new binary

## Quick-Switch Feature

The plugin supports quick-switching to the previous session:
- When switching sessions, writes current session name to `/tmp/zsm-previous-session` via `run_command`
- When plugin opens, reads that file and pre-selects the previous session
- Press Enter to instantly toggle back

Implementation in `state.rs`: `write_previous_session()` and `request_previous_session_read()` use async `run_command` because direct filesystem access is sandboxed.
