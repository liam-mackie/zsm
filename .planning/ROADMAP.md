# Roadmap: ZSM Usability Improvements

## Overview

Polish ZSM's visual presentation and interaction patterns across six focused phases — from color refinement through responsive feedback — making the session selector feel instant and intuitive.

## Domain Expertise

None

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [x] **Phase 1: Color Palette** - Replace harsh pink/magenta with muted colors for live sessions
- [x] **Phase 2: MRU Ordering** - Sessions ordered by most recent switch with config option
- [x] **Phase 3: Tabular Alignment** - Session names and directories in clean columns
- [x] **Phase 4: Title Update** - Change title to "ZSM - Zellij Session Manager"
- [x] **Phase 5: Responsive Deletion** - Immediate UI feedback when killing sessions
- [x] **Phase 6: Help Text Layout** - Two-row help text with logical grouping
- [x] **Phase 7: Fix Session Naming Bug** - Fix session naming bug
- [x] **Phase 8: Resurrectable Session Improvements** - Improve resurrectable session handling and display
- [x] **Phase 9: Fix Session Name Editing** - Fix session name editing during session creation
- [x] **Phase 10: Help Text Phase 2** - Second iteration of help text improvements

## Phase Details

### Phase 1: Color Palette
**Goal**: Replace harsh pink/magenta bullet indicators with subtle, muted colors that complement the cyan session names
**Depends on**: Nothing (first phase)
**Research**: Unlikely (internal UI theming in src/ui/theme.rs and src/ui/components.rs)
**Plans**: TBD

Plans:
- [x] 01-01: Update theme colors for live session indicators

### Phase 2: MRU Ordering
**Goal**: Sort sessions by most recent switch rather than alphabetically, with a config option to choose sort order
**Depends on**: Phase 1
**Research**: Unlikely (state management in src/state.rs, config in src/config.rs)
**Plans**: TBD

Plans:
- [x] 02-01: Add sort_order config option
- [x] 02-02: Implement MRU tracking and ordering

### Phase 3: Tabular Alignment
**Goal**: Align session names and directory paths in clean columns for improved readability
**Depends on**: Phase 2
**Research**: Unlikely (renderer changes in src/ui/renderer.rs)
**Plans**: TBD

Plans:
- [x] 03-01: Implement columnar layout for session list

### Phase 4: Title Update
**Goal**: Change plugin title from current to "ZSM - Zellij Session Manager"
**Depends on**: Phase 3
**Research**: Unlikely (simple string change in renderer)
**Plans**: TBD

Plans:
- [x] 04-01: Update title string in renderer

### Phase 5: Responsive Deletion
**Goal**: Provide immediate visual feedback when deleting sessions to reduce perceived lag
**Depends on**: Phase 4
**Research**: Unlikely (session manager patterns in src/session/manager.rs)
**Plans**: TBD

Plans:
- [x] 05-01: Add optimistic UI update on delete action

### Phase 6: Help Text Layout
**Goal**: Reorganize help text into two rows with navigation keys on row 1, action keys on row 2
**Depends on**: Phase 5
**Research**: Unlikely (renderer component layout)
**Plans**: TBD

Plans:
- [x] 06-01: Restructure help text rendering

### Phase 7: Fix Session Naming Bug
**Goal**: Add session rename capability so users can rename sessions with unwanted `.2` suffix
**Depends on**: Phase 6
**Research**: None (using existing zellij-tile `rename_session()` API)
**Plans**: 1

Plans:
- [x] 07-01: Add session rename with Ctrl+n keybinding

### Phase 8: Resurrectable Session Improvements
**Goal**: Improve resurrectable session handling and display
**Depends on**: Phase 7
**Research**: None (internal state and rendering changes)
**Plans**: 1

Plans:
- [x] 08-01: Add Alt+D toggle for resurrectable sessions with pink coloring

### Phase 9: Fix Session Name Editing
**Goal**: Fix session name editing during session creation so users can modify the auto-generated name
**Depends on**: Phase 8
**Research**: Complete (Enter from name entry returns to Main instead of showing layout selection)
**Plans**: 1

Plans:
- [x] 09-01: Fix NewSession Enter key after name editing

### Phase 10: Help Text Phase 2
**Goal**: Two-color help text styling where keys are pink/magenta and labels are default foreground
**Depends on**: Phase 9
**Research**: None (internal UI styling)
**Plans**: 1

Plans:
- [x] 10-01: Two-color help text styling

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Color Palette | 1/1 | Complete | 2026-01-14 |
| 2. MRU Ordering | 2/2 | Complete | 2026-01-13 |
| 3. Tabular Alignment | 1/1 | Complete | 2026-01-13 |
| 4. Title Update | 1/1 | Complete | 2026-01-13 |
| 5. Responsive Deletion | 1/1 | Complete | 2026-01-13 |
| 6. Help Text Layout | 1/1 | Complete | 2026-01-13 |
| 7. Fix Session Naming Bug | 1/1 | Complete | 2026-01-13 |
| 8. Resurrectable Session Improvements | 1/1 | Complete | 2026-01-14 |
| 9. Fix Session Name Editing | 1/1 | Complete | 2026-01-15 |
| 10. Help Text Phase 2 | 1/1 | Complete | 2026-01-16 |
