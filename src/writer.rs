//! Cleanroom Rust port of upstream Go source file: `writer.go`
//! Upstream Target Tag / Version: `v0.4.3`
//!
//! <public-docs>
//! The color profile writer: writes ANSI text to an underlying writer,
//! downsampling SGR color sequences to the detected profile.
//! </public-docs>

use crate::{detect, Profile};
use rusty_x_ansi::color::{BasicColor, IndexedColor, RGBColor};
use rusty_x_ansi::parser::{decode_sequence, get_parser, has_csi_prefix};
use rusty_x_ansi::style::{read_style_color, Color};
use std::io::{self, Write};

/// NewWriter creates a new color profile writer that downgrades color
/// sequences based on the detected color profile.
pub fn new_writer<W: Write>(w: W, environ: &[String]) -> Writer<W> {
    Writer {
        forward: w,
        profile: detect(true, environ),
    }
}

/// Writer represents a color profile writer that writes ANSI sequences to the
/// underlying writer.
pub struct Writer<W: Write> {
    /// The underlying writer.
    pub forward: W,
    /// The color profile used for downsampling.
    pub profile: Profile,
}

impl<W: Write> Writer<W> {
    /// Write writes the given text to the underlying writer.
    pub fn write(&mut self, p: &[u8]) -> io::Result<usize> {
        match self.profile {
            Profile::TrueColor => self.forward.write(p),
            Profile::NoTty | Profile::Unknown => {
                let stripped = rusty_x_ansi::util::strip(std::str::from_utf8(p).unwrap_or(""));
                self.forward.write_all(stripped.as_bytes())?;
                Ok(p.len())
            }
            Profile::Ascii | Profile::Ansi | Profile::Ansi256 => {
                self.downsample(p)?;
                Ok(p.len())
            }
        }
    }

    /// downsample downgrades the given text to the appropriate color profile.
    fn downsample(&mut self, p: &[u8]) -> io::Result<()> {
        let mut buf: Vec<u8> = Vec::new();
        let mut state = rusty_x_ansi::parser::NORMAL_STATE;
        let mut parser = get_parser();

        let mut rest = p;
        while !rest.is_empty() {
            parser.reset();
            let d = decode_sequence(rest, state, Some(&mut parser));
            let seq = d.seq;
            let read = d.n;
            state = d.state;

            if has_csi_prefix(seq) && parser.command() as u8 == b'm' {
                handle_sgr(&self.profile, &parser, &mut buf);
            } else {
                // If we're not a style SGR sequence, just write the bytes.
                buf.extend_from_slice(seq);
            }

            rest = &rest[read..];
        }

        self.forward.write_all(&buf)
    }

    /// WriteString writes the given text to the underlying writer.
    pub fn write_string(&mut self, s: &str) -> io::Result<usize> {
        self.write(s.as_bytes())
    }
}

/// The ANSI SGR style parameter list being built.
type StyleParams = Vec<String>;

/// The SGR prefix for a color position.
#[derive(Clone, Copy)]
enum ColorPos {
    /// Foreground (38).
    Fg,
    /// Background (48).
    Bg,
    /// Underline (58).
    Ul,
}

impl ColorPos {
    fn prefix(self) -> &'static str {
        match self {
            ColorPos::Fg => "38",
            ColorPos::Bg => "48",
            ColorPos::Ul => "58",
        }
    }
    fn basic_prefix(self) -> i32 {
        match self {
            ColorPos::Fg => 30,
            ColorPos::Bg => 40,
            ColorPos::Ul => 50,
        }
    }
    fn default_attr(self) -> &'static str {
        match self {
            ColorPos::Fg => "39",
            ColorPos::Bg => "49",
            ColorPos::Ul => "59",
        }
    }
}

/// Encodes a color as an SGR parameter for the given position, mirroring
/// upstream `foregroundColorString`/`backgroundColorString`/`underlineColorString`.
fn color_param(pos: ColorPos, c: Option<Color>) -> String {
    match c {
        None => pos.default_attr().to_string(),
        Some(Color::Basic(b)) => {
            // "3<n>" or "9<n>" foreground, "4<n>" or "10<n>" background.
            if b < 8 {
                (pos.basic_prefix() + b as i32).to_string()
            } else {
                (pos.basic_prefix() + 60 + (b as i32 - 8)).to_string()
            }
        }
        Some(Color::Indexed(i)) => format!("{};5;{}", pos.prefix(), i),
        Some(Color::RGB(rgb)) => format!("{};2;{};{};{}", pos.prefix(), rgb.r, rgb.g, rgb.b),
        Some(Color::Default) => pos.default_attr().to_string(),
    }
}

