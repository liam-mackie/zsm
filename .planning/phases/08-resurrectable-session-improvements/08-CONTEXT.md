# Phase 8: Resurrectable Session Improvements - Context

**Gathered:** 2026-01-14
**Status:** Ready for planning

<vision>
## How This Should Work

Resurrectable sessions should be hidden by default but easily accessible via a keyboard toggle. When you open the session selector, you see your live sessions and directories as usual. Press Alt+D and the dead sessions appear in their own section — clearly distinct in pink so you instantly know they're not running.

The list hierarchy becomes:
1. **Live sessions** (top) — your active work
2. **Resurrectable sessions** (middle, when toggled) — dead but recoverable
3. **Directories** (bottom) — for creating new sessions

This keeps the default view clean while making resurrection just a keypress away. No config file needed — it's a runtime toggle.

</vision>

<essential>
## What Must Be Nailed

- **Alt+D toggle** — Quick show/hide without touching config files. Hidden by default.
- **Pink color for dead sessions** — Instant visual distinction from live (cyan) sessions
- **Separate section** — Resurrectable sessions grouped between live sessions and directories, not interleaved

All three work together as a package — the toggle reveals a visually distinct section.

</essential>

<specifics>
## Specific Ideas

- Remove the `show_resurrectable_sessions` config option entirely
- Use color index 3 (pink/magenta) for resurrectable session names
- Keep the `↺` prefix to reinforce "this can be brought back"
- Delete functionality already exists (Del key) — no changes needed there

</specifics>

<notes>
## Additional Context

The pink color that was deemed too harsh for live session indicators is perfect here — it signals "warning, this is dead" which is exactly the right connotation for resurrectable sessions.

</notes>

---

*Phase: 08-resurrectable-session-improvements*
*Context gathered: 2026-01-14*
