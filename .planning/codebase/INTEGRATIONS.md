# External Integrations

**Analysis Date:** 2026-01-13

## APIs & External Services

**No Remote API Services:**
- The plugin does not make HTTP requests or communicate with external cloud services
- All operations are local within the Zellij plugin sandbox

## CLI Tool Integrations

**Zoxide (Smart Directory Navigation):**
- External command execution: `zoxide query -l -s` (`src/main.rs:158`)
- Parses zoxide output format: `"<score> <path>"` (`src/main.rs:169-180`)
- Requires zoxide to be installed and used for directory history
- Failure handling with error messaging (`src/main.rs:79-86`)

## Zellij Plugin System Integration

**Permissions Requested:**
- `PermissionType::RunCommands` - Execute shell commands (zoxide) (`src/main.rs:21`)
- `PermissionType::ReadApplicationState` - Read current sessions/layouts (`src/main.rs:22`)
- `PermissionType::ChangeApplicationState` - Create and switch sessions (`src/main.rs:23`)
- `PermissionType::MessageAndLaunchOtherPlugins` - Launch filepicker plugin (`src/main.rs:24`)

**Event Subscriptions:**
- `EventType::ModeUpdate` - Theme/color updates
- `EventType::SessionUpdate` - Session changes
- `EventType::Key` - Keyboard input
- `EventType::RunCommandResult` - Command execution results
- `EventType::PermissionRequestResult` - Permission grant/deny responses

**Plugin Communication:**
- Filepicker plugin integration via `pipe_message` for folder selection (`src/main.rs:102-147`)
- Request ID tracking with UUID validation for plugin-to-plugin communication (`src/state.rs`)

## Data Storage

**Databases:**
- None - No database connections

**File Storage:**
- Sandboxed within WASI - Direct filesystem operations use sandboxed paths
- `/host` - Working directory of focused terminal (read-only)
- `/data` - Plugin-specific folder
- `/tmp` - Sandboxed temporary directory (NOT real `/tmp`)

**Caching:**
- None - No caching layer

## Quick-Switch Feature Storage

**Previous Session Tracking:**
- Writes current session name to `/tmp/zsm-previous-session` via `run_command` (`src/state.rs:675`)
- Reads previous session on plugin open for instant toggle (`src/main.rs:87-94`)
- Async file operations due to WASM sandbox limitations
- World-writable location, shared across sessions

## Authentication & Identity

**Auth Provider:**
- None - No authentication required

**OAuth Integrations:**
- None

## Monitoring & Observability

**Error Tracking:**
- None - No external error tracking service

**Analytics:**
- None - No telemetry or analytics

**Logs:**
- Plugin logs to Zellij's plugin log output
- No external logging service

## CI/CD & Deployment

**Hosting:**
- GitHub Releases for binary distribution
- Installed to `~/.config/zellij/plugins/` by users

**CI Pipeline:**
- GitHub Actions (`.github/workflows/ci.yml`)
- Jobs: test, build-wasm, clippy, fmt
- release-please automation for versioning (`.github/workflows/release-please.yml`)
- Automated WASM binary build and upload to GitHub Releases

## Environment Configuration

**Development:**
- No environment variables required
- Configuration via Zellij layout KDL files (`plugin.kdl`, `zellij.kdl`)
- zoxide must be installed for directory listing

**Staging:**
- Not applicable (local plugin)

**Production:**
- No secrets management required
- Plugin runs with user's local permissions

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

---

*Integration audit: 2026-01-13*
*Update when adding/removing external services*
