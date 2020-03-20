use chrono::{FixedOffset, TimeZone, Utc};
use std::io::Write;
use structopt::StructOpt;
use termcolor::*;

const API_URL: &str = "https://api.openweathermap.org/data/2.5/weather?units=metric";
const TEMP_COLORS: [u8; 57] = [
    57, 63, 63, 63, 27, 27, 27, 33, 33, 33, 39, 39, 39, 45, 45, 45, 51, 51, 50, 50, 49, 49, 48, 48,
    47, 47, 46, 46, 46, 82, 82, 82, 118, 118, 118, 154, 154, 154, 190, 190, 190, 226, 226, 226,
    220, 220, 220, 214, 214, 214, 208, 208, 208, 202, 202, 202, 196,
];
const WIND_COLORS: [u8; 52] = [
    46, 46, 46, 82, 82, 82, 118, 118, 118, 154, 154, 154, 190, 190, 190, 226, 226, 226, 226, 220,
    220, 220, 220, 214, 214, 214, 214, 208, 208, 208, 208, 202, 202, 202, 202, 196, 196, 196, 196,
    160, 160, 160, 160, 124, 124, 124, 124, 127, 127, 127, 127, 129,
];
const WIND_DIR_ICONS: &str =
    "\u{e35a}\u{e359}\u{e35b}\u{e356}\u{e357}\u{e355}\u{e354}\u{e358}\u{e35a}";
const HUMIDITY_COLORS: [u8; 11] = [220, 226, 190, 118, 82, 46, 48, 50, 51, 45, 39];

#[derive(StructOpt, Debug)]
#[structopt(about = "Display the current weather using the Openweather API.")]
struct ProgramOptions {
    #[structopt(short, long)]
    key: String,

    #[structopt(short, long, parse(from_str = parse_location))]
    location: Location,
}

#[derive(Debug)]
enum Location {
    LatLon(f64, f64),
    Place(String),
}

