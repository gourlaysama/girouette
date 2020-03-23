use crate::{response::Wind, DisplayMode, WeatherResponse, WindType};
use chrono::{FixedOffset, TimeZone, Utc};
use std::io::Write;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

pub struct Renderer {
    segments: Vec<Segment>,
    pub base_style: ColorSpec,
    pub separator: String,
}

impl Renderer {
    pub fn new(segments: Vec<Segment>, base_style: ColorSpec, separator: &str) -> Self {
        Renderer {
            segments,
            base_style,
            separator: separator.to_owned(),
        }
    }

    pub fn render(
        &mut self,
        out: &mut StandardStream,
        resp: &WeatherResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.segments.is_empty() {
            return Ok(());
        }

        out.set_color(&self.base_style)?;

        self.segments[0]
            .inner
            .render(out, &self.base_style, &resp)?;
        for s in self.segments[1..].iter() {
            out.set_color(&self.base_style)?;
            write!(out, "{}", self.separator)?;
            s.inner.render(out, &self.base_style, &resp)?;
        }

        out.reset()?;

        Ok(())
    }
}

pub struct Segment {
    inner: Box<dyn SegmentContent>,
}

pub trait SegmentContent {
    fn render(
        &self,
        out: &mut StandardStream,
        base_style: &ColorSpec,
        resp: &WeatherResponse,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct Instant {
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

    pub fn build(self) -> Segment {
        Segment {
            inner: Box::new(self),
        }
    }
}

impl SegmentContent for Instant {
    fn render(
        &self,
        out: &mut StandardStream,
        _: &ColorSpec,
        resp: &WeatherResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_date = FixedOffset::east(resp.timezone).timestamp(resp.dt, 0);

        if let Some(ref style) = self.style {
            out.set_color(style)?;
        }

        write!(out, "{}", source_date.format(&self.date_format))?;

        Ok(())
    }
}

pub struct LocationName {
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

    pub fn build(self) -> Segment {
        Segment {
            inner: Box::new(self),
        }
    }
}

impl SegmentContent for LocationName {
    fn render(
        &self,
        out: &mut StandardStream,
        _: &ColorSpec,
        resp: &WeatherResponse,
    ) -> std::result::Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
        if let Some(ref style) = self.style {
            out.set_color(style)?;
        }

        write!(out, "{}", resp.name)?;

        Ok(())
    }
}

const TEMP_COLORS: [u8; 57] = [
    57, 63, 63, 63, 27, 27, 27, 33, 33, 33, 39, 39, 39, 45, 45, 45, 51, 51, 50, 50, 49, 49, 48, 48,
    47, 47, 46, 46, 46, 82, 82, 82, 118, 118, 118, 154, 154, 154, 190, 190, 190, 226, 226, 226,
    220, 220, 220, 214, 214, 214, 208, 208, 208, 202, 202, 202, 196,
];

pub struct Temperature {
    pub display_mode: DisplayMode,
}

impl Default for Temperature {
    fn default() -> Self {
        Temperature {
            display_mode: DisplayMode::NerdFont,
        }
    }
}

impl Temperature {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> Segment {
        Segment {
            inner: Box::new(self),
        }
    }

    fn display_temp(
        &self,
        out: &mut StandardStream,
        temp: f32,
        base_style: &ColorSpec,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_idx = (temp.round() + 16f32).min(57f32).max(0f32) as usize;

        out.set_color(
            base_style
                .clone()
                .set_fg(Some(Color::Ansi256(TEMP_COLORS[temp_idx])))
                .set_bold(true),
        )?;
        write!(out, " {:.1}", temp)?;
        out.set_color(base_style)?;
        write!(out, " °C")?;
        Ok(())
    }
}

impl SegmentContent for Temperature {
    fn render(
        &self,
        out: &mut StandardStream,
        base_style: &ColorSpec,
        resp: &WeatherResponse,
    ) -> std::result::Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
        write!(out, "\u{e350}")?;
        self.display_temp(out, resp.main.temp, base_style)?;
        write!(out, " (feels")?;
        self.display_temp(out, resp.main.feels_like, base_style)?;
        write!(out, ")")?;

