---
phase: 08-resurrectable-session-improvements
plan: 01
subsystem: ui
tags: [zellij, sessions, keybindings, toggle]

# Dependency graph
requires:
  - phase: 07-session-rename
    provides: session management UI patterns
provides:
  - Alt+D toggle for resurrectable sessions visibility
  - Pink color (index 3) for dead session visual distinction
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Runtime toggle state over config option for ephemeral visibility settings

key-files:
  created: []
  modified:
    - src/state.rs
    - src/ui/renderer.rs
    - src/config.rs

key-decisions:
  - "Runtime toggle over config option for dead sessions visibility"

patterns-established:
  - "Use runtime state for ephemeral UI toggles rather than config file options"

issues-created: []

# Metrics
duration: 5 min
completed: 2026-01-14
---

# Phase 8 Plan 1: Alt+D Toggle for Resurrectable Sessions Summary

**Alt+D runtime toggle replaces config option for dead sessions with pink visual distinction**

## Performance

- **Duration:** 5 min
- **Started:** 2026-01-14T17:30:00Z
- **Completed:** 2026-01-14T17:35:00Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Added Alt+D keybinding to toggle resurrectable session visibility
- Changed resurrectable session color from cyan (index 1) to pink (index 3) for visual distinction
- Removed `show_resurrectable_sessions` config option - feature is now purely runtime-toggled

## Task Commits

Each task was committed atomically:

1. **Task 1: Add show_dead_sessions state and Alt+D toggle** - `4b7c212` (feat)
2. **Task 2: Update resurrectable session color to pink and add help text** - `4af31c6` (feat)
3. **Task 3: Remove show_resurrectable_sessions config option** - `e7d9e3a` (refactor)

## Files Created/Modified
- `src/state.rs` - Added show_dead_sessions field, Alt+D handler, updated combined_items()
- `src/ui/renderer.rs` - Changed resurrectable session color to pink, added Alt+d to help text
- `src/config.rs` - Removed show_resurrectable_sessions field and parsing logic

## Decisions Made
- Runtime toggle over config option: Dead sessions are ephemeral UI state, not persistent config

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## Next Phase Readiness
- Alt+D toggle working, dead sessions hidden by default
- Pink color provides clear visual distinction when shown
- Ready for any additional resurrectable session improvements

---
*Phase: 08-resurrectable-session-improvements*
*Completed: 2026-01-14*
