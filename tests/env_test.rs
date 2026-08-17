//! Cleanroom Rust port of upstream Go test file: `env_test.go`
//! Upstream Target Tag / Version: `v0.4.3`

use rusty_colorprofile::{detect, env, terminfo, tmux, Profile};

fn envs(pairs: &[&str]) -> Vec<String> {
    pairs.iter().map(|s| s.to_string()).collect()
}

/// Ported from upstream `cases` + `TestEnvColorProfile`; the windows-only
/// expectations are omitted (the port targets POSIX, matching the upstream
/// `runtime.GOOS != "windows"` branch).
#[test]
fn test_env_color_profile() {
    let cases: &[(&[&str], Profile)] = &[
        // empty
        (&[], Profile::NoTty),
        // no tty
        (&["TERM=dumb"], Profile::NoTty),
        // dumb term, truecolor, not forced
        (&["TERM=dumb", "COLORTERM=truecolor"], Profile::NoTty),
        // dumb term, truecolor, forced
        (
            &["TERM=dumb", "COLORTERM=truecolor", "CLICOLOR_FORCE=1"],
            Profile::TrueColor,
        ),
        // dumb term, CLICOLOR_FORCE=1
        (&["TERM=dumb", "CLICOLOR_FORCE=1"], Profile::Ansi),
        // dumb term, CLICOLOR=1
        (&["TERM=dumb", "CLICOLOR=1"], Profile::NoTty),
        // xterm-256color
        (&["TERM=xterm-256color"], Profile::Ansi256),
        // xterm-256color, CLICOLOR=1
        (&["TERM=xterm-256color", "CLICOLOR=1"], Profile::Ansi256),
        // xterm-256color, COLORTERM=yes
        (
            &["TERM=xterm-256color", "COLORTERM=yes"],
            Profile::TrueColor,
        ),
        // xterm-256color, NO_COLOR=1
        (&["TERM=xterm-256color", "NO_COLOR=1"], Profile::Ascii),
        // xterm
        (&["TERM=xterm"], Profile::Ansi),
        // xterm, NO_COLOR=1
        (&["TERM=xterm", "NO_COLOR=1"], Profile::Ascii),
        // xterm, CLICOLOR=1
        (&["TERM=xterm", "CLICOLOR=1"], Profile::Ansi),
        // xterm, CLICOLOR_FORCE=1
        (&["TERM=xterm", "CLICOLOR_FORCE=1"], Profile::Ansi),
        // xterm-16color
        (&["TERM=xterm-16color"], Profile::Ansi),
        // xterm-color
        (&["TERM=xterm-color"], Profile::Ansi),
        // xterm-256color, NO_COLOR=1, CLICOLOR_FORCE=1
        (
            &["TERM=xterm-256color", "NO_COLOR=1", "CLICOLOR_FORCE=1"],
            Profile::Ascii,
        ),
        // Windows Terminal without TERM (POSIX: NoTTY)
        (&["WT_SESSION=1"], Profile::NoTty),
        // Windows Terminal with TERM set: TrueColor on all platforms
        (&["TERM=xterm-256color", "WT_SESSION=1"], Profile::TrueColor),
        // screen default
        (&["TERM=screen"], Profile::Ansi256),
        // screen colorterm: GNU Screen doesn't support TrueColor
        (&["TERM=screen", "COLORTERM=truecolor"], Profile::Ansi256),
        // tmux colorterm: Tmux doesn't support $COLORTERM
        (&["TERM=tmux", "COLORTERM=truecolor"], Profile::Ansi256),
        // tmux 256color
        (&["TERM=tmux-256color"], Profile::Ansi256),
        // ignore COLORTERM when no TERM is defined
        (&["COLORTERM=truecolor"], Profile::NoTty),
        // direct color xterm terminal
        (&["TERM=xterm-direct"], Profile::TrueColor),
    ];
    for (i, (envp, expected)) in cases.iter().enumerate() {
        let v = envs(envp);
        assert_eq!(&env(&v), expected, "case {i}: env={envp:?}");
    }
}

