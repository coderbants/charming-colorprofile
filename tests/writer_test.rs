//! Cleanroom Rust port of upstream Go test file: `writer_test.go`
//! Upstream Target Tag / Version: `v0.4.3`

use rusty_colorprofile::{detect, new_writer, Profile, Writer};

fn write_with(profile: Profile, input: &str) -> String {
    let mut out: Vec<u8> = Vec::new();
    Writer {
        forward: &mut out,
        profile,
    }
    .write(input.as_bytes())
    .unwrap();
    String::from_utf8(out).unwrap()
}

/// Ported from upstream `TestWriter` + `writer_cases` (the five profiles;
/// NoTTY expectations come from `ansi.Strip`).
#[test]
fn test_writer() {
    let cases: &[(&str, &str, &str, &str, &str)] = &[
        // name, expectedTrueColor, expectedANSI256, expectedANSI, expectedAscii
        ("empty", "", "", "", ""),
        (
            "no styles",
            "hello world",
            "hello world",
            "hello world",
            "hello world",
        ),
        (
            "simple style attributes",
            "hello \x1b[1mworld\x1b[m",
            "hello \x1b[1mworld\x1b[m",
            "hello \x1b[1mworld\x1b[m",
            "hello \x1b[1mworld\x1b[m",
        ),
        (
            "simple ansi color fg",
            "hello \x1b[31mworld\x1b[m",
            "hello \x1b[31mworld\x1b[m",
            "hello \x1b[31mworld\x1b[m",
            "hello \x1b[mworld\x1b[m",
        ),
        (
            "default fg color after ansi color",
            "\x1b[31mhello \x1b[39mworld\x1b[m",
            "\x1b[31mhello \x1b[39mworld\x1b[m",
            "\x1b[31mhello \x1b[39mworld\x1b[m",
            "\x1b[mhello \x1b[mworld\x1b[m",
        ),
        (
            "ansi color fg and bg",
            "\x1b[31;42mhello world\x1b[m",
            "\x1b[31;42mhello world\x1b[m",
            "\x1b[31;42mhello world\x1b[m",
            "\x1b[mhello world\x1b[m",
        ),
        (
            "bright ansi color fg and bg",
            "\x1b[91;102mhello world\x1b[m",
            "\x1b[91;102mhello world\x1b[m",
            "\x1b[91;102mhello world\x1b[m",
            "\x1b[mhello world\x1b[m",
        ),
        (
            "simple 256 color fg",
            "hello \x1b[38;5;196mworld\x1b[m",
            "hello \x1b[38;5;196mworld\x1b[m",
            "hello \x1b[91mworld\x1b[m",
            "hello \x1b[mworld\x1b[m",
        ),
        (
            "256 color bg",
            "\x1b[48;5;196mhello world\x1b[m",
            "\x1b[48;5;196mhello world\x1b[m",
            "\x1b[101mhello world\x1b[m",
            "\x1b[mhello world\x1b[m",
        ),
        (
            "simple true color bg",
            "hello \x1b[38;2;255;133;55mworld\x1b[m",
            "hello \x1b[38;5;209mworld\x1b[m",
            "hello \x1b[91mworld\x1b[m",
            "hello \x1b[mworld\x1b[m",
        ),
        (
            "itu true color bg",
            "hello \x1b[38:2::255:133:55mworld\x1b[m",
            "hello \x1b[38;5;209mworld\x1b[m",
            "hello \x1b[91mworld\x1b[m",
            "hello \x1b[mworld\x1b[m",
        ),
        (
            "simple ansi 256 color bg",
            "hello \x1b[48:5:196mworld\x1b[m",
            "hello \x1b[48;5;196mworld\x1b[m",
            "hello \x1b[101mworld\x1b[m",
            "hello \x1b[mworld\x1b[m",
        ),
        (
            "simple missing param",
            "\x1b[31mhello \x1b[;1mworld",
            "\x1b[31mhello \x1b[;1mworld",
            "\x1b[31mhello \x1b[;1mworld",
            "\x1b[mhello \x1b[;1mworld",
        ),
        (
            "color with other attributes",
            "\x1b[1;38;5;204mhello \x1b[38;5;204mworld\x1b[m",
            "\x1b[1;38;5;204mhello \x1b[38;5;204mworld\x1b[m",
            "\x1b[1;91mhello \x1b[91mworld\x1b[m",
            "\x1b[1mhello \x1b[mworld\x1b[m",
        ),
    ];

    for (i, (name, exp_tc, exp_256, exp_ansi, exp_ascii)) in cases.iter().enumerate() {
        let input = match i {
            0 => "",
            1 => "hello world",
            2 => "hello \x1b[1mworld\x1b[m",
            3 => "hello \x1b[31mworld\x1b[m",
            4 => "\x1b[31mhello \x1b[39mworld\x1b[m",
            5 => "\x1b[31;42mhello world\x1b[m",
            6 => "\x1b[91;102mhello world\x1b[m",
            7 => "hello \x1b[38;5;196mworld\x1b[m",
            8 => "\x1b[48;5;196mhello world\x1b[m",
            9 => "hello \x1b[38;2;255;133;55mworld\x1b[m",
            10 => "hello \x1b[38:2::255:133:55mworld\x1b[m",
            11 => "hello \x1b[48:5:196mworld\x1b[m",
            12 => "\x1b[31mhello \x1b[;1mworld",
            13 => "\x1b[1;38;5;204mhello \x1b[38;5;204mworld\x1b[m",
            _ => unreachable!(),
        };

        let profiles: &[(Profile, &str)] = &[
            (Profile::TrueColor, exp_tc),
            (Profile::Ansi256, exp_256),
            (Profile::Ansi, exp_ansi),
            (Profile::Ascii, exp_ascii),
            (Profile::NoTty, ""), // NoTTY expectation filled below
        ];

        for (profile, expected) in profiles {
            // NoTTY expectations mirror the upstream `ansi.Strip`.
            let expected = if *profile == Profile::NoTty {
                rusty_x_ansi::util::strip(input)
            } else {
                expected.to_string()
            };
            let got = write_with(*profile, input);
            assert_eq!(&got, &expected, "case {i} ({name}) profile={profile:?}");
        }
    }
}

