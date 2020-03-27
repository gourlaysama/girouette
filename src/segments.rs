use crate::config::*;
use crate::{response::Wind, DisplayMode, WeatherResponse, WindType};
use chrono::{FixedOffset, TimeZone, Utc};
use failure::*;
use log::*;
use serde::{Deserialize, Serialize};
use std::io::Write;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

macro_rules! display_print {
    ($out:ident, $display:ident, $a:expr, $b:expr, $c:expr) => {
        match $display {
            DisplayMode::NerdFonts => write!($out, "{}", $a)?,
            DisplayMode::Unicode => write!($out, "{}", $b)?,
            DisplayMode::Ascii => write!($out, "{}", $c)?,
        }
    };
}

pub struct Renderer {
    pub display_config: DisplayConfig,
}

enum RenderStatus {
    Empty,
    Rendered,
}

impl Renderer {
    pub fn new(mut display_config: DisplayConfig) -> Self {
        // reset stays false for segments but we hardcode it to true
        // for the base style. TODO: find a better way to do this
        display_config.base_style.set_reset(true);

        Renderer { display_config }
    }

    pub fn render(
        &mut self,
        out: &mut StandardStream,
        resp: &WeatherResponse,
    ) -> Result<(), Error> {
        if self.display_config.segments.is_empty() {
            warn!("there are not segments to display!");
            return Ok(());
        }

        out.set_color(&self.display_config.base_style)?;

        let mut status = self.display_config.segments[0].render(
            out,
            &self.display_config.base_style,
            self.display_config.display_mode,
            &resp,
        )?;
        for s in self.display_config.segments[1..].iter() {
            out.set_color(&self.display_config.base_style)?;
            if let RenderStatus::Rendered = status {
                write!(out, "{}", self.display_config.separator)?;
            }
            status = s.render(
                out,
                &self.display_config.base_style,
                self.display_config.display_mode,
                &resp,
            )?;
        }

        out.reset()?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Segment {
    Instant(Instant),
    LocationName(LocationName),
    Temperature(Temperature),
    WeatherIcon(WeatherIcon),
    WeatherDescription(WeatherDescription),
    WindSpeed(WindSpeed),
    Humidity(Humidity),
    Rain(Rain),
    Pressure(Pressure),
}

impl Segment {
    fn render(
        &self,
        out: &mut StandardStream,
        base_style: &ColorSpec,
        display_mode: DisplayMode,
        resp: &WeatherResponse,
    ) -> Result<RenderStatus, Error> {
        match self {
            Segment::Instant(i) => i.render(out, base_style, display_mode, resp),
            Segment::LocationName(i) => i.render(out, base_style, display_mode, resp),
            Segment::Temperature(i) => i.render(out, base_style, display_mode, resp),
            Segment::WeatherIcon(i) => i.render(out, base_style, display_mode, resp),
            Segment::WeatherDescription(i) => i.render(out, base_style, display_mode, resp),
            Segment::WindSpeed(i) => i.render(out, base_style, display_mode, resp),
            Segment::Humidity(i) => i.render(out, base_style, display_mode, resp),
            Segment::Rain(i) => i.render(out, base_style, display_mode, resp),
            Segment::Pressure(i) => i.render(out, base_style, display_mode, resp),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Instant {
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
    pub date_format: String,
}

impl Default for Instant {
    fn default() -> Self {
        Instant {
            style: None,
            date_format: "%F %T %:z".to_owned(),
        }
    }
}

impl Instant {
    pub fn new(style: Option<ColorSpec>, date_format: String) -> Instant {
        Instant { style, date_format }
    }

    fn render(
        &self,
        out: &mut StandardStream,
        _: &ColorSpec,
        _: DisplayMode,
        resp: &WeatherResponse,
    ) -> Result<RenderStatus, Error> {
        let source_date = FixedOffset::east(resp.timezone).timestamp(resp.dt, 0);

        if let Some(ref style) = self.style {
            out.set_color(style)?;
        }

        write!(out, "{}", source_date.format(&self.date_format))?;

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LocationName {
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl Default for LocationName {
    fn default() -> Self {
        let mut style = ColorSpec::new();
        style.set_reset(false);
        style.set_fg(Some(Color::Blue)).set_intense(true);
        LocationName { style: Some(style) }
    }
}

impl LocationName {
    pub fn new() -> Self {
        Default::default()
    }

    fn render(
        &self,
        out: &mut StandardStream,
        _: &ColorSpec,
        _: DisplayMode,
        resp: &WeatherResponse,
    ) -> Result<RenderStatus, Error> {
        if let Some(ref style) = self.style {
            out.set_color(style)?;
        }

        write!(out, "{}", resp.name)?;

        Ok(RenderStatus::Rendered)
    }
}

const TEMP_COLORS: [u8; 57] = [
    57, 63, 63, 63, 27, 27, 27, 33, 33, 33, 39, 39, 39, 45, 45, 45, 51, 51, 50, 50, 49, 49, 48, 48,
    47, 47, 46, 46, 46, 82, 82, 82, 118, 118, 118, 154, 154, 154, 190, 190, 190, 226, 226, 226,
    220, 220, 220, 214, 214, 214, 208, 208, 208, 202, 202, 202, 196,
];

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Temperature {
    pub display_mode: Option<DisplayMode>,
    pub feels_like: bool,
    pub style: ScaledColor,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ScaledColor {
    #[serde(with = "scaled_color")]
    Scaled,
    #[serde(with = "option_color_spec")]
    Spec(Option<ColorSpec>),
}

impl Default for ScaledColor {
    fn default() -> Self {
        ScaledColor::Scaled
    }
}

impl Temperature {
    fn display_temp(
        &self,
        out: &mut StandardStream,
        temp: f32,
        base_style: &ColorSpec,
    ) -> Result<(), Error> {
        match &self.style {
            ScaledColor::Scaled => {
                let temp_idx = (temp.round() + 16f32).min(57f32).max(0f32) as usize;

                out.set_color(
                    base_style
                        .clone()
                        .set_fg(Some(Color::Ansi256(TEMP_COLORS[temp_idx])))
                        .set_bold(true),
                )?;
            }
            ScaledColor::Spec(Some(style)) => {
                out.set_color(style)?;
            }
            _ => {}
        }

        write!(out, " {:.1}", temp)?;
        out.set_color(base_style)?;
        write!(out, " °C")?;
        Ok(())
    }

    fn render(
        &self,
        out: &mut StandardStream,
        base_style: &ColorSpec,
        display_mode: DisplayMode,
        resp: &WeatherResponse,
    ) -> Result<RenderStatus, Error> {
        let display_mode = self.display_mode.unwrap_or(display_mode);

        display_print!(out, display_mode, "\u{e350}", "T", "T");

        self.display_temp(out, resp.main.temp, base_style)?;
        if self.feels_like {
            write!(out, " (feels")?;
            self.display_temp(out, resp.main.feels_like, base_style)?;
            write!(out, ")")?;
        }

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct WeatherIcon {
    pub display_mode: Option<DisplayMode>,
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl WeatherIcon {
    fn get_icon(&self, id: u16, night: bool, wind_type: &WindType) -> &'static str {
        match (night, id) {
            // thunderstorm + rain
            (true, 200..=209) => "\u{e32a}",
            (false, 200..=209) => "\u{e30f}",
            // thunderstorm
            (true, 210..=219) | (true, 221) => "\u{e332}",
            (false, 210..=219) | (false, 221) => "\u{e305}",
            // thunderstorm + sleet/drizzle
            (true, 230..=239) => "\u{e364}",
            (false, 230..=239) => "\u{e362}",
            // sprinkle
            (true, 300..=309) | (true, 310..=312) => "\u{e328}",
            (false, 300..=309) | (false, 310..=312) => "\u{e30b}",
            // rain
            (true, 500..=509) => "\u{e325}",
            (false, 500..=509) => "\u{e308}",
            // freezing rain
            (true, 511) => "\u{e321}",
            (false, 511) => "\u{e304}",
            // showers
            (true, 520..=529) | (true, 313..=319) | (true, 531) => "\u{e326}",
            (false, 520..=529) | (false, 313..=319) | (false, 531) => "\u{e309}",
            // snow
            (true, 600..=609) => "\u{e327}",
            (false, 600..=609) => "\u{e30a}",
            // sleet
            (true, 611..=615) => "\u{e3ac}",
            (false, 613..=615) => "\u{e3aa}",
            // rain/snow mix
            (true, 620..=629) | (true, 616) => "\u{e331}",
            (false, 620..=629) | (false, 616) => "\u{e306}",
            // mist
            (true, 701) => "\u{e320}",
            (false, 701) => "\u{e311}",
            // smoke
            (_, 711) => "\u{e35c}",
            // haze
            (false, 721) => "\u{e36b}",
            // dust
            (_, 731) | (_, 761) => "\u{e35d}",
            // fog
            (true, 741) => "\u{e346}",
            (false, 741) => "\u{e303}",
            // sandstorm
            (_, 751) => "\u{e37a}",
            // volcanic ash
            (_, 762) => "\u{e3c0}",
            // squalls
            (_, 771) => "\u{e34b}",
            // tornado
            (_, 781) => "\u{e351}",
            // clear
            (true, 800) => "\u{e32b}",
            (false, 800) => match wind_type {
                WindType::High => "\u{e37d}",
                WindType::Mid => "\u{e3bc}",
                WindType::Low => "\u{e30d}",
            },
            // clouds 25-50%
            (true, 801) => "\u{e379}",
            (false, 801) => "\u{e30c}",
            // clouds >=50%
            (true, 802..=809) => match wind_type {
                WindType::High => "\u{e31f}",
                WindType::Mid => "\u{e320}",
                WindType::Low => "\u{e37e}",
            },
            (false, 802..=809) => match wind_type {
                WindType::High => "\u{e300}",
                WindType::Mid => "\u{e301}",
                WindType::Low => "\u{e302}",
            },
            (a, b) => {
                debug!("no icon for (night: {}, code: {}); using fallback", a, b);
                "\u{e374}"
            }
        }
    }

    fn get_unicode(&self, id: u16, night: bool) -> &'static str {
        match (night, id) {
            // thunderstorm + rain
            (_, 200..=209) => "\u{26c8}",
            // thunderstorm
            (_, 210..=219) | (_, 221) | (_, 230..=239) => "\u{1f329}",
            // rain (all types)
            (true, 300..=309)
            | (true, 310..=312)
            | (true, 500..=509)
            | (true, 511)
            | (true, 520..=529)
            | (true, 313..=319)
            | (true, 531)
            | (true, 611..=615)
            | (true, 620..=629)
            | (true, 616) => "\u{1f327}",
            (false, 300..=309)
            | (false, 310..=312)
            | (false, 500..=509)
            | (false, 511)
            | (false, 520..=529)
            | (false, 313..=319)
            | (false, 531)
            | (false, 613..=615)
            | (false, 620..=629)
            | (false, 616) => "\u{1f326}",
            // snow
            (_, 600..=609) => "\u{1f328}",
            // mist/fog/smoke/haze/dust/sandstorm/ash
            (_, 701)
            | (_, 711)
            | (false, 721)
            | (_, 731)
            | (_, 761)
            | (_, 741)
            | (_, 751)
            | (_, 762) => "\u{1f32b}",
            // squalls
            (_, 771) => "\u{1f32c}",
            // tornado
            (_, 781) => "\u{1f32a}",
            // clear
            (true, 800) => "\u{263e}",
            (false, 800) => "\u{1f31e}",
            // clouds 25-50%
            (false, 801) => "\u{1f324}",
            // clouds >=50%
            (true, 801..=809) => "\u{2601}",
            (false, 802..=809) => "\u{26c5}",
            (a, b) => {
                debug!("no unicode for (night: {}, code: {}); using fallback", a, b);
                "\u{e374}"
            }
        }
    }

    fn render(
        &self,
        out: &mut StandardStream,
        _: &ColorSpec,
        display_mode: DisplayMode,
        resp: &WeatherResponse,
    ) -> Result<RenderStatus, Error> {
        if let Some(ref style) = self.style {
            out.set_color(style)?;
        }

        let now = Utc::now();
        let night =
            now >= Utc.timestamp(resp.sys.sunset, 0) || now <= Utc.timestamp(resp.sys.sunrise, 0);

        display_print!(
            out,
            display_mode,
            {
                let wind_type = resp
                    .wind
                    .as_ref()
                    .map_or(WindType::Low, |w| get_wind_type(w.speed));

                self.get_icon(resp.weather[0].id, night, &wind_type)
            },
            self.get_unicode(resp.weather[0].id, night),
            {
                warn!("no weather icon to display in ascii mode!");
                ""
            }
        );

        if let DisplayMode::Ascii = display_mode {
            Ok(RenderStatus::Empty)
        } else {
            Ok(RenderStatus::Rendered)
        }
    }
}

fn get_wind_type(speed: f32) -> WindType {
    if speed >= 35f32 {
        WindType::High
    } else if speed >= 20f32 {
        WindType::Mid
    } else {
        WindType::Low
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct WeatherDescription {
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl WeatherDescription {
    fn render(
        &self,
        out: &mut StandardStream,
        _: &ColorSpec,
        _: DisplayMode,
        resp: &WeatherResponse,
    ) -> Result<RenderStatus, Error> {
        if let Some(ref style) = self.style {
            out.set_color(style)?;
        }

        write!(out, "{}", resp.weather[0].description)?;

        Ok(RenderStatus::Rendered)
    }
}

const WIND_COLORS: [u8; 52] = [
    46, 46, 46, 82, 82, 82, 118, 118, 118, 154, 154, 154, 190, 190, 190, 226, 226, 226, 226, 220,
    220, 220, 220, 214, 214, 214, 214, 208, 208, 208, 208, 202, 202, 202, 202, 196, 196, 196, 196,
    160, 160, 160, 160, 124, 124, 124, 124, 127, 127, 127, 127, 129,
];

const WIND_DIR_ICONS: &str =
    "\u{e35a}\u{e359}\u{e35b}\u{e356}\u{e357}\u{e355}\u{e354}\u{e358}\u{e35a}";

const WIND_DIR_UNICODE: &str =
    "\u{2b07}\u{2199}\u{2b05}\u{2196}\u{2b06}\u{2197}\u{27a1}\u{2198}\u{2b07}";

const WIND_DIR_ASCII: &str = " S  SW W  NW N  NE E  SE S ";

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct WindSpeed {
    pub display_mode: Option<DisplayMode>,
    pub style: ScaledColor,
}

impl WindSpeed {
    fn display_wind(
        &self,
        stdout: &mut StandardStream,
        wind: &Wind,
        base_style: &ColorSpec,
        display_mode: DisplayMode,
    ) -> Result<(), Error> {
        let (icons, fallback) = match display_mode {
            DisplayMode::Ascii => (WIND_DIR_ASCII, ""),
            DisplayMode::Unicode => (WIND_DIR_UNICODE, ""),
            DisplayMode::NerdFonts => (WIND_DIR_ICONS, "\u{e3a9}"),
        };

        let icon = wind
            .deg
            .map(|deg| {
                let dir_idx = ((deg + 22.5) / 45f32).floor() as usize;
                &icons[3 * dir_idx..3 * dir_idx + 3]
            })
            .unwrap_or(fallback);
        write!(stdout, "{}", icon)?;

        let speed = wind.speed * 3.6;

        if let WindType::High = get_wind_type(speed) {
            display_print!(stdout, display_mode, "\u{e34b} ", " \u{1f32c} ", "");
        }

        match &self.style {
            ScaledColor::Scaled => {
                let speed_color_idx = speed.floor() as usize;
                let mut tmp_style = base_style.clone();
                stdout.set_color(
                    &tmp_style.set_fg(Some(Color::Ansi256(WIND_COLORS[speed_color_idx]))),
                )?;
            }
            ScaledColor::Spec(Some(style)) => {
                stdout.set_color(style)?;
            }
            _ => {}
        };
        write!(stdout, " {:.1}", speed)?;
        stdout.set_color(base_style)?;
        write!(stdout, " km/h")?;

        Ok(())
    }

    fn render(
        &self,
        out: &mut StandardStream,
        base_style: &ColorSpec,
        display_mode: DisplayMode,
        resp: &WeatherResponse,
    ) -> Result<RenderStatus, Error> {
        if let Some(w) = &resp.wind {
            self.display_wind(out, &w, base_style, display_mode)?;
            Ok(RenderStatus::Rendered)
        } else {
            Ok(RenderStatus::Empty)
        }
    }
}

const HUMIDITY_COLORS: [u8; 11] = [220, 226, 190, 118, 82, 46, 48, 50, 51, 45, 39];

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Humidity {
    pub display_mode: Option<DisplayMode>,
    pub style: ScaledColor,
}

impl Humidity {
    fn display_humidity(
        &self,
        stdout: &mut StandardStream,
        humidity: u8,
        base_style: &ColorSpec,
        display_mode: DisplayMode,
    ) -> Result<(), Error> {
        display_print!(stdout, display_mode, "\u{e373}", "H", "H");

        match &self.style {
            ScaledColor::Scaled => {
                let hum_idx = (humidity / 10) as usize;
                let mut tmp_style = base_style.clone();
                stdout
                    .set_color(tmp_style.set_fg(Some(Color::Ansi256(HUMIDITY_COLORS[hum_idx]))))?;
            }
            ScaledColor::Spec(Some(style)) => {
                stdout.set_color(style)?;
            }
            _ => {}
        };

        write!(stdout, " {}", humidity)?;
        stdout.set_color(base_style)?;
        write!(stdout, " %")?;
        Ok(())
    }

    fn render(
        &self,
        out: &mut StandardStream,
        base_style: &ColorSpec,
        display_mode: DisplayMode,
        resp: &WeatherResponse,
    ) -> Result<RenderStatus, Error> {
        self.display_humidity(out, resp.main.humidity, base_style, display_mode)?;

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Rain {
    pub display_mode: Option<DisplayMode>,
}

impl Rain {
    fn render(
        &self,
        out: &mut StandardStream,
        _: &ColorSpec,
        display_mode: DisplayMode,
        resp: &WeatherResponse,
    ) -> Result<RenderStatus, Error> {
        if let Some(r) = &resp.rain {
            if let Some(mm) = r.one_h.or(r.three_h) {
                display_print!(out, display_mode, "\u{e371}", "\u{2614}", "R");
                write!(out, " {:.1} mm/h  ", mm)?;

                return Ok(RenderStatus::Rendered);
            }
        }

        debug!("did not receive rain data; doing nothing");
        Ok(RenderStatus::Empty)
    }
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Pressure {
    pub display_mode: Option<DisplayMode>,
}

impl Pressure {
    fn display_pressure(
        &self,
        stdout: &mut StandardStream,
        pressure: u16,
        base_style: &ColorSpec,
        display_mode: DisplayMode,
    ) -> Result<(), Error> {
        display_print!(stdout, display_mode, "\u{e372}", "P", "P");

        let mut tmp_style = base_style.clone();
        stdout.set_color(tmp_style.set_fg(Some(Color::White)))?;
        write!(stdout, " {}", pressure)?;
        stdout.set_color(tmp_style.set_fg(None))?;
        write!(stdout, " hPa")?;

        Ok(())
    }

    fn render(
        &self,
        out: &mut StandardStream,
        base_style: &ColorSpec,
        display_mode: DisplayMode,
        resp: &WeatherResponse,
    ) -> Result<RenderStatus, Error> {
        self.display_pressure(out, resp.main.pressure, base_style, display_mode)?;

        Ok(RenderStatus::Rendered)
    }
}
