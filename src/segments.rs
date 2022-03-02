use crate::api::Response;
use crate::{api::current::Wind, DisplayMode, WindType};
use crate::{config::*, serde_utils::*, QueryKind, UnitMode};
use anyhow::*;
use chrono::{Datelike, FixedOffset, Locale, TimeZone, Utc};
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::TryInto;
use std::io::Write;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

macro_rules! display_print {
    ($out:ident, $display:expr, $a:expr, $b:expr, $c:expr) => {
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

struct RenderConf<'a> {
    base_style: &'a ColorSpec,
    display_mode: DisplayMode,
    locale: Locale,
    units: UnitMode,
}

impl Renderer {
    pub fn from(display_config: &DisplayConfig) -> Self {
        let mut display_config = display_config.clone();

        // reset stays false for segments but we hardcode it to true
        // for the base style. TODO: find a better way to do this
        display_config.base_style.set_reset(true);

        Renderer { display_config }
    }

    pub fn render(
        &mut self,
        out: &mut StandardStream,
        resp: &Response,
        language: Option<&str>,
    ) -> Result<()> {
        if self.display_config.segments.is_empty() {
            warn!("there are not segments to display!");
            return Ok(());
        }

        out.set_color(&self.display_config.base_style)?;

        let env_locale = std::env::var("LANG").ok();
        // clippy 1.57 wrongly warns about this, see https://github.com/rust-lang/rust-clippy/pull/7639#issuecomment-1050340564
        // and the corresponding PR for context. Can be remove when MSRV is bumped.
        #[allow(clippy::or_fun_call)]
        let locale = language
            .or(env_locale.as_deref())
            .and_then(|l| {
                let l = if let Some(s) = l.split_once('.') {
                    s.0
                } else {
                    l
                };
                l.try_into().map_err(|_| {
                warn!("unknown locale: {}; ensure it has the shape 'aa_AA', e.g. ja_JP, en_US", l);
            }).ok()
            })
            .unwrap_or_else(|| "en_US".try_into().unwrap());

        let conf = RenderConf {
            base_style: &self.display_config.base_style,
            display_mode: self.display_config.display_mode,
            locale,
            units: self.display_config.units,
        };

        let mut status = self.display_config.segments[0].render(out, &conf, resp)?;
        for s in self.display_config.segments[1..].iter() {
            out.set_color(&self.display_config.base_style)?;
            if let RenderStatus::Rendered = status {
                write!(out, "{}", self.display_config.separator)?;
            }
            status = s.render(out, &conf, resp)?;
        }

        out.reset()?;

        Ok(())
    }

    pub fn display_kind(&self) -> Result<QueryKind> {
        let mut current = false;
        let mut forecast = false;
        for s in &self.display_config.segments {
            if s.is_forecast() {
                forecast = true;
            } else {
                current = true;
            }
        }

        match (current, forecast) {
            (true, true) => Ok(QueryKind::Both),
            (true, false) => Ok(QueryKind::Current),
            (false, true) => Ok(QueryKind::ForeCast),
            (false, false) => bail!("there is no weather info to display"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    Snow(Snow),
    Pressure(Pressure),
    CloudCover(CloudCover),
    DailyForecast(DailyForecast),
    HourlyForecast(HourlyForecast),
    Alerts(Alerts),
    DayTime(DayTime),
}

impl Segment {
    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        match self {
            Segment::Instant(i) => i.render(out, conf, resp),
            Segment::LocationName(i) => i.render(out, conf, resp),
            Segment::Temperature(i) => i.render(out, conf, resp),
            Segment::WeatherIcon(i) => i.render(out, conf, resp),
            Segment::WeatherDescription(i) => i.render(out, conf, resp),
            Segment::WindSpeed(i) => i.render(out, conf, resp),
            Segment::Humidity(i) => i.render(out, conf, resp),
            Segment::Rain(i) => i.render(out, conf, resp),
            Segment::Snow(i) => i.render(out, conf, resp),
            Segment::Pressure(i) => i.render(out, conf, resp),
            Segment::CloudCover(c) => c.render(out, conf, resp),
            Segment::DailyForecast(c) => c.render(out, conf, resp),
            Segment::HourlyForecast(c) => c.render(out, conf, resp),
            Segment::Alerts(c) => c.render(out, conf, resp),
            Segment::DayTime(c) => c.render(out, conf, resp),
        }
    }

    pub fn is_forecast(&self) -> bool {
        matches!(
            self,
            Segment::DailyForecast(_) | Segment::HourlyForecast(_) | Segment::Alerts(_)
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        _conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let resp = resp.as_current()?;
        let timezone = resp.timezone;
        let dt = resp.dt;

        let source_date = FixedOffset::east(timezone).timestamp(dt, 0);

        if let Some(ref style) = self.style {
            out.set_color(style)?;
        }

        write!(out, "{}", source_date.format(&self.date_format))?;

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LocationName {
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl LocationName {
    pub fn new() -> Self {
        Default::default()
    }

    fn render(
        &self,
        out: &mut StandardStream,
        _conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let name = &resp.as_current()?.name;

        if let Some(ref style) = self.style {
            out.set_color(style)?;
        }

        write!(out, "{}", name)?;

        Ok(RenderStatus::Rendered)
    }
}

const TEMP_COLORS: [u8; 57] = [
    57, 63, 63, 63, 27, 27, 27, 33, 33, 33, 39, 39, 39, 45, 45, 45, 51, 51, 50, 50, 49, 49, 48, 48,
    47, 47, 46, 46, 46, 82, 82, 82, 118, 118, 118, 154, 154, 154, 190, 190, 190, 226, 226, 226,
    220, 220, 220, 214, 214, 214, 208, 208, 208, 202, 202, 202, 196,
];

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Temperature {
    pub display_mode: Option<DisplayMode>,
    pub feels_like: bool,
    pub min_max: bool,
    pub style: ScaledColor,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let resp = resp.as_current()?;
        let temp = resp.main.temp;
        let feels_like = resp.main.feels_like;
        let temp_min = resp.main.temp_min;
        let temp_max = resp.main.temp_max;

        let display_mode = self.display_mode.unwrap_or(conf.display_mode);

        if self.min_max {
            display_print!(out, display_mode, " \u{f175}", " \u{2b07}\u{fe0f} ", " (m ");
            display_temp(&self.style, out, temp_min, conf.base_style, conf.units)?;
            display_print!(
                out,
                display_mode,
                " \u{e350} ",
                " \u{1f321}\u{fe0f} ",
                " T "
            );
            display_temp(&self.style, out, temp, conf.base_style, conf.units)?;
            display_print!(out, display_mode, " \u{f176}", " \u{2b06}\u{fe0f} ", " M ");
            display_temp(&self.style, out, temp_max, conf.base_style, conf.units)?;
            if let DisplayMode::Ascii = display_mode {
                write!(out, ")")?;
            }
        } else {
            display_print!(out, display_mode, "\u{e350} ", "\u{1f321}\u{fe0f} ", "T ");
            display_temp(&self.style, out, temp, conf.base_style, conf.units)?;
        }
        if self.feels_like {
            write!(out, " (feels ")?;
            display_temp(&self.style, out, feels_like, conf.base_style, conf.units)?;
            write!(out, ")")?;
        }

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct WeatherIcon {
    pub display_mode: Option<DisplayMode>,
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl WeatherIcon {
    fn render_icon(
        out: &mut StandardStream,
        display_mode: DisplayMode,
        units: UnitMode,
        style: &Option<ColorSpec>,
        night: bool,
        wind: Option<&Wind>,
        id: u16,
    ) -> Result<RenderStatus> {
        if let Some(ref style) = style {
            out.set_color(style)?;
        }

        display_print!(
            out,
            display_mode,
            {
                let wind_type = wind.map_or(WindType::Low, |w| {
                    let speed = match units {
                        UnitMode::Metric => w.speed * 3.6,
                        _ => w.speed,
                    };
                    get_wind_type(speed, units)
                });

                get_icon(id, night, &wind_type)
            },
            format!("{}\u{fe0f}", get_unicode(id, night)),
            { "" }
        );

        if let DisplayMode::Ascii = display_mode {
            Ok(RenderStatus::Empty)
        } else {
            Ok(RenderStatus::Rendered)
        }
    }

    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let resp = resp.as_current()?;
        let sunset = resp.sys.sunset;
        let sunrise = resp.sys.sunrise;
        let wind = resp.wind.as_ref();
        let id = resp.weather[0].id;

        if let DisplayMode::Ascii = conf.display_mode {
            warn!("no weather icon to display in ascii mode!");
        }

        let now = Utc.timestamp(resp.dt, 0);
        let night = now >= Utc.timestamp(sunset, 0) || now <= Utc.timestamp(sunrise, 0);

        WeatherIcon::render_icon(
            out,
            conf.display_mode,
            conf.units,
            &self.style,
            night,
            wind,
            id,
        )
    }
}

fn get_wind_type(speed: f32, units: UnitMode) -> WindType {
    match units {
        UnitMode::Standard => {
            if speed >= 9.722_222_f32 {
                WindType::High
            } else if speed >= 5.555_555_3_f32 {
                WindType::Mid
            } else {
                WindType::Low
            }
        }
        UnitMode::Metric => {
            if speed >= 35f32 {
                WindType::High
            } else if speed >= 20_f32 {
                WindType::Mid
            } else {
                WindType::Low
            }
        }
        UnitMode::Imperial => {
            if speed >= 21.747_986_f32 {
                WindType::High
            } else if speed >= 12.427_42_f32 {
                WindType::Mid
            } else {
                WindType::Low
            }
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct WeatherDescription {
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl WeatherDescription {
    fn render(
        &self,
        out: &mut StandardStream,
        _conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let description = &resp.as_current()?.weather[0].description;

        let description = if let Some(d) = description {
            d
        } else {
            return Ok(RenderStatus::Empty);
        };

        if let Some(ref style) = self.style {
            out.set_color(style)?;
        }

        write!(out, "{}", description)?;

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

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
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
        conf: &RenderConf,
    ) -> Result<()> {
        let (icons, fallback) = match conf.display_mode {
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
        if let DisplayMode::Unicode = conf.display_mode {
            write!(stdout, "{}\u{fe0f}", icon)?;
        } else {
            write!(stdout, "{}", icon)?;
        }

        let speed = match conf.units {
            UnitMode::Metric => wind.speed * 3.6,
            _ => wind.speed,
        };

        if let WindType::High = get_wind_type(speed, conf.units) {
            display_print!(stdout, conf.display_mode, "\u{e34b} ", " \u{1f32c} ", "");
        }

        match &self.style {
            ScaledColor::Scaled => {
                let speed_color_idx = (speed.floor() as usize).min(WIND_COLORS.len() - 1).max(0);
                let mut tmp_style = conf.base_style.clone();
                stdout.set_color(
                    tmp_style.set_fg(Some(Color::Ansi256(WIND_COLORS[speed_color_idx]))),
                )?;
            }
            ScaledColor::Spec(Some(style)) => {
                stdout.set_color(style)?;
            }
            _ => {}
        };
        write!(stdout, " {:.1}", speed)?;
        stdout.set_color(conf.base_style)?;
        let unit = match conf.units {
            UnitMode::Standard => "m/s",
            UnitMode::Metric => "km/h",
            UnitMode::Imperial => "mph",
        };
        write!(stdout, " {}", unit)?;

        Ok(())
    }

    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let wind = resp.as_current()?.wind.as_ref();

        if let Some(w) = wind {
            self.display_wind(out, w, conf)?;
            Ok(RenderStatus::Rendered)
        } else {
            Ok(RenderStatus::Empty)
        }
    }
}

const HUMIDITY_COLORS: [u8; 11] = [220, 226, 190, 118, 82, 46, 48, 50, 51, 45, 39];

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
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
    ) -> Result<()> {
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
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let humidity = resp.as_current()?.main.humidity;

        self.display_humidity(out, humidity, conf.base_style, conf.display_mode)?;

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Rain {
    pub display_mode: Option<DisplayMode>,
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl Rain {
    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let rain = resp.as_current()?.rain.as_ref();

        if let Some(r) = rain {
            if let Some(mm) = r.one_h.or(r.three_h) {
                display_print!(out, conf.display_mode, "\u{e371}", "\u{2614}", "R");
                if let Some(ref style) = self.style {
                    out.set_color(style)?;
                }
                write!(out, " {:.1} ", mm)?;
                out.set_color(conf.base_style)?;
                write!(out, "mm/h")?;

                return Ok(RenderStatus::Rendered);
            }
        }

        debug!("did not receive rain data; doing nothing");
        Ok(RenderStatus::Empty)
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Snow {
    pub display_mode: Option<DisplayMode>,
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl Snow {
    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let snow = resp.as_current()?.snow.as_ref();

        if let Some(r) = snow {
            if let Some(mm) = r.one_h.or(r.three_h) {
                display_print!(out, conf.display_mode, "\u{f2dc}", "\u{2744}\u{fe0f}", "S");
                if let Some(ref style) = self.style {
                    out.set_color(style)?;
                }
                write!(out, " {:.1} ", mm)?;
                out.set_color(conf.base_style)?;
                write!(out, "mm/h")?;

                return Ok(RenderStatus::Rendered);
            }
        }

        debug!("did not receive snow data; doing nothing");
        Ok(RenderStatus::Empty)
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Pressure {
    pub display_mode: Option<DisplayMode>,
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl Pressure {
    fn display_pressure(
        &self,
        stdout: &mut StandardStream,
        pressure: u16,
        base_style: &ColorSpec,
        display_mode: DisplayMode,
    ) -> Result<()> {
        display_print!(stdout, display_mode, "\u{e372}", "P", "P");

        if let Some(ref style) = self.style {
            stdout.set_color(style)?;
        }
        write!(stdout, " {}", pressure)?;
        stdout.set_color(base_style)?;
        write!(stdout, " hPa")?;

        Ok(())
    }

    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let pressure = resp.as_current()?.main.pressure;

        self.display_pressure(out, pressure, conf.base_style, conf.display_mode)?;

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct CloudCover {
    pub display_mode: Option<DisplayMode>,
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl CloudCover {
    fn display_cover(
        &self,
        stdout: &mut StandardStream,
        cloud_cover: u16,
        base_style: &ColorSpec,
        display_mode: DisplayMode,
    ) -> Result<()> {
        display_print!(stdout, display_mode, "\u{e33d}", "\u{2601}\u{fe0f}", "C");

        if let Some(ref style) = self.style {
            stdout.set_color(style)?;
        }
        write!(stdout, " {}", cloud_cover)?;
        stdout.set_color(base_style)?;
        write!(stdout, " %")?;

        Ok(())
    }

    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let clouds = resp.as_current()?.clouds.as_ref();

        if let Some(clouds) = clouds {
            self.display_cover(out, clouds.all, conf.base_style, conf.display_mode)?;
        }

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DailyForecast {
    pub display_mode: Option<DisplayMode>,
    pub temp_style: ScaledColor,
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
    pub days: u8,
}

impl Default for DailyForecast {
    fn default() -> Self {
        Self {
            display_mode: Default::default(),
            temp_style: Default::default(),
            style: Default::default(),
            days: 3,
        }
    }
}

impl DailyForecast {
    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let resp = resp.as_forecast()?;
        let daily = resp.daily.as_deref().unwrap_or_default();
        let timezone = FixedOffset::east(resp.timezone_offset);
        let mut first = true;
        out.set_color(conf.base_style)?;

        let end = daily.len().min(1 + self.days as usize);

        for i in 1..end {
            let day = &daily[i as usize];

            let dt = day.dt;

            if let crate::api::one_call::Temperature::Values(ref t) = day.temp {
                if first {
                    write!(out, " ")?;
                    first = false;
                } else {
                    write!(out, "   ")?;
                }
                let source_date = timezone.timestamp(dt, 0);

                write!(out, "{} ", source_date.format_localized("%a", conf.locale))?;

                let wind = Wind {
                    speed: day.wind_speed,
                    deg: day.wind_deg,
                    gale: day.wind_gust,
                };

                WeatherIcon::render_icon(
                    out,
                    conf.display_mode,
                    conf.units,
                    &self.style,
                    false,
                    Some(&wind),
                    day.weather[0].id,
                )?;
                display_print!(out, conf.display_mode, "  ", " ", "");

                display_temp(&self.temp_style, out, t.day, conf.base_style, conf.units)?;

                out.set_color(conf.base_style)?;
            }
        }

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HourlyForecast {
    pub display_mode: Option<DisplayMode>,
    pub temp_style: ScaledColor,
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
    pub step: u8,
    pub hours: u8,
}

impl Default for HourlyForecast {
    fn default() -> Self {
        Self {
            display_mode: Default::default(),
            temp_style: Default::default(),
            style: Default::default(),
            step: 2,
            hours: 3,
        }
    }
}

impl HourlyForecast {
    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let resp = resp.as_forecast()?;
        let hourly = resp.hourly.as_deref().unwrap_or_default();
        let timezone = FixedOffset::east(resp.timezone_offset);
        let current_instant = Utc.timestamp(resp.current.dt, 0);
        let mut first = true;
        out.set_color(conf.base_style)?;

        let hours = self.hours as usize;
        let step = 1.max(self.step as usize);
        let end = hourly.len();

        let mut i = 0;
        while i * step + 1 < end && i < hours {
            let hour = &hourly[i * step + 1];

            let dt = hour.dt;

            if let crate::api::one_call::Temperature::Value(t) = hour.temp {
                if first {
                    write!(out, " ")?;
                    first = false;
                } else {
                    write!(out, "   ")?;
                }
                let instant = timezone.timestamp(dt, 0);
                write!(out, "{}h ", instant.format("%k"))?;

                let wind = Wind {
                    speed: hour.wind_speed,
                    deg: hour.wind_deg,
                    gale: hour.wind_gust,
                };

                let night = if instant.day() != current_instant.day() {
                    // use current-day sunrise/sunset to estimate if time tomorrow is day/night;
                    // it's close enough not to matter, considering we are using 1h increments
                    if let (Some(sunset), Some(sunrise)) =
                        (resp.current.sunset, resp.current.sunrise)
                    {
                        let sunrise_dt = timezone.timestamp(sunrise, 0).time();
                        let sunset_dt = timezone.timestamp(sunset, 0).time();
                        let date = instant.date();
                        instant >= date.and_time(sunset_dt).unwrap_or(instant)
                            || instant <= date.and_time(sunrise_dt).unwrap_or(instant)
                    } else {
                        false
                    }
                } else if let (Some(sunset), Some(sunrise)) =
                    (resp.current.sunset, resp.current.sunrise)
                {
                    instant >= Utc.timestamp(sunset, 0) || instant <= Utc.timestamp(sunrise, 0)
                } else {
                    false
                };

                WeatherIcon::render_icon(
                    out,
                    conf.display_mode,
                    conf.units,
                    &self.style,
                    night,
                    Some(&wind),
                    hour.weather[0].id,
                )?;
                display_print!(out, conf.display_mode, "  ", " ", "");

                display_temp(&self.temp_style, out, t, conf.base_style, conf.units)?;

                out.set_color(conf.base_style)?;
            }

            i += 1;
        }

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Alerts {
    pub display_mode: Option<DisplayMode>,
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
    pub description: bool,
    pub sender: bool,
}

impl Alerts {
    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let resp = resp.as_forecast()?;
        let alerts = resp.alerts.as_deref().unwrap_or_default();
        let timezone = resp.timezone_offset;

        for (i, a) in alerts.iter().enumerate() {
            if alerts.len() != 1 {
                write!(out, "{}. ", i + 1)?;
            }

            let mut seen_tags = HashSet::new();
            for t in &a.tags {
                let t = t.to_ascii_lowercase();
                if !seen_tags.contains(&t) {
                    match t.as_str() {
                        "flood" => {
                            display_print!(
                                out,
                                conf.display_mode,
                                "\u{e375}  ",
                                "\u{1f4a7}\u{fe0f}  ",
                                ""
                            )
                        }
                        "wind" => display_print!(
                            out,
                            conf.display_mode,
                            "\u{e34b}  ",
                            "\u{1f32c}\u{fe0f}  ",
                            ""
                        ),
                        "rain" => display_print!(
                            out,
                            conf.display_mode,
                            "\u{e318}  ",
                            "\u{1f327}\u{fe0f}  ",
                            ""
                        ),
                        "thunderstorm" => display_print!(
                            out,
                            conf.display_mode,
                            "\u{e31d}  ",
                            "\u{1f329}\u{fe0f}  ",
                            ""
                        ),
                        "fog" => display_print!(
                            out,
                            conf.display_mode,
                            "\u{e313}  ",
                            "\u{1f32b}\u{fe0f}  ",
                            ""
                        ),
                        "coastal event" => display_print!(
                            out,
                            conf.display_mode,
                            "\u{1f3d6}\u{fe0f}  ",
                            "\u{1f3d6}\u{fe0f}  ",
                            ""
                        ),
                        "extreme temperature value" => display_print!(
                            out,
                            conf.display_mode,
                            "\u{e350} ",
                            "\u{1f321}\u{fe0f}  ",
                            ""
                        ),
                        "snow/ice" => display_print!(
                            out,
                            conf.display_mode,
                            "\u{e36f} ",
                            "\u{2744}\u{fe0f}  ",
                            ""
                        ),
                        a => {
                            debug!("no icon for tag: {}; ignoring", a);
                        }
                    };
                    seen_tags.insert(t);
                }
            }

            if let Some(ref style) = self.style {
                out.set_color(style)?;
                write!(out, "{}", a.event)?;
                out.set_color(conf.base_style)?;
            } else {
                write!(out, "{}", a.event)?;
            }

            if self.description {
                write!(out, ": {}", a.description.replace('\n', " "))?;
            }

            if self.sender {
                write!(out, " ({})", a.sender_name)?;
            }

            write!(out, " ")?;

            let start_date = FixedOffset::east(timezone).timestamp(a.start, 0);
            let end_date = FixedOffset::east(timezone).timestamp(a.end, 0);
            let now = Utc::now();

            if start_date > now {
                display_print!(
                    out,
                    conf.display_mode,
                    "\u{f071} \u{f176}",
                    "\u{26a0}\u{fe0f} \u{2b06}\u{fe0f} ",
                    ""
                );
                let format = if start_date.date() == now.date() {
                    "%R"
                } else {
                    "%a %R"
                };
                write!(out, "{} ", start_date.format_localized(format, conf.locale))?;
            }

            if end_date > now {
                display_print!(
                    out,
                    conf.display_mode,
                    "\u{f071} \u{f175}",
                    "\u{26a0}\u{fe0f} \u{2b07}\u{fe0f} ",
                    ""
                );
                let format = if end_date.date() == now.date() {
                    "%R"
                } else {
                    "%a %R"
                };
                write!(out, "{} ", end_date.format_localized(format, conf.locale))?;
            }
        }

        Ok(RenderStatus::Rendered)
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DayTime {
    pub display_mode: Option<DisplayMode>,
    #[serde(with = "option_color_spec")]
    pub style: Option<ColorSpec>,
}

impl DayTime {
    fn render(
        &self,
        out: &mut StandardStream,
        conf: &RenderConf,
        resp: &Response,
    ) -> Result<RenderStatus> {
        let resp = resp.as_current()?;
        let timezone = resp.timezone;
        let sunrise = resp.sys.sunrise;
        let sunset = resp.sys.sunset;

        display_print!(
            out,
            conf.display_mode,
            "\u{e34c}  ",
            "\u{2600}\u{fe0f} \u{2b06}\u{fe0f} ",
            "S "
        );
        let sunrise_date = FixedOffset::east(timezone).timestamp(sunrise, 0);
        write!(out, "{} ", sunrise_date.format("%R"))?;

        display_print!(
            out,
            conf.display_mode,
            "\u{e34d}  ",
            "\u{2b07}\u{fe0f} ",
            "-> "
        );
        let sunset_date = FixedOffset::east(timezone).timestamp(sunset, 0);
        write!(out, "{}", sunset_date.format("%R"))?;

        Ok(RenderStatus::Rendered)
    }
}

fn display_temp(
    color_scale: &ScaledColor,
    out: &mut StandardStream,
    temp: f32,
    base_style: &ColorSpec,
    units: UnitMode,
) -> Result<()> {
    match color_scale {
        ScaledColor::Scaled => {
            let c = match units {
                UnitMode::Standard => (temp - 273.15),
                UnitMode::Metric => temp,
                UnitMode::Imperial => (temp - 32f32) * 0.555_555_6,
            };
            let temp_idx = (c.round() + 16f32).min(56f32).max(0f32) as usize;

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

    write!(out, "{:.1}", temp)?;
    out.set_color(base_style)?;

    match units {
        UnitMode::Standard => write!(out, " K")?,
        UnitMode::Metric => write!(out, " °C")?,
        UnitMode::Imperial => write!(out, " °F")?,
    };

    Ok(())
}

fn get_icon(id: u16, night: bool, wind_type: &WindType) -> &'static str {
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
            if night {
                "\u{e32b}"
            } else {
                "\u{e30d}"
            }
        }
    }
}

fn get_unicode(id: u16, night: bool) -> &'static str {
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
        (_, 701) | (_, 711) | (_, 721) | (_, 731) | (_, 761) | (_, 741) | (_, 751) | (_, 762) => {
            "\u{1f32b}"
        }
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
            if night {
                "\u{263e}"
            } else {
                "\u{1f31e}"
            }
        }
    }
}
