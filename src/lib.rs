//! Cleanroom Rust port of upstream Go source file: `doc.go`
//! Upstream Target Tag / Version: `v0.4.3`
//!
//! <public-docs>
//! Downsample ANSI escape sequence colors and styles automatically based on
//! output, environment variables, and Terminfo databases.
//! </public-docs>

mod env;
mod writer;

pub use env::{detect, env, environ, terminfo, tmux, Environ};
pub use writer::{new_writer, Writer};

use charming_x_ansi::color::{ansi256_to_16, convert_16, convert_256};
use charming_x_ansi::style::Color;
use std::sync::OnceLock;

/// Profile is a color profile: NoTTY, Ascii, ANSI, ANSI256, or TrueColor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Profile {
    /// Unknown is a profile that represents the absence of a profile.
    #[default]
    Unknown,
    /// NoTTY is a profile with no terminal support.
    NoTty,
    /// ASCII is a profile with no color support.
    Ascii,
    /// ANSI is a profile with 16 colors (4-bit).
    Ansi,
    /// ANSI256 is a profile with 256 colors (8-bit).
    Ansi256,
    /// TrueColor is a profile with 16 million colors (24-bit).
    TrueColor,
}

impl Profile {
    /// String returns the string representation of a [Profile].
    pub fn string(&self) -> &'static str {
        match self {
            Profile::TrueColor => "TrueColor",
            Profile::Ansi256 => "ANSI256",
            Profile::Ansi => "ANSI",
            Profile::Ascii => "Ascii",
            Profile::NoTty => "NoTTY",
            Profile::Unknown => "Unknown",
        }
    }

    /// Convert transforms a given color to a color supported within the
    /// [Profile].
    pub fn convert(&self, c: Color) -> Option<Color> {
        if *self <= Profile::Ascii {
            return None;
        }
        if *self == Profile::TrueColor {
            // TrueColor is a passthrough.
            return Some(c);
        }

        // NOTE: upstream caches converted colors in a `sync.RWMutex` map
        // keyed by `color.Color`; the cache is a performance optimization
        // with no observable semantics, so it is omitted here.
        match c {
            Color::Basic(_) => Some(c),
            Color::Indexed(i) => {
                if *self == Profile::Ansi {
                    Some(Color::Basic(ansi256_to_16(i)))
                } else {
                    Some(c)
                }
            }
            Color::RGB(rgb) => match self {
                Profile::Ansi256 => Some(Color::Indexed(convert_256(rgb.r, rgb.g, rgb.b))),
                Profile::Ansi => Some(Color::Basic(convert_16(rgb.r, rgb.g, rgb.b))),
                _ => Some(c),
            },
            Color::Default => Some(c),
        }
    }
}

/// Convert16 converts a 256-color index to a 16-color ANSI color.
/// Re-exported for parity with the upstream `ansi.Convert16` usage.
pub fn convert16(c: u8) -> charming_x_ansi::color::BasicColor {
    ansi256_to_16(c)
}

/// Convert256 converts an RGB color to a 256-color index.
/// Re-exported for parity with the upstream `ansi.Convert256` usage.
pub fn convert256(r: u8, g: u8, b: u8) -> charming_x_ansi::color::IndexedColor {
    convert_256(r, g, b)
}

/// A global cache placeholder to keep parity with the upstream API shape.
#[allow(dead_code)]
fn cache() -> &'static OnceLock<()> {
    static CACHE: OnceLock<()> = OnceLock::new();
    &CACHE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_string() {
        assert_eq!(Profile::TrueColor.string(), "TrueColor");
        assert_eq!(Profile::Ansi256.string(), "ANSI256");
        assert_eq!(Profile::Ansi.string(), "ANSI");
        assert_eq!(Profile::Ascii.string(), "Ascii");
        assert_eq!(Profile::NoTty.string(), "NoTTY");
        assert_eq!(Profile::Unknown.string(), "Unknown");
    }

    #[test]
    fn test_convert() {
        let rgb = Color::RGB(charming_x_ansi::color::RGBColor { r: 255, g: 0, b: 0 });
        // TrueColor passthrough.
        assert_eq!(Profile::TrueColor.convert(rgb), Some(rgb));
        // ANSI256 downsampling.
        assert_eq!(Profile::Ansi256.convert(rgb), Some(Color::Indexed(196)));
        // ANSI downsampling.
        let c = Profile::Ansi.convert(rgb);
        assert_eq!(c, Some(Color::Basic(9)));
        // ASCII strips colors.
        assert_eq!(Profile::Ascii.convert(rgb), None);
        // NoTTY strips colors.
        assert_eq!(Profile::NoTty.convert(rgb), None);
        // Basic colors pass through.
        assert_eq!(
            Profile::Ansi.convert(Color::Basic(1)),
            Some(Color::Basic(1))
        );
        // Indexed to ANSI.
        assert_eq!(
            Profile::Ansi.convert(Color::Indexed(196)),
            Some(Color::Basic(9))
        );
    }
}