/// Exercises the `detect` branches (is_tty, terminfo, tmux, early returns)
/// that the upstream `Env`-only table cannot reach.
#[test]
fn test_detect_branches() {
    let v = |pairs: &[&str]| pairs.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    // is_tty = false: everything collapses to NoTTY unless forced.
    assert_eq!(detect(false, &v(&["TERM=xterm-256color"])), Profile::NoTty);
    assert_eq!(
        detect(false, &v(&["TERM=xterm-256color", "CLICOLOR_FORCE=1"])),
        Profile::Ansi256
    );
    // NO_COLOR only downgrades when the output is a terminal.
    assert_eq!(
        detect(false, &v(&["TERM=xterm-256color", "NO_COLOR=1"])),
        Profile::NoTty
    );
    // is_tty = true with a known truecolor TERM: early return.
    assert_eq!(detect(true, &v(&["TERM=kitty"])), Profile::TrueColor);
    // NO_COLOR early return.
    assert_eq!(
        detect(true, &v(&["TERM=xterm-256color", "NO_COLOR=1"])),
        Profile::Ascii
    );
    // is_tty = true, non-dumb TERM: env.max(terminfo).max(tmux).
    assert_eq!(detect(true, &v(&["TERM=xterm"])), Profile::Ansi);
    assert_eq!(detect(true, &v(&["TERM=screen"])), Profile::Ansi256);
    assert_eq!(
        detect(true, &v(&["TERM=xterm", "COLORTERM=truecolor"])),
        Profile::TrueColor
    );
    // dumb TERM with is_tty = true stays NoTTY.
    assert_eq!(detect(true, &v(&["TERM=dumb"])), Profile::NoTty);
}

/// Exercises `terminfo` and the `tmux` environment paths.
#[test]
fn test_terminfo_and_tmux() {
    assert_eq!(terminfo(""), Profile::NoTty);
    assert_eq!(terminfo("dumb"), Profile::NoTty);
    assert_eq!(terminfo("xterm"), Profile::Ansi);
    // TMUX unset: not in tmux.
    assert_eq!(tmux(&[]), Profile::NoTty);
    // TMUX empty: not in tmux.
    assert_eq!(tmux(&["TMUX=".to_string()]), Profile::NoTty);
    // TMUX set: resolves via `tmux info`; without tmux (or a Tmux/RGB
    // capability) the fallback is ANSI256.
    let p = tmux(&["TMUX=1".to_string()]);
    assert!(
        p == Profile::Ansi256 || p == Profile::TrueColor,
        "unexpected tmux profile {p:?}"
    );
}

/// Exercises the remaining `env_color_profile` branches: cloud shell, the
/// COLORTERM upgrade, 256color suffix and direct-color suffix.
#[test]
fn test_env_extra_branches() {
    let v = |pairs: &[&str]| pairs.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    // GOOGLE_CLOUD_SHELL with a TERM.
    assert_eq!(
        env(&v(&["TERM=xterm", "GOOGLE_CLOUD_SHELL=1"])),
        Profile::TrueColor
    );
    // COLORTERM=24bit / =true upgrade.
    assert_eq!(
        env(&v(&["TERM=xterm", "COLORTERM=24bit"])),
        Profile::TrueColor
    );
    assert_eq!(
        env(&v(&["TERM=xterm", "COLORTERM=true"])),
        Profile::TrueColor
    );
    // 256color suffix on a non-xterm term.
    assert_eq!(env(&v(&["TERM=foo-256color"])), Profile::Ansi256);
    // direct suffix.
    assert_eq!(env(&v(&["TERM=whatever-direct"])), Profile::TrueColor);
}

/// The `cliColor` upgrade branch requires a non-dumb TERM with a profile
/// below ANSI (e.g. an empty TERM) plus CLICOLOR=1.
#[test]
fn test_env_clicolor_upgrade() {
    let v = |pairs: &[&str]| pairs.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(env(&v(&["TERM=", "CLICOLOR=1"])), Profile::Ansi);
    // Without CLICOLOR the empty TERM stays NoTTY.
    assert_eq!(env(&v(&["TERM="])), Profile::NoTty);
}
