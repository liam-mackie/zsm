//! Session names end up in zellij's IPC socket path, so the usable length
//! depends on where that socket lives. Unix socket paths must fit in
//! `sockaddr_un.sun_path`: zellij enforces the macOS limit of 103 bytes.

const MAX_SOCKET_PATH: usize = 103;

/// Name budget assumed until the socket dir probe answers, sized for the worst
/// common case: macOS's `$TMPDIR/zellij-<uid>/contract_version_N/` leaves ~24
/// bytes, minus margin for longer uids and future contract versions.
pub const FALLBACK_NAME_BUDGET: usize = 22;

/// Generated names stay below this even when the socket allows far more (on
/// Linux the budget can exceed 70) — a switcher full of long names is unusable.
pub const DEFAULT_MAX_NAME_LENGTH: usize = 29;

/// Kept free for the conflict suffix (`<name><sep>99`) added at creation time.
const INCREMENT_HEADROOM: usize = 3;

/// Resolves the socket directory the same way zellij does ($ZELLIJ_SOCKET_DIR,
/// then the XDG runtime dir, then $TMPDIR/zellij-<uid>), but discovers the
/// contract_version dir on disk rather than hardcoding zellij's current
/// contract version. Run with the host's `sh` via run_command, which inherits
/// the zellij server's environment — the same one that placed the socket.
pub const SOCKET_DIR_PROBE: &str = r##"
base="$ZELLIJ_SOCKET_DIR"
[ -z "$base" ] && [ -n "$XDG_RUNTIME_DIR" ] && base="$XDG_RUNTIME_DIR/zellij"
if [ -z "$base" ]; then tmp="${TMPDIR:-/tmp}"; base="${tmp%/}/zellij-$(id -u)"; fi
found=""
for d in "$base"/contract_version_*; do
    [ -e "$d" ] && [ "${#d}" -gt "${#found}" ] && found="$d"
done
printf %s "${found:-$base/contract_version_1}"
"##;

/// Bytes available for a session name given the socket directory it lives in.
pub fn name_budget_for_socket_dir(socket_dir: &str) -> usize {
    MAX_SOCKET_PATH
        .saturating_sub(socket_dir.trim_end_matches('/').len() + 1)
        .max(1)
}

/// Max length for generated names: the readability ceiling or the socket
/// budget minus increment headroom, whichever is tighter.
pub fn max_generated_name_len(budget: usize) -> usize {
    DEFAULT_MAX_NAME_LENGTH
        .min(budget.saturating_sub(INCREMENT_HEADROOM))
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The path from a real macOS failure: a 105-byte socket path for a
    // 26-byte name, meaning this dir leaves a 24-byte budget.
    const MACOS_SOCKET_DIR: &str =
        "/var/folders/b3/45ywmlcd4h753vdpw_p11c740000gn/T/zellij-501/contract_version_1";

    #[test]
    fn budget_for_macos_socket_dir() {
        assert_eq!(name_budget_for_socket_dir(MACOS_SOCKET_DIR), 24);
    }

    #[test]
    fn budget_ignores_trailing_slash() {
        let with_slash = format!("{}/", MACOS_SOCKET_DIR);
        assert_eq!(name_budget_for_socket_dir(&with_slash), 24);
    }

    #[test]
    fn budget_never_reaches_zero() {
        let absurd = "x".repeat(200);
        assert_eq!(name_budget_for_socket_dir(&absurd), 1);
    }

    #[test]
    fn generated_len_capped_by_readability() {
        assert_eq!(max_generated_name_len(100), DEFAULT_MAX_NAME_LENGTH);
    }

    #[test]
    fn generated_len_leaves_increment_headroom() {
        assert_eq!(max_generated_name_len(24), 21);
    }

    #[test]
    fn generated_len_never_reaches_zero() {
        assert_eq!(max_generated_name_len(2), 1);
        assert_eq!(max_generated_name_len(0), 1);
    }
}