/// handleSgr processes an SGR sequence and appends the downsampled style to
/// the buffer.
fn handle_sgr(profile: &Profile, parser: &rusty_x_ansi::parser::Parser, buf: &mut Vec<u8>) {
    let mut style: StyleParams = Vec::new();
    let params = parser.params().as_slice().to_vec();

    let mut i = 0usize;
    while i < params.len() {
        let param = params[i] & !rusty_x_ansi::parser::HAS_MORE_FLAG;
        if param == rusty_x_ansi::parser::MISSING_PARAM {
            style.push("".to_string());
            i += 1;
            continue;
        }

        match param {
            0 => {
                // SGR default parameter is 0. We use an empty string to
                // reduce the number of bytes written to the buffer.
                style.push(String::new());
            }
            30..=37 => {
                // 8-bit foreground color
                if *profile < Profile::Ansi {
                    i += 1;
                    continue;
                }
                let c = profile.convert(Color::Basic((param - 30) as BasicColor));
                style.push(color_param(ColorPos::Fg, c));
            }
            38 => {
                // 16 or 24-bit foreground color
                let mut c = None;
                let n = read_style_color(&params[i..], &mut c);
                if n > 0 {
                    i += n - 1;
                }
                if *profile < Profile::Ansi {
                    i += 1;
                    continue;
                }
                style.push(color_param(
                    ColorPos::Fg,
                    profile.convert(c.unwrap_or(Color::Default)),
                ));
            }
            39 => {
                // default foreground color
                if *profile < Profile::Ansi {
                    i += 1;
                    continue;
                }
                style.push(color_param(ColorPos::Fg, None));
            }
            40..=47 => {
                // 8-bit background color
                if *profile < Profile::Ansi {
                    i += 1;
                    continue;
                }
                let c = profile.convert(Color::Basic((param - 40) as BasicColor));
                style.push(color_param(ColorPos::Bg, c));
            }
            48 => {
                // 16 or 24-bit background color
                let mut c = None;
                let n = read_style_color(&params[i..], &mut c);
                if n > 0 {
                    i += n - 1;
                }
                if *profile < Profile::Ansi {
                    i += 1;
                    continue;
                }
                style.push(color_param(
                    ColorPos::Bg,
                    profile.convert(c.unwrap_or(Color::Default)),
                ));
            }
            49 => {
                // default background color
                if *profile < Profile::Ansi {
                    i += 1;
                    continue;
                }
                style.push(color_param(ColorPos::Bg, None));
            }
            58 => {
                // 16 or 24-bit underline color
                let mut c = None;
                let n = read_style_color(&params[i..], &mut c);
                if n > 0 {
                    i += n - 1;
                }
                if *profile < Profile::Ansi {
                    i += 1;
                    continue;
                }
                style.push(color_param(
                    ColorPos::Ul,
                    profile.convert(c.unwrap_or(Color::Default)),
                ));
            }
            59 => {
                // default underline color
                if *profile < Profile::Ansi {
                    i += 1;
                    continue;
                }
                style.push(color_param(ColorPos::Ul, None));
            }
            90..=97 => {
                // 8-bit bright foreground color
                if *profile < Profile::Ansi {
                    i += 1;
                    continue;
                }
                let c = profile.convert(Color::Basic((param - 90 + 8) as BasicColor));
                style.push(color_param(ColorPos::Fg, c));
            }
            100..=107 => {
                // 8-bit bright background color
                if *profile < Profile::Ansi {
                    i += 1;
                    continue;
                }
                let c = profile.convert(Color::Basic((param - 100 + 8) as BasicColor));
                style.push(color_param(ColorPos::Bg, c));
            }
            _ => {
                // If this is not a color attribute, just append it to the
                // style.
                style.push(param.to_string());
            }
        }
        i += 1;
    }

    let _ = write!(buf, "\x1b[{}m", style.join(";"));
}

