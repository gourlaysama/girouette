pub(crate) mod option_color {
    use hex::FromHex;
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
                a if a.starts_with('#') => match <[u8; 3]>::from_hex(a.trim_start_matches('#')) {
                    Ok(c) => Ok(Some(Color::Rgb(c[0], c[1], c[2]))),
                    Err(_) => Err(de::Error::invalid_value(
                        de::Unexpected::Str(a),
                        &"a sharp '#' character followed by 6 hex digits",
                    )),
                },
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

pub(crate) mod segment_vec {
    use crate::segments::*;
    use serde::de::{self, Visitor};

    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Inner {
        Name(String),
        Struct(Segment),
    }

    struct SVisitor;

    impl<'de> Visitor<'de> for SVisitor {
        type Value = Vec<Segment>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a segment name, alone or as key to a mapping of options")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(inner) = seq.next_element()? {
                match inner {
                    Inner::Struct(s) => vec.push(s),
                    Inner::Name(name) => {
                        vec.push(match name.as_ref() {
                            "instant" => Segment::Instant(Instant::default()),
                            "location_name" => Segment::LocationName(LocationName::default()),
                            "temperature" => Segment::Temperature(Temperature::default()),
                            "weather_icon" => Segment::WeatherIcon(WeatherIcon::default()),
                            "weather_description" => {
                                Segment::WeatherDescription(WeatherDescription::default())
                            }
                            "wind_speed" => Segment::WindSpeed(WindSpeed::default()),
                            "humidity" => Segment::Humidity(Humidity::default()),
                            "rain" => Segment::Rain(Rain::default()),
                            "pressure" => Segment::Pressure(Pressure::default()),
                            "cloud_cover" => Segment::CloudCover(CloudCover::default()),
                            "daily_forecast" => Segment::DailyForecast(DailyForecast::default()),
                            "hourly_forecast" => Segment::HourlyForecast(HourlyForecast::default()),
                            "alerts" => Segment::Alerts(Alerts::default()),
                            "daytime" => Segment::DayTime(DayTime::default()),
                            "pollution" => Segment::Pollution(Pollution::default()),
                            a => {
                                return Err(de::Error::unknown_variant(
                                    a,
                                    &[
                                        "instant",
                                        "location_name",
                                        "temperature",
                                        "weather_icon",
                                        "weather_description",
                                        "wind_speed",
                                        "humidity",
                                        "rain",
                                        "pressure",
                                        "daily_forecast",
                                        "hourly_forecast",
                                        "alerts",
                                        "daytime",
                                        "pollution",
                                    ],
                                ))
                            }
                        });
                    }
                }
            }

            Ok(vec)
        }
    }

    pub(crate) fn deserialize<'de, D>(d: D) -> Result<Vec<Segment>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_seq(SVisitor)
    }
}
