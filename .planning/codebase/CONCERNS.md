# Codebase Concerns

**Analysis Date:** 2026-01-13

## Tech Debt

**Code Duplication - Range Calculation:**
- Issue: Duplicate range-to-render logic
- Location: `src/new_session_info.rs:271` - TODO comment: "merge with similar function in zoxide_directories"
- Why: Rapid development, similar patterns emerged independently
- Impact: Maintenance burden, risk of divergent behavior
- Fix approach: Extract shared utility function

**Complex Smart Naming Algorithm:**
- Issue: 300+ lines of interrelated functions for session name generation
- Location: `src/main.rs:197-502` - `generate_smart_session_names`, `generate_context_aware_name`, `normalize_path`, `apply_smart_truncation`, `abbreviate_segment`
- Why: Feature complexity (conflict detection, context-awareness, truncation limits)
- Impact: Difficult to test, maintain, and debug
- Fix approach: Extract to dedicated module, add comprehensive unit tests

## Known Bugs

**None identified during analysis.**

The codebase appears stable with no obvious bugs. Session stability tracking specifically addresses Zellij's inconsistent event behavior.

## Security Considerations

**Shell Command Injection (Medium Risk):**
- Risk: Session names interpolated into shell commands without escaping
- Location: `src/state.rs:675-686` (write_previous_session function)
- Code: `format!("echo '{}' > /tmp/zsm-previous-session", session_name)`
- Current mitigation: Session name validation blocks `/` characters (`src/state.rs:619-621`)
- Recommendations: Use shell-safe escaping, or write directly if WASM sandbox permits

**World-Writable Temp File:**
- Risk: `/tmp/zsm-previous-session` is world-readable/writable
- Location: `src/state.rs:675, 690`
- Current mitigation: File only contains session name (low sensitivity)
- Recommendations: Use user-specific path or XDG_RUNTIME_DIR

## Performance Bottlenecks

**None identified.**

The codebase handles small datasets (typically <100 directories/sessions). No performance issues expected at current scale.

**Potential Future Concern - Combined Items Rebuild:**
- Location: `src/state.rs:524-529` (update_search_if_needed)
- Pattern: `combined_items()` rebuilds full list on every search update
- Impact: Low with current dataset sizes
- Improvement path: Cache combined items, use incremental updates

## Fragile Areas

**Smart Session Naming Algorithm:**
- Location: `src/main.rs:197-502`
- Why fragile: Complex interdependent functions, many edge cases (Unicode, symlinks, deep paths)
- Common failures: Edge cases with unusual path structures
- Safe modification: Add comprehensive unit tests before changes
- Test coverage: **None** - HIGH RISK

**WASM Sandbox Limitations:**
- Location: `src/state.rs:675-702` (file I/O workarounds)
- Why fragile: Relies on shelling out to bypass WASM restrictions
- Common failures: File persistence issues, race conditions between plugin instances
- Safe modification: Understand WASM sandbox constraints before changes
- Test coverage: Cannot test in isolation (WASM-specific)

## Scaling Limits

**Not applicable.**

Plugin runs locally with user's directory history. No cloud services or shared resources.

## Dependencies at Risk

**All dependencies current (as of Jan 2026):**
- `zellij-tile 0.43.1` - Actively maintained with Zellij
- `zellij-utils 0.43.1` - Same lifecycle as zellij-tile
- `serde 1.0.164` - Stable, widely used
- `fuzzy-matcher 0.3.7` - Stable, minimal updates expected
- `uuid 1.8.0` - Stable, widely used
- `humantime 2.2.0` - Stable, low update frequency

No immediate risks identified.

## Missing Critical Features

**None identified.**

The plugin provides complete session/directory management functionality as designed.

## Test Coverage Gaps

**Smart Naming Algorithm (HIGH PRIORITY):**
- What's not tested: 503 lines of logic in `src/main.rs` - conflict detection, context naming, truncation
- Location: `src/main.rs:197-502`
- Risk: Naming bugs affect user experience, edge cases unknown
- Priority: HIGH
- Difficulty to test: Medium - need to mock or construct path data

**State Coordination Logic:**
- What's not tested: 709 lines in `src/state.rs` - display logic, selection, item combination
- Location: `src/state.rs`
- Risk: State management bugs could cause UI issues
- Priority: Medium
- Difficulty to test: Medium - need to simulate events

**Fuzzy Search Engine:**
- What's not tested: 204 lines in `src/zoxide/search.rs`
- Location: `src/zoxide/search.rs`
- Risk: Search ranking issues
- Priority: Low (uses well-tested fuzzy-matcher crate)
- Difficulty to test: Low - pure function logic

**UI Rendering:**
- What's not tested: 431 lines in `src/ui/renderer.rs`
- Location: `src/ui/renderer.rs`
- Risk: Display issues
- Priority: Low (visual verification during development)
- Difficulty to test: High - Zellij rendering API

## Minor Issues

**Error Messages Clear on Any Keypress:**
- Location: `src/state.rs:108-113`
- Behavior: Any keypress clears error, even if user didn't read it
- UX impact: Users might miss validation errors
- Recommendation: Only clear errors on meaningful actions

**Unicode Truncation Edge Case:**
- Location: `src/main.rs:343-344`
- Problem: `.len()` counts bytes, not characters; multibyte UTF-8 could truncate mid-character
- Impact: Low probability (requires non-ASCII paths)
- Recommendation: Use `.chars().count()` or check byte boundaries

**Configuration Delimiter Fragility:**
- Location: `src/config.rs:43-51`
- Issue: `base_paths` uses `|` delimiter with no escape mechanism
- Impact: Cannot use `|` in paths (edge case)
- Recommendation: Document limitation or add escape support

---

## Summary

**Overall Assessment:** Clean, well-structured codebase with no critical issues.

**Priorities:**
1. **HIGH**: Add unit tests for smart naming algorithm (`src/main.rs:197-502`)
2. **MEDIUM**: Address shell command injection concern in session persistence
3. **LOW**: Extract duplicate range calculation logic

**Positive Notes:**
- No `unsafe` blocks
- No unchecked `unwrap()` or `expect()` calls
- Good inline documentation
- CI/CD properly configured with tests, clippy, and fmt checks
- Error handling is defensive and informative

---

*Concerns audit: 2026-01-13*
*Update as issues are fixed or new ones discovered*
