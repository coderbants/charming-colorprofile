//! Cleanroom Rust port of upstream Go source file: `env.go`
//! Upstream Target Tag / Version: `v0.4.3`
//!
//! Also covers (same upstream module): `env_other.go`, `env_windows.go`
//!
//! <public-docs>
//! Color-profile detection from environment variables, Terminfo databases,
//! and tmux.
//! </public-docs>

use crate::Profile;
use std::collections::HashMap;

/// The value of `TERM` that disables color support.
pub const DUMB_TERM: &str = "dumb";

/// Environ is a map of environment variables.
#[derive(Debug, Clone, Default)]
pub struct Environ(pub HashMap<String, String>);

/// NewEnviron returns a new environment map from a slice of environment
/// variables.
pub fn environ(env: &[String]) -> Environ {
    let mut m = HashMap::with_capacity(env.len());
    for e in env {
        let mut parts = e.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");
        m.insert(key.to_string(), value.to_string());
    }
    Environ(m)
}

impl Environ {
    /// Lookup returns the value of an environment variable and whether it
    /// exists.
    pub fn lookup(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    /// Get returns the value of an environment variable and empty string if
    /// it doesn't exist.
    pub fn get(&self, key: &str) -> String {
        self.lookup(key).unwrap_or_default()
    }
}

/// Detect returns the color profile based on the terminal output, and
/// environment variables. This respects NO_COLOR, CLICOLOR, and
/// CLICOLOR_FORCE environment variables.
///
/// The rules are as follows:
/// - TERM=dumb is always treated as NoTTY unless CLICOLOR_FORCE=1 is set.
/// - If COLORTERM=truecolor, and the profile is not NoTTY, it gets upgraded
///   to TrueColor.
/// - Using any 256 color terminal (e.g. TERM=xterm-256color) will set the
///   profile to ANSI256.
/// - Using any color terminal (e.g. TERM=xterm-color) will set the profile to
///   ANSI.
/// - Using CLICOLOR=1 without TERM defined should be treated as ANSI if the
///   output is a terminal.
/// - NO_COLOR takes precedence over CLICOLOR/CLICOLOR_FORCE.
///
/// NOTE: upstream takes the output writer and checks `term.IsTerminal`; the
/// port takes the terminal-ness as a parameter.
pub fn detect(is_tty: bool, env: &[String]) -> Profile {
    let environ = environ(env);
    let term = environ.lookup("TERM");
    let is_dumb = term.is_none() || term.as_deref() == Some(DUMB_TERM);
    let envp = color_profile(is_tty, &environ);
    if envp == Profile::TrueColor || env_no_color(&environ) {
        // We already know we have TrueColor, or NO_COLOR is set.
        return envp;
    }

    if is_tty && !is_dumb {
        let term = term.unwrap_or_default();
        let tip = terminfo(&term);
        let tmuxp = tmux_env(&environ);

        // Color profile is the maximum of env, terminfo, and tmux.
        return envp.max(tip).max(tmuxp);
    }

    envp
}

/// Env returns the color profile based on the terminal environment variables.
pub fn env(env: &[String]) -> Profile {
    color_profile(true, &environ(env))
}

/// colorProfile infers the color profile from the environment and
/// terminal-ness.
fn color_profile(isatty: bool, env: &Environ) -> Profile {
    let term = env.lookup("TERM");
    let is_dumb = (term.is_none() && !is_windows()) || term.as_deref() == Some(DUMB_TERM);
    let envp = env_color_profile(env);
    let mut p = if !isatty || is_dumb {
        // Check if the output is a terminal. Treat dumb terminals as NoTTY.
        Profile::NoTty
    } else {
        envp
    };

    if env_no_color(env) && isatty {
        if p > Profile::Ascii {
            p = Profile::Ascii;
        }
        return p;
    }

    if cli_color_forced(env) {
        if p < Profile::Ansi {
            p = Profile::Ansi;
        }
        if envp > p {
            p = envp;
        }

        return p;
    }

    if cli_color(env) {
        if isatty && !is_dumb && p < Profile::Ansi {
            p = Profile::Ansi;
        }
    }

    p
}

fn is_windows() -> bool {
    cfg!(windows)
}

/// EnvNoColor returns true if the environment variables explicitly disable
/// color output by setting NO_COLOR.
pub fn env_no_color(env: &Environ) -> bool {
    parse_bool(&env.get("NO_COLOR")).unwrap_or(false)
}

fn cli_color(env: &Environ) -> bool {
    parse_bool(&env.get("CLICOLOR")).unwrap_or(false)
}

fn cli_color_forced(env: &Environ) -> bool {
    parse_bool(&env.get("CLICOLOR_FORCE")).unwrap_or(false)
}

/// Mirrors Go's `strconv.ParseBool`: accepts 1, t, T, TRUE, true, True, 0,
/// f, F, FALSE, false, False.
fn parse_bool(v: &str) -> Option<bool> {
    match v {
        "1" | "t" | "T" | "TRUE" | "true" | "True" => Some(true),
        "0" | "f" | "F" | "FALSE" | "false" | "False" => Some(false),
        _ => None,
    }
}

fn color_term(env: &Environ) -> bool {
    let ct = env.get("COLORTERM").to_lowercase();
    ct == "truecolor" || ct == "24bit" || ct == "yes" || ct == "true"
}

/// EnvColorProfile infers the color profile from the environment.
fn env_color_profile(env: &Environ) -> Profile {
    let term = env.lookup("TERM");
    let mut p = if term.is_none() || term.as_deref().map(|t| t.is_empty()).unwrap_or(true)
        || term.as_deref() == Some(DUMB_TERM)
    {
        Profile::NoTty
    } else {
        Profile::Ansi
    };
    let term = term.unwrap_or_default();

    match () {
        _ if ["alacritty", "contour", "foot", "ghostty", "kitty", "rio", "st", "wezterm"]
            .iter()
            .any(|t| term.contains(t)) =>
        {
            return Profile::TrueColor;
        }
        _ if term.starts_with("tmux") || term.starts_with("screen") => {
            if p < Profile::Ansi256 {
                p = Profile::Ansi256;
            }
        }
        _ if term.starts_with("xterm") => {
            if p < Profile::Ansi {
                p = Profile::Ansi;
            }
        }
        _ => {}
    }

    if !env.get("WT_SESSION").is_empty() {
        // Windows Terminal supports TrueColor.
        return Profile::TrueColor;
    }

    if env.get("GOOGLE_CLOUD_SHELL").parse::<bool>().unwrap_or(false) {
        return Profile::TrueColor;
    }

    // GNU Screen doesn't support TrueColor. Tmux doesn't support $COLORTERM.
    if color_term(env) && !term.starts_with("screen") && !term.starts_with("tmux") {
        return Profile::TrueColor;
    }

    if term.ends_with("256color") && p < Profile::Ansi256 {
        p = Profile::Ansi256;
    }

    // Direct color terminals support true colors.
    if term.ends_with("direct") {
        return Profile::TrueColor;
    }

    p
}

/// Terminfo returns the color profile based on the terminal's terminfo
/// database. This relies on the Tc and RGB capabilities to determine if the
/// terminal supports TrueColor.
///
/// NOTE: upstream loads the terminfo database via `xo/terminfo`; that parser
/// is not ported. Mirroring Go's behavior when the database cannot be
/// loaded, this returns the ANSI baseline.
pub fn terminfo(term: &str) -> Profile {
    if term.is_empty() || term == DUMB_TERM {
        return Profile::NoTty;
    }
    Profile::Ansi
}

/// Tmux returns the color profile based on `tmux info` output.
pub fn tmux(env: &[String]) -> Profile {
    tmux_impl(&environ(env))
}

/// TmuxEnv returns the color profile based on an [Environ].
pub(crate) fn tmux_env(env: &Environ) -> Profile {
    tmux_impl(env)
}

/// tmux returns the color profile based on the tmux environment variables.
fn tmux_impl(env: &Environ) -> Profile {
    if let Some(tmux) = env.lookup("TMUX") {
        if tmux.is_empty() {
            // Not in tmux.
            return Profile::NoTty;
        }
    } else {
        // Not in tmux.
        return Profile::NoTty;
    }

    // Check if tmux has either Tc or RGB capabilities. Otherwise, return
    // ANSI256.
    let p = Profile::Ansi256;
    match std::process::Command::new("tmux").arg("info").output() {
        Ok(out) => {
            for line in out.stdout.split(|&b| b == b'\n') {
                if (line.windows(2).any(|w| w == b"Tc") || line.windows(3).any(|w| w == b"RGB"))
                    && line.windows(4).any(|w| w == b"true")
                {
                    return Profile::TrueColor;
                }
            }
        }
        Err(_) => return p,
    }

    p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envs(pairs: &[(&str, &str)]) -> Vec<String> {
        pairs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect()
    }

    #[test]
    fn test_env_no_color() {
        let e = environ(&envs(&[("NO_COLOR", "1")]));
        assert!(env_no_color(&e));
        let e = environ(&envs(&[("NO_COLOR", "0")]));
        assert!(!env_no_color(&e));
    }

    #[test]
    fn test_env_profile_terms() {
        assert_eq!(
            env(&envs(&[("TERM", "xterm-256color")])),
            Profile::Ansi256
        );
        assert_eq!(env(&envs(&[("TERM", "xterm")])), Profile::Ansi);
        assert_eq!(
            env(&envs(&[("TERM", "kitty")])),
            Profile::TrueColor
        );
        assert_eq!(
            env(&envs(&[("TERM", "xterm"), ("COLORTERM", "truecolor")])),
            Profile::TrueColor
        );
        assert_eq!(env(&envs(&[("TERM", "dumb")])), Profile::NoTty);
        assert_eq!(env(&envs(&[])), Profile::NoTty);
    }

    #[test]
    fn test_env_profile_no_color() {
        assert_eq!(
            env(&envs(&[("TERM", "xterm-256color"), ("NO_COLOR", "1")])),
            Profile::Ascii
        );
    }

    #[test]
    fn test_env_profile_clicolor_force() {
        assert_eq!(
            env(&envs(&[("TERM", "dumb"), ("CLICOLOR_FORCE", "1")])),
            Profile::Ansi
        );
    }

    #[test]
    fn test_detect_non_tty() {
        assert_eq!(
            detect(false, &envs(&[("TERM", "xterm-256color")])),
            Profile::NoTty
        );
        // CLICOLOR_FORCE upgrades even non-TTY.
        assert_eq!(
            detect(false, &envs(&[("CLICOLOR_FORCE", "1")])),
            Profile::Ansi
        );
    }


    #[test]
    fn test_env_go_verified_vectors() {
        // Byte-verified against the real Go library via the workspace parity
        // harness (tools/go-probes/colorprofile).
        let cases: &[(&[&str], Profile)] = &[
            (&["TERM=xterm-256color"], Profile::Ansi256),
            (&["TERM=xterm"], Profile::Ansi),
            (&["TERM=kitty"], Profile::TrueColor),
            (&["TERM=xterm", "COLORTERM=truecolor"], Profile::TrueColor),
            (&["TERM=dumb"], Profile::NoTty),
            (&[], Profile::NoTty),
            (&["TERM=xterm-256color", "NO_COLOR=1"], Profile::Ascii),
            (&["TERM=dumb", "CLICOLOR_FORCE=1"], Profile::Ansi),
            (&["TERM=xterm-256color", "NO_COLOR=t"], Profile::Ascii),
            (&["TERM=xterm", "CLICOLOR=1"], Profile::Ansi),
            (&["TERM=screen"], Profile::Ansi256),
            (&["TERM=tmux"], Profile::Ansi256),
            (&["TERM=alacritty"], Profile::TrueColor),
            (&["TERM=xterm-direct"], Profile::TrueColor),
            (&["GOOGLE_CLOUD_SHELL=1"], Profile::NoTty),
            (&["TERM=xterm-256color", "TTY_FORCE=1"], Profile::Ansi256),
        ];
        for (env, expected) in cases {
            let v: Vec<String> = env.iter().map(|s| s.to_string()).collect();
            assert_eq!(&env(&v), expected, "env={env:?}");
        }
    }

    #[test]
    fn test_environ_helpers() {
        let e = environ(&envs(&[("FOO", "bar"), ("EMPTY", "")]));
        assert_eq!(e.get("FOO"), "bar");
        assert_eq!(e.get("EMPTY"), "");
        assert_eq!(e.get("MISSING"), "");
        assert_eq!(e.lookup("MISSING"), None);
    }
}