        Ok(())
    }
}

pub struct WeatherIcon {
    pub display_mode: DisplayMode,
    pub style: Option<ColorSpec>,
}

impl Default for WeatherIcon {
    fn default() -> Self {
        let mut style = ColorSpec::new();
        style.set_reset(false);
        style.set_fg(Some(Color::White)).set_bold(true);
        WeatherIcon {
            display_mode: DisplayMode::NerdFont,
            style: Some(style),
        }
    }
}

impl WeatherIcon {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> Segment {
        Segment {
            inner: Box::new(self),
        }
    }

    fn get_icon(&self, id: u16, sunset: i64, sunrise: i64, wind_type: &WindType) -> &'static str {
        let now = Utc::now();
        let night = now >= Utc.timestamp(sunset, 0) || now <= Utc.timestamp(sunrise, 0);
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
            _ => "\u{e374}",
        }
    }
}

impl SegmentContent for WeatherIcon {
    fn render(
        &self,
        out: &mut StandardStream,
        _: &ColorSpec,
        resp: &WeatherResponse,
    ) -> std::result::Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
        let wind_type = resp
            .wind
            .as_ref()
            .map_or(WindType::Low, |w| get_wind_type(w.speed));

        if let Some(ref style) = self.style {
            out.set_color(style)?;
        }

        write!(
            out,
            "{}",
            self.get_icon(
                resp.weather[0].id,
                resp.sys.sunset,
                resp.sys.sunrise,
                &wind_type
            )
        )?;

        Ok(())
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

#[derive(Default)]
pub struct WeatherDescription {
    pub style: Option<ColorSpec>,
}

impl SegmentContent for WeatherDescription {
    fn render(
        &self,
        out: &mut StandardStream,
        _: &ColorSpec,
        resp: &WeatherResponse,
    ) -> std::result::Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
        write!(out, "{}", resp.weather[0].description)?;

        Ok(())
    }
}

impl WeatherDescription {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> Segment {
        Segment {
            inner: Box::new(self),
        }
    }
}

const WIND_COLORS: [u8; 52] = [
    46, 46, 46, 82, 82, 82, 118, 118, 118, 154, 154, 154, 190, 190, 190, 226, 226, 226, 226, 220,
    220, 220, 220, 214, 214, 214, 214, 208, 208, 208, 208, 202, 202, 202, 202, 196, 196, 196, 196,
    160, 160, 160, 160, 124, 124, 124, 124, 127, 127, 127, 127, 129,
];

const WIND_DIR_ICONS: &str =
    "\u{e35a}\u{e359}\u{e35b}\u{e356}\u{e357}\u{e355}\u{e354}\u{e358}\u{e35a}";

pub struct WindSpeed {
    pub display_mode: DisplayMode,
}

impl Default for WindSpeed {
    fn default() -> Self {
        WindSpeed {
            display_mode: DisplayMode::NerdFont,
        }
    }
}

impl WindSpeed {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> Segment {
        Segment {
            inner: Box::new(self),
        }
    }

    fn display_wind(
        &self,
        stdout: &mut StandardStream,
        wind: &Wind,
        wind_type: &WindType,
        base_style: &ColorSpec,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let icon = wind
            .deg
            .map(|deg| {
                let dir_idx = ((deg + 22.5) / 45f32).floor() as usize;
                &WIND_DIR_ICONS[3 * dir_idx..3 * dir_idx + 3]
            })
            .unwrap_or("\u{e3a9}");
        let speed = wind.speed * 3.6;
        let speed_color_idx = speed.floor() as usize;
        write!(stdout, "{}", icon)?;
        if let WindType::High = wind_type {
            write!(stdout, "\u{e34b}")?;
        }
        let mut tmp_style = base_style.clone();
        stdout.set_color(&tmp_style.set_fg(Some(Color::Ansi256(WIND_COLORS[speed_color_idx]))))?;
        write!(stdout, " {:.1}", speed)?;
        stdout.set_color(tmp_style.set_fg(None).set_bold(false))?;
        write!(stdout, " km/h")?;
        Ok(())
    }
}

