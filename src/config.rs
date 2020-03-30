use crate::{segments::*, serde_utils::*, DisplayMode, Location};
use serde::{Deserialize, Serialize};
use termcolor::{Color, ColorSpec};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ProgramConfig {
    pub key: Option<String>,

    pub location: Option<Location>,

    pub cache: Option<String>,

    #[serde(flatten)]
    pub display_config: DisplayConfig,
}

impl Default for ProgramConfig {
    fn default() -> Self {
        ProgramConfig {
            key: None,
            location: None,
            cache: None,
            display_config: DisplayConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplayConfig {
    #[serde(with = "FakeColorSpec")]
    pub base_style: ColorSpec,

    pub separator: String,

    pub display_mode: DisplayMode,

    #[serde(deserialize_with = "segment_vec::deserialize")]
    pub segments: Vec<Segment>,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            base_style: ColorSpec::default(),
            separator: "  ".to_owned(),
            display_mode: DisplayMode::Unicode,
            segments: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(remote = "ColorSpec")]
pub struct FakeColorSpec {
    #[serde(default, rename = "fg", getter = "fg_copied", with = "option_color")]
    fg_color: Option<Color>,
    #[serde(default, rename = "bg", getter = "bg_copied", with = "option_color")]
    bg_color: Option<Color>,
    #[serde(default, getter = "ColorSpec::bold")]
    bold: bool,
    #[serde(default, getter = "ColorSpec::intense")]
    intense: bool,
    #[serde(default, getter = "ColorSpec::underline")]
    underline: bool,
    #[serde(default, getter = "ColorSpec::italic")]
    italic: bool,
    #[serde(default, getter = "ColorSpec::reset")]
    reset: bool,
}

impl From<FakeColorSpec> for ColorSpec {
    fn from(fc: FakeColorSpec) -> ColorSpec {
        let mut c = ColorSpec::default();
        c.set_fg(fc.fg_color)
            .set_bg(fc.bg_color)
            .set_bold(fc.bold)
            .set_intense(fc.intense)
            .set_underline(fc.underline)
            .set_italic(fc.italic)
            .set_reset(fc.reset);

        c
    }
}

fn fg_copied(c: &ColorSpec) -> Option<Color> {
    c.fg().copied()
}

fn bg_copied(c: &ColorSpec) -> Option<Color> {
    c.bg().copied()
}
