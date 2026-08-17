//! Cleanroom Rust port of upstream Go test file: `profile_test.go`
//! Upstream Target Tag / Version: `v0.4.3`

use rusty_colorprofile::{convert16, convert256, env, Profile};
use rusty_x_ansi::color::RGBColor;
use rusty_x_ansi::style::Color;

/// Mirrors `colorful.Color`'s `R*255` -> `uint8` truncation.
fn colorful_to_u8(f: f64) -> u8 {
    (f * 255.0) as u8
}

/// Ported from upstream `TestHexTo256`: `ANSI256.Convert` of a truecolor
/// value maps to the expected 256-color index.
#[test]
fn test_hex_to_256() {
    let cases: &[(f64, f64, f64, u8)] = &[
        // white
        (1.0, 1.0, 1.0, 231),
        // offwhite
        (0.9333, 0.9333, 0.933, 255),
        // slightly brighter than offwhite
        (0.95, 0.95, 0.95, 255),
        // red
        (1.0, 0.0, 0.0, 196),
        // silver foil
        (0.6863, 0.6863, 0.6863, 145),
        // silver chalice
        (0.698, 0.698, 0.698, 249),
        // slightly closer to silver foil
        (0.692, 0.692, 0.692, 145),
        // slightly closer to silver chalice: the float 0.694 truncates to
        // 176 in the 8-bit API, and upstream maps the exact 8-bit 176 to the
        // cube color 145 (verified against `ansi.Convert256` with an RGBA
        // input); the full-precision float 176.97 would map to grey 249.
        (0.694, 0.694, 0.694, 145),
        // gray
        (0.5, 0.5, 0.5, 244),
    ];
    for (i, (r, g, b, expected)) in cases.iter().enumerate() {
        let r = colorful_to_u8(*r);
        let g = colorful_to_u8(*g);
        let b = colorful_to_u8(*b);
        let idx = convert256(r, g, b);
        assert_eq!(idx, *expected, "case {i}: rgb=({r},{g},{b})");
    }
}

/// Ported from upstream `TestDetectionByEnvironment`.
#[test]
fn test_detection_by_environment() {
    let v = |pairs: &[&str]| pairs.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let cases: &[(&[&str], Profile)] = &[
        (&["TERM=dumb"], Profile::NoTty),
        (&["TERM=xterm"], Profile::Ansi),
        (&["TERM=rio"], Profile::TrueColor),
        (&["TERM=xterm-256color"], Profile::Ansi256),
    ];
    for (envp, expected) in cases {
        assert_eq!(&env(&v(envp)), expected, "env={envp:?}");
    }
}

/// Ported from upstream `TestCache`. The upstream cache is a performance
/// optimization omitted by the port (no observable semantics); the conversion
/// assertions are ported 1:1.
#[test]
fn test_convert_vectors() {
    let rgb = |r: u8, g: u8, b: u8| Color::RGB(RGBColor { r, g, b });
    let hex = |s: &str| -> Color {
        let s = s.trim_start_matches('#');
        Color::RGB(RGBColor {
            r: u8::from_str_radix(&s[0..2], 16).unwrap(),
            g: u8::from_str_radix(&s[2..4], 16).unwrap(),
            b: u8::from_str_radix(&s[4..6], 16).unwrap(),
        })
    };

    let cases: &[(Color, Profile, Color)] = &[
        // red
        (rgb(255, 0, 0), Profile::Ansi256, Color::Indexed(196)),
        // grey
        (rgb(128, 128, 128), Profile::Ansi256, Color::Indexed(244)),
        // white
        (rgb(255, 255, 255), Profile::Ansi, Color::Basic(15)),
        // light burgundy
        (hex("#7b2c2c"), Profile::Ansi256, Color::Indexed(88)),
        // truecolor passthrough
        (hex("#8ab7ed"), Profile::TrueColor, hex("#8ab7ed")),
        // offwhite
        (hex("#eeeeee"), Profile::Ansi256, Color::Indexed(255)),
    ];
    for (i, (input, profile, expected)) in cases.iter().enumerate() {
        let got = profile.convert(*input);
        assert_eq!(
            &got,
            &Some(*expected),
            "case {i}: profile={profile:?} input={input:?}"
        );
    }
}

/// Exercises the remaining `Profile::convert` branches: indexed-to-ANSI,
/// RGB-to-ANSI, Default passthrough, and Basic passthrough on every profile.
#[test]
fn test_convert_remaining_branches() {
    use Profile::*;
    let rgb = Color::RGB(RGBColor { r: 255, g: 0, b: 0 });
    let idx = Color::Indexed(196);
    // Indexed to ANSI.
    assert_eq!(Ansi.convert(idx), Some(Color::Basic(9)));
    // Indexed stays indexed on ANSI256.
    assert_eq!(Ansi256.convert(idx), Some(idx));
    // RGB to ANSI.
    assert_eq!(Ansi.convert(rgb), Some(Color::Basic(9)));
    // Default passthrough on every profile above ASCII.
    let dflt = Color::Default;
    for p in [Ansi, Ansi256, TrueColor] {
        assert_eq!(p.convert(dflt), Some(dflt));
    }
    // Basic passthrough.
    let basic = Color::Basic(1);
    for p in [Ansi, Ansi256, TrueColor] {
        assert_eq!(p.convert(basic), Some(basic));
    }
}

/// Exercises the public `convert16` re-export (mirroring upstream
/// `ansi.Convert16` usage).
#[test]
fn test_convert16_reexport() {
    assert_eq!(convert16(196), 9);
    assert_eq!(convert16(244), 7);
}