impl SegmentContent for WindSpeed {
    fn render(
        &self,
        out: &mut StandardStream,
        base_style: &ColorSpec,
        resp: &WeatherResponse,
    ) -> std::result::Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
        if let Some(w) = &resp.wind {
            let wind_type = resp
                .wind
                .as_ref()
                .map_or(WindType::Low, |w| get_wind_type(w.speed));

            self.display_wind(out, &w, &wind_type, base_style)?;
        }

        Ok(())
    }
}

const HUMIDITY_COLORS: [u8; 11] = [220, 226, 190, 118, 82, 46, 48, 50, 51, 45, 39];

pub struct Humidity {
    pub display_mode: DisplayMode,
}

impl Default for Humidity {
    fn default() -> Self {
        Humidity {
            display_mode: DisplayMode::NerdFont,
        }
    }
}

impl Humidity {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> Segment {
        Segment {
            inner: Box::new(self),
        }
    }

    fn display_humidity(
        &self,
        stdout: &mut StandardStream,
        humidity: u8,
        base_style: &ColorSpec,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let hum_idx = (humidity / 10) as usize;
        write!(stdout, "\u{e373} ")?;
        let mut tmp_style = base_style.clone();
        stdout.set_color(tmp_style.set_fg(Some(Color::Ansi256(HUMIDITY_COLORS[hum_idx]))))?;
        write!(stdout, "{}", humidity)?;
        stdout.set_color(tmp_style.set_fg(None))?;
        write!(stdout, " %")?;
        Ok(())
    }
}

impl SegmentContent for Humidity {
    fn render(
        &self,
        out: &mut StandardStream,
        base_style: &ColorSpec,
        resp: &WeatherResponse,
    ) -> std::result::Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
        self.display_humidity(out, resp.main.humidity, base_style)?;

        Ok(())
    }
}

pub struct Rain {
    pub display_mode: DisplayMode,
}

impl Default for Rain {
    fn default() -> Self {
        Rain {
            display_mode: DisplayMode::NerdFont,
        }
    }
}

impl Rain {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> Segment {
        Segment {
            inner: Box::new(self),
        }
    }
}

impl SegmentContent for Rain {
    fn render(
        &self,
        out: &mut StandardStream,
        _: &ColorSpec,
        resp: &WeatherResponse,
    ) -> std::result::Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
        if let Some(r) = &resp.rain {
            if let Some(mm) = r.one_h.or(r.three_h) {
                write!(out, "\u{e371} {:.1} mm/h  ", mm)?;
            }
        }
        Ok(())
    }
}

pub struct Pressure {
    pub display_mode: DisplayMode,
}

impl Default for Pressure {
    fn default() -> Self {
        Pressure {
            display_mode: DisplayMode::NerdFont,
        }
    }
}

impl Pressure {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> Segment {
        Segment {
            inner: Box::new(self),
        }
    }

    fn display_pressure(
        &self,
        stdout: &mut StandardStream,
        pressure: u16,
        base_style: &ColorSpec,
    ) -> Result<(), Box<dyn std::error::Error>> {
        write!(stdout, "\u{e372} ")?;
        let mut tmp_style = base_style.clone();
        stdout.set_color(tmp_style.set_fg(Some(Color::White)))?;
        write!(stdout, "{}", pressure)?;
        stdout.set_color(tmp_style.set_fg(None))?;
        write!(stdout, " hPa")?;

        Ok(())
    }
}

impl SegmentContent for Pressure {
    fn render(
        &self,
        out: &mut StandardStream,
        base_style: &ColorSpec,
        resp: &WeatherResponse,
    ) -> std::result::Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
        self.display_pressure(out, resp.main.pressure, base_style)?;
        Ok(())
    }
}
