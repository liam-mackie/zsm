---
phase: 02-mru-ordering
plan: 01
subsystem: config
tags: [rust, wasm, config, enum]

# Dependency graph
requires:
  - phase: 01-color-palette
    provides: Visual polish foundation
provides:
  - SortOrder enum with Mru/Alphabetical variants
  - Config.sort_order field parsed from Zellij layout
affects: [02-02-mru-tracking, state-ordering]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Enum with #[default] for config defaults"
    - "Case-insensitive config parsing with to_lowercase()"

key-files:
  created: []
  modified:
    - src/config.rs

key-decisions:
  - "MRU as default sort order - matches user expectation from shell history"
  - "Case-insensitive config value parsing - more forgiving for users"
  - "#[allow(dead_code)] on field until consumed in 02-02"

patterns-established:
  - "Enum-based config options with from_config_str() parser method"

issues-created: []

# Metrics
duration: 4min
completed: 2026-01-13
---

# Phase 2 Plan 01: Add sort_order Config Option Summary

**SortOrder enum with Mru/Alphabetical variants and Config.sort_order field parsed from Zellij layout config**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-13T18:20:00Z
- **Completed:** 2026-01-13T18:24:00Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments

- Created SortOrder enum with Mru (default) and Alphabetical variants
- Added sort_order field to Config struct
- Implemented case-insensitive config parsing via from_config_str() method
- Default sort order is MRU (most recently used)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add sort_order config option** - `af4d93b` (feat)

## Files Created/Modified

- `src/config.rs` - Added SortOrder enum and sort_order field with config parsing
- `src/ui/renderer.rs` - Formatting fix (cargo fmt) included in commit

## Decisions Made

- Used MRU as default sort order to match user expectation from shell history behavior
- Implemented case-insensitive config value parsing (user can write "MRU", "mru", "Mru", etc.)
- Added #[allow(dead_code)] annotation on sort_order field since it will be consumed in plan 02-02

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed formatting in unrelated file**
- **Found during:** Task 1 verification (cargo fmt --check)
- **Issue:** src/ui/renderer.rs had a method chain split across lines that cargo fmt wanted on one line
- **Fix:** Ran cargo fmt to fix all formatting issues
- **Files modified:** src/ui/renderer.rs
- **Verification:** cargo fmt --check passes
- **Committed in:** af4d93b (included in task commit)

---

**Total deviations:** 1 auto-fixed (1 blocking), 0 deferred
**Impact on plan:** Formatting fix was necessary for CI to pass. No scope creep.

## Issues Encountered

None

## Next Phase Readiness

- Config option is ready for consumption in 02-02-PLAN.md
- SortOrder enum is public and available for use in state.rs
- Next plan will implement MRU tracking and use sort_order to control ordering

---
*Phase: 02-mru-ordering*
*Completed: 2026-01-13*