/// Ported from upstream `TestNewWriterPanic`.
#[test]
fn test_new_writer_dumb_term() {
    let out = new_writer(Vec::new(), &["TERM=dumb".to_string()]);
    assert_eq!(out.profile, Profile::NoTty);
}

/// Ported from upstream `TestNewWriterOsEnviron`: the writer adopts the
/// profile detected from the process environment.
#[test]
fn test_new_writer_os_environ() {
    let env: Vec<String> = std::env::vars().map(|(k, v)| format!("{k}={v}")).collect();
    let out = new_writer(Vec::new(), &env);
    assert_eq!(out.profile, detect(true, &env));
}

/// Ported from upstream `TestWriterMiddleware`: a buffered writer layered over
/// the profile writer flushes the downsampled output.
#[test]
fn test_writer_middleware() {
    for profile in [
        Profile::TrueColor,
        Profile::Ansi256,
        Profile::Ansi,
        Profile::Ascii,
        Profile::NoTty,
    ] {
        let mut out: Vec<u8> = Vec::new();
        {
            let mut bw = std::io::BufWriter::new(Writer {
                forward: &mut out,
                profile,
            });
            use std::io::Write as _;
            bw.write_all(b"\x1b[38;2;20;249;10mhello\x1b[0m\n").unwrap();
            bw.flush().unwrap();
        }
        assert!(!out.is_empty(), "profile={profile:?} produced no output");
    }
}

/// Exercises the remaining `handle_sgr` branches: underline colors (58/59),
/// default colors, bright colors, and the non-color parameter fallback.
#[test]
fn test_writer_extra_sgr_branches() {
    use Profile::*;
    // Underline color (58) preserved on TrueColor/ANSI256, dropped on ASCII.
    assert_eq!(
        write_with(TrueColor, "\x1b[58;5;196mhello\x1b[m"),
        "\x1b[58;5;196mhello\x1b[m"
    );
    assert_eq!(
        write_with(Ansi256, "\x1b[58;2;255;133;55mhello\x1b[m"),
        "\x1b[58;5;209mhello\x1b[m"
    );
    // Default underline color (59).
    assert_eq!(
        write_with(Ansi, "\x1b[59mhello\x1b[m"),
        "\x1b[59mhello\x1b[m"
    );
    // Default fg/bg after a color (39/49).
    assert_eq!(
        write_with(Ansi, "\x1b[39mhello\x1b[m"),
        "\x1b[39mhello\x1b[m"
    );
    assert_eq!(
        write_with(Ansi256, "\x1b[49mhello\x1b[m"),
        "\x1b[49mhello\x1b[m"
    );
    // Bright colors on ANSI (90..=97, 100..=107).
    assert_eq!(
        write_with(Ansi, "\x1b[95mhello\x1b[m"),
        "\x1b[95mhello\x1b[m"
    );
    assert_eq!(
        write_with(Ansi, "\x1b[104mhello\x1b[m"),
        "\x1b[104mhello\x1b[m"
    );
    // Non-color attribute passthrough (underline style, etc.).
    assert_eq!(write_with(Ansi, "\x1b[4mhello\x1b[m"), "\x1b[4mhello\x1b[m");
    // 256-color fg on ANSI256 stays indexed.
    assert_eq!(
        write_with(Ansi256, "\x1b[38;5;196mhello\x1b[m"),
        "\x1b[38;5;196mhello\x1b[m"
    );
    // TrueColor RGB on ANSI256 downsamples.
    assert_eq!(
        write_with(Ansi256, "\x1b[48;2;255;133;55mhello\x1b[m"),
        "\x1b[48;5;209mhello\x1b[m"
    );
}

/// Exercises the `Unknown` profile, `write_string`, and a Default-color SGR.
#[test]
fn test_writer_unknown_and_string() {
    use Profile::*;
    // Unknown behaves like NoTTY (strip).
    assert_eq!(write_with(Unknown, "\x1b[31mred\x1b[m"), "red");
    // write_string goes through the same pipeline.
    let mut out: Vec<u8> = Vec::new();
    {
        let mut w = Writer {
            forward: &mut out,
            profile: Ansi256,
        };
        w.write_string("\x1b[38;5;196mred\x1b[m").unwrap();
    }
    assert_eq!(String::from_utf8(out).unwrap(), "\x1b[38;5;196mred\x1b[m");
    // An incomplete 256-color spec (38;5 without an index) leaves the
    // orphan param in place and resets to the default color, mirroring
    // upstream's SGR handling.
    let out = write_with(Ansi, "\x1b[38;5mhello\x1b[m");
    assert_eq!(out, "\x1b[39;5mhello\x1b[m");
}
