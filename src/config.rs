use crate::{segments::*, DisplayMode, Location};
use serde::{Deserialize, Serialize};
use termcolor::{Color, ColorSpec};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ProgramConfig {
    pub key: Option<String>,

    pub location: Option<Location>,

    #[serde(flatten)]
    pub display_config: DisplayConfig,
}

impl Default for ProgramConfig {
    fn default() -> Self {
        ProgramConfig {
            key: None,
            location: None,
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

pub(crate) mod option_color {
    use serde::de::{self, SeqAccess, Visitor};
    use serde::ser::SerializeTuple;
    use termcolor::Color;
    const FIELDS: &[&str] = &[
        "black", "blue", "green", "red", "cyan", "magenta", "yellow", "white",
    ];

    struct OVisitor;

    impl<'de> Visitor<'de> for OVisitor {
        type Value = Option<Color>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a color string, an ansi int value or a triple of RGB values")
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<Color>, E>
        where
            E: de::Error,
        {
            match value {
                "black" => Ok(Some(Color::Black)),
                "blue" => Ok(Some(Color::Blue)),
                "green" => Ok(Some(Color::Green)),
                "red" => Ok(Some(Color::Red)),
                "cyan" => Ok(Some(Color::Cyan)),
                "magenta" => Ok(Some(Color::Magenta)),
                "yellow" => Ok(Some(Color::Yellow)),
                "white" => Ok(Some(Color::White)),
                _ => Err(de::Error::unknown_field(value, FIELDS)),
            }
        }

        fn visit_i64<E>(self, value: i64) -> Result<Option<Color>, E>
        where
            E: de::Error,
        {
            Ok(Some(Color::Ansi256(value as u8)))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Option<Color>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let r = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let g = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;
            let b = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(2, &self))?;

            Ok(Some(Color::Rgb(r, g, b)))
        }
    }

    pub(crate) fn deserialize<'de, D>(d: D) -> Result<Option<Color>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_any(OVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(crate) fn serialize<S>(c: &Option<Color>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match c {
            Some(ref value) => match value {
                Color::Black => ser.serialize_str("black"),
                Color::Blue => ser.serialize_str("blue"),
                Color::Green => ser.serialize_str("green"),
                Color::Red => ser.serialize_str("red"),
                Color::Cyan => ser.serialize_str("cyan"),
                Color::Magenta => ser.serialize_str("magenta"),
                Color::Yellow => ser.serialize_str("yellow"),
                Color::White => ser.serialize_str("white"),
                Color::Ansi256(u) => ser.serialize_u8(*u),
                Color::Rgb(r, g, b) => {
                    let mut tp = ser.serialize_tuple(3)?;
                    tp.serialize_element(r)?;
                    tp.serialize_element(g)?;
                    tp.serialize_element(b)?;
                    tp.end()
                }
                Color::__Nonexhaustive => unreachable!(),
            },
            None => ser.serialize_none(),
        }
    }
}

pub(crate) mod option_color_spec {
    use crate::config::FakeColorSpec;
    use termcolor::ColorSpec;

    pub(crate) fn deserialize<'de, D>(d: D) -> Result<Option<ColorSpec>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        FakeColorSpec::deserialize(d).map(Some)
    }

    pub(crate) fn serialize<S>(c: &Option<ColorSpec>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match c {
            Some(ref value) => FakeColorSpec::serialize(value, ser),
            None => ser.serialize_none(),
        }
    }
}

pub(crate) mod scaled_color {
    use serde::de::{self, Visitor};

    struct SVisitor;

    impl<'de> Visitor<'de> for SVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("either 'scaled' or a style definition")
        }

        fn visit_str<E>(self, value: &str) -> Result<(), E>
        where
            E: de::Error,
        {
            match value {
                "scaled" => Ok(()),
                _ => Err(de::Error::invalid_value(
                    de::Unexpected::Str(value),
                    &"scaled",
                )),
            }
        }
    }

    pub(crate) fn deserialize<'de, D>(d: D) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_str(SVisitor)
    }

    pub(crate) fn serialize<S>(ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ser.serialize_str("scaled")
    }
}