fn parse_location(s: &str) -> Location {
    let sp: Vec<_> = s.split(',').collect();
    if sp.len() == 2 {
        if let (Ok(lat), Ok(lon)) = (sp[0].parse(), sp[1].parse()) {
            return Location::LatLon(lat, lon)
        }
    }

    Location::Place(s.to_owned())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ProgramOptions::from_args();

    let mut params = Vec::with_capacity(3);
    match options.location {
        Location::LatLon(lat, lon) => {
            params.push(("lat", lat.to_string()));
            params.push(("lon", lon.to_string()));
        }
        Location::Place(place) => params.push(("q", place))
    };

    params.push(("appid", options.key));

    let resp = reqwest::Client::new()
        .get(API_URL)
        .query(&params)
        .send()
        .await?
        .json::<WeatherResponse>()
        .await?;

    let wind_type = resp
        .wind
        .as_ref()
        .map_or(WindType::Low, |w| get_wind_type(w.speed));

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let mut style = ColorSpec::new();
    style.set_bg(Some(Color::Rgb(15, 55, 84)));

    stdout.set_color(&style)?;

    let source_date = FixedOffset::east(resp.timezone).timestamp(resp.dt, 0);
    write!(stdout, "{}", source_date)?;
    write!(stdout, "  ")?;
    stdout.set_color(style.set_fg(Some(Color::Blue)).set_intense(true))?;
    write!(stdout, "{}", resp.name)?;
    stdout.set_color(style.set_fg(None))?;
    write!(stdout, "  \u{e350}")?;
    display_temp(&mut stdout, resp.main.temp, &mut style)?;
    write!(stdout, " (feels")?;
    display_temp(&mut stdout, resp.main.feels_like, &mut style)?;
    write!(stdout, ")")?;

    stdout.set_color(style.set_fg(Some(Color::White)).set_intense(true))?;
    write!(
        stdout,
        "  {}  ",
        get_icon(
            resp.weather[0].id,
            resp.sys.sunset,
            resp.sys.sunrise,
            &wind_type
        )
    )?;
    stdout.set_color(style.set_fg(None).set_bold(false))?;
    write!(stdout, "{}", resp.weather[0].description)?;
    write!(stdout, "  ")?;

    if let Some(w) = resp.wind {
        display_wind(&mut stdout, &w, &wind_type, &mut style)?
    };

    if let Some(r) = resp.rain {
        if let Some(mm) = r.one_h.or(r.three_h) {
            write!(stdout, "  \u{e371} {:.1} mm/h  ", mm)?;
        }
    }

    display_humidity(&mut stdout, resp.main.humidity, &mut style)?;
    display_pressure(&mut stdout, resp.main.pressure, &mut style)?;

    stdout.reset()?;
    Ok(())
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

enum WindType {
    Low,
    Mid,
    High,
}

fn get_icon(id: u16, sunset: i64, sunrise: i64, wind_type: &WindType) -> &'static str {
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

fn display_pressure(
    stdout: &mut StandardStream,
    pressure: u16,
    style: &mut ColorSpec,
) -> Result<(), Box<dyn std::error::Error>> {
    write!(stdout, "  \u{e372} ")?;
    stdout.set_color(style.set_fg(Some(Color::White)))?;
    write!(stdout, "{}", pressure)?;
    stdout.set_color(style.set_fg(None))?;
    write!(stdout, " hPa")?;

    Ok(())
}

fn display_humidity(
    stdout: &mut StandardStream,
    humidity: u8,
    style: &mut ColorSpec,
) -> Result<(), Box<dyn std::error::Error>> {
    let hum_idx = (humidity / 10) as usize;
    write!(stdout, "  \u{e373} ")?;
    stdout.set_color(style.set_fg(Some(Color::Ansi256(HUMIDITY_COLORS[hum_idx]))))?;
    write!(stdout, "{}", humidity)?;
    stdout.set_color(style.set_fg(None))?;
    write!(stdout, " %")?;
    Ok(())
}

fn display_wind(
    stdout: &mut StandardStream,
    wind: &Wind,
    wind_type: &WindType,
    style: &mut ColorSpec,
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
    stdout.set_color(style.set_fg(Some(Color::Ansi256(WIND_COLORS[speed_color_idx]))))?;
    write!(stdout, " {:.1}", speed)?;
    stdout.set_color(style.set_fg(None).set_bold(false))?;
    write!(stdout, " km/h")?;

    Ok(())
}

fn display_temp(
    stdout: &mut StandardStream,
    temp: f32,
    style: &mut ColorSpec,
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_idx = (temp.round() + 16f32).min(57f32).max(0f32) as usize;
    stdout.set_color(
        style
            .set_fg(Some(Color::Ansi256(TEMP_COLORS[temp_idx])))
            .set_bold(true),
    )?;
    write!(stdout, " {:.1}", temp)?;
    stdout.set_color(style.set_fg(None).set_bold(false))?;
    write!(stdout, " °C")?;
    Ok(())
}

#[derive(serde::Deserialize, Debug)]
struct WeatherResponse {
    coord: Coord,
    weather: Vec<Weather>,
    main: Main,
    visibility: u16,
    wind: Option<Wind>,
    rain: Option<Rain>,
    snow: Option<Snow>,
    clouds: Option<Clouds>,
    dt: i64,
    sys: Sys,
    timezone: i32,
    id: u32,
    name: String,
}

#[derive(serde::Deserialize, Debug)]
struct Coord {
    lat: f32,
    lon: f32,
}

#[derive(serde::Deserialize, Debug)]
struct Weather {
    id: u16,
    main: String,
    description: String,
    icon: String,
}

#[derive(serde::Deserialize, Debug)]
struct Main {
    temp: f32,
    feels_like: f32,
    temp_min: f32,
    temp_max: f32,
    pressure: u16,
    humidity: u8,
}

#[derive(serde::Deserialize, Debug)]
struct Wind {
    speed: f32,
    deg: Option<f32>,
    gale: Option<f32>,
}

#[derive(serde::Deserialize, Debug)]
struct Rain {
    #[serde(rename(deserialize = "1h"))]
    one_h: Option<f32>,
    #[serde(rename(deserialize = "3h"))]
    three_h: Option<f32>,
}

#[derive(serde::Deserialize, Debug)]
struct Snow {
    #[serde(rename(deserialize = "1h"))]
    one_h: Option<f32>,
    #[serde(rename(deserialize = "3h"))]
    three_h: Option<f32>,
}

#[derive(serde::Deserialize, Debug)]
struct Clouds {
    all: u16,
}

#[derive(serde::Deserialize, Debug)]
struct Sys {
    country: String,
    sunrise: i64,
    sunset: i64,
}