#[allow(unused)]
fn _color_types(_: (BasicColor, IndexedColor, RGBColor)) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Profile::*;

    fn write_with(profile: Profile, input: &str) -> String {
        let mut out: Vec<u8> = Vec::new();
        {
            let mut w = Writer {
                forward: Box::new(&mut out),
                profile,
            };
            let _ = w.write(input.as_bytes());
        }
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn test_writer_truecolor_passthrough() {
        let out = write_with(Profile::TrueColor, "\x1b[38;2;255;0;0mred");
        assert_eq!(out, "\x1b[38;2;255;0;0mred");
    }

    #[test]
    fn test_writer_notty_strips() {
        let out = write_with(Profile::NoTty, "\x1b[31mred\x1b[0m");
        assert_eq!(out, "red");
    }

    #[test]
    fn test_writer_ansi256_downsamples() {
        // TrueColor SGR to 256-color index (upstream writer_test.go:
        // "hello \x1b[38;2;255;133;55mworld\x1b[m" -> ANSI256
        // "hello \x1b[38;5;209mworld\x1b[m").
        let out = write_with(Profile::Ansi256, "hello \x1b[38;2;255;133;55mworld\x1b[m");
        assert_eq!(out, "hello \x1b[38;5;209mworld\x1b[m");
    }

    #[test]
    fn test_writer_ansi_downsamples() {
        // 256-color fg to ANSI (upstream: "hello \x1b[38;5;196mworld\x1b[m"
        // -> ANSI "hello \x1b[91mworld\x1b[m").
        let out = write_with(Profile::Ansi, "hello \x1b[38;5;196mworld\x1b[m");
        assert_eq!(out, "hello \x1b[91mworld\x1b[m");
        // TrueColor fg to ANSI (upstream: -> "hello \x1b[91mworld\x1b[m").
        let out = write_with(Profile::Ansi, "hello \x1b[38;2;255;133;55mworld\x1b[m");
        assert_eq!(out, "hello \x1b[91mworld\x1b[m");
    }

    #[test]
    fn test_writer_ansi_preserves_basic() {
        let out = write_with(Profile::Ansi, "\x1b[31mred\x1b[0m");
        assert_eq!(out, "\x1b[31mred\x1b[m");
    }

    #[test]
    fn test_writer_go_verified_vectors() {
        // Every vector is byte-verified against the real Go library via the
        // workspace parity harness (tools/go-probes/colorprofile).
        let vectors: &[(Profile, &str, &str)] = &[
        (TrueColor, "\x1b[31mred\x1b[0m", "\x1b[31mred\x1b[0m"),
        (Ansi256, "\x1b[31mred\x1b[0m", "\x1b[31mred\x1b[m"),
        (Ansi, "\x1b[31mred\x1b[0m", "\x1b[31mred\x1b[m"),
        (Ascii, "\x1b[31mred\x1b[0m", "\x1b[mred\x1b[m"),
        (NoTty, "\x1b[31mred\x1b[0m", "red"),
        (TrueColor, "\x1b[91;102mhello world\x1b[m", "\x1b[91;102mhello world\x1b[m"),
        (Ansi256, "\x1b[91;102mhello world\x1b[m", "\x1b[91;102mhello world\x1b[m"),
        (Ansi, "\x1b[91;102mhello world\x1b[m", "\x1b[91;102mhello world\x1b[m"),
        (Ascii, "\x1b[91;102mhello world\x1b[m", "\x1b[mhello world\x1b[m"),
        (NoTty, "\x1b[91;102mhello world\x1b[m", "hello world"),
        (TrueColor, "hello \x1b[38;5;196mworld\x1b[m", "hello \x1b[38;5;196mworld\x1b[m"),
        (Ansi256, "hello \x1b[38;5;196mworld\x1b[m", "hello \x1b[38;5;196mworld\x1b[m"),
        (Ansi, "hello \x1b[38;5;196mworld\x1b[m", "hello \x1b[91mworld\x1b[m"),
        (Ascii, "hello \x1b[38;5;196mworld\x1b[m", "hello \x1b[mworld\x1b[m"),
        (NoTty, "hello \x1b[38;5;196mworld\x1b[m", "hello world"),
        (TrueColor, "hello \x1b[38;2;255;133;55mworld\x1b[m", "hello \x1b[38;2;255;133;55mworld\x1b[m"),
        (Ansi256, "hello \x1b[38;2;255;133;55mworld\x1b[m", "hello \x1b[38;5;209mworld\x1b[m"),
        (Ansi, "hello \x1b[38;2;255;133;55mworld\x1b[m", "hello \x1b[91mworld\x1b[m"),
        (Ascii, "hello \x1b[38;2;255;133;55mworld\x1b[m", "hello \x1b[mworld\x1b[m"),
        (NoTty, "hello \x1b[38;2;255;133;55mworld\x1b[m", "hello world"),
        (TrueColor, "hello \x1b[48;5;196mworld\x1b[m", "hello \x1b[48;5;196mworld\x1b[m"),
        (Ansi256, "hello \x1b[48;5;196mworld\x1b[m", "hello \x1b[48;5;196mworld\x1b[m"),
        (Ansi, "hello \x1b[48;5;196mworld\x1b[m", "hello \x1b[101mworld\x1b[m"),
        (Ascii, "hello \x1b[48;5;196mworld\x1b[m", "hello \x1b[mworld\x1b[m"),
        (NoTty, "hello \x1b[48;5;196mworld\x1b[m", "hello world"),
        (TrueColor, "\x1b[1;38;5;204mhello \x1b[38;5;204mworld\x1b[m", "\x1b[1;38;5;204mhello \x1b[38;5;204mworld\x1b[m"),
        (Ansi256, "\x1b[1;38;5;204mhello \x1b[38;5;204mworld\x1b[m", "\x1b[1;38;5;204mhello \x1b[38;5;204mworld\x1b[m"),
        (Ansi, "\x1b[1;38;5;204mhello \x1b[38;5;204mworld\x1b[m", "\x1b[1;91mhello \x1b[91mworld\x1b[m"),
        (Ascii, "\x1b[1;38;5;204mhello \x1b[38;5;204mworld\x1b[m", "\x1b[1mhello \x1b[mworld\x1b[m"),
        (NoTty, "\x1b[1;38;5;204mhello \x1b[38;5;204mworld\x1b[m", "hello world"),
        (TrueColor, "\x1b[1;3;59mthe quick\x1b[m \x1b[1;2;48;5;52mbrown\x1b[49m fox \x1b[38;2;255;0;0mjumps\x1b[m over the lazy \x1b[31mdog\x1b[m\n", "\x1b[1;3;59mthe quick\x1b[m \x1b[1;2;48;5;52mbrown\x1b[49m fox \x1b[38;2;255;0;0mjumps\x1b[m over the lazy \x1b[31mdog\x1b[m\n"),
        (Ansi256, "\x1b[1;3;59mthe quick\x1b[m \x1b[1;2;48;5;52mbrown\x1b[49m fox \x1b[38;2;255;0;0mjumps\x1b[m over the lazy \x1b[31mdog\x1b[m\n", "\x1b[1;3;59mthe quick\x1b[m \x1b[1;2;48;5;52mbrown\x1b[49m fox \x1b[38;5;196mjumps\x1b[m over the lazy \x1b[31mdog\x1b[m\n"),
        (Ansi, "\x1b[1;3;59mthe quick\x1b[m \x1b[1;2;48;5;52mbrown\x1b[49m fox \x1b[38;2;255;0;0mjumps\x1b[m over the lazy \x1b[31mdog\x1b[m\n", "\x1b[1;3;59mthe quick\x1b[m \x1b[1;2;41mbrown\x1b[49m fox \x1b[91mjumps\x1b[m over the lazy \x1b[31mdog\x1b[m\n"),
        (Ascii, "\x1b[1;3;59mthe quick\x1b[m \x1b[1;2;48;5;52mbrown\x1b[49m fox \x1b[38;2;255;0;0mjumps\x1b[m over the lazy \x1b[31mdog\x1b[m\n", "\x1b[1;3mthe quick\x1b[m \x1b[1;2mbrown\x1b[m fox \x1b[mjumps\x1b[m over the lazy \x1b[mdog\x1b[m\n"),
        (NoTty, "\x1b[1;3;59mthe quick\x1b[m \x1b[1;2;48;5;52mbrown\x1b[49m fox \x1b[38;2;255;0;0mjumps\x1b[m over the lazy \x1b[31mdog\x1b[m\n", "the quick brown fox jumps over the lazy dog\n"),
        ];
        for (profile, input, expected) in vectors {
            let out = write_with(*profile, input);
            assert_eq!(&out, expected, "profile={profile:?} input={input:?}");
        }
    }

    #[test]
    fn test_writer_ascii() {
        // ASCII: colors stripped, non-color attrs preserved
        // (upstream: expectedAscii "\x1b[mhello world\x1b[m").
        let out = write_with(Profile::Ascii, "\x1b[31mhello world\x1b[m");
        assert_eq!(out, "\x1b[mhello world\x1b[m");
    }

    #[test]
    fn test_writer_plain_text() {
        let out = write_with(Profile::Ansi, "hello world");
        assert_eq!(out, "hello world");
    }
}
