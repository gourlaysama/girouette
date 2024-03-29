pub mod api;
pub mod cli;
pub mod config;
#[cfg(feature = "geoclue")]
pub mod geoclue;
pub mod segments;
mod serde_utils;

use std::{borrow::Cow, fmt::Display, path::Path, time::Duration};

use crate::config::DisplayConfig;
use anyhow::{bail, Context, Result};
use api::{current::ApiResponse as CResponse, one_call::ApiResponse as OResponse, pollution::ApiResponse as PResponse, Response};
use directories_next::ProjectDirs;
use log::*;
use reqwest::StatusCode;
use segments::Renderer;
use serde::{Deserialize, Serialize};
use termcolor::StandardStream;
use tokio::time::timeout;

const CURRENT_API_URL: &str = "https://api.openweathermap.org/data/2.5/weather";
const ONECALL_API_URL: &str = "https://api.openweathermap.org/data/2.5/onecall";
const POLLUTION_API_URL: &str = "http://api.openweathermap.org/data/2.5/air_pollution";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(from = "String", into = "String")]
pub enum Location {
    LatLon(f64, f64),
    Place(String),
}

impl From<String> for Location {
    fn from(s: String) -> Self {
        Location::new(&s)
    }
}

impl From<Location> for String {
    fn from(loc: Location) -> Self {
        format!("{}", loc)
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::LatLon(lat, lon) => write!(f, "{}, {}", lat, lon),
            Location::Place(place) => write!(f, "{}", place),
        }
    }
}

pub enum WindType {
    Low,
    Mid,
    High,
}

impl Location {
    pub fn new(s: &str) -> Location {
        let sp: Vec<_> = s.split(',').collect();
        if sp.len() == 2 {
            if let (Ok(lat), Ok(lon)) = (sp[0].parse(), sp[1].parse()) {
                return Location::LatLon(lat, lon);
            }

            debug!(
                "could not parse '{}' as 'lat,lon', assuming it is a place",
                s
            );
        }

        Location::Place(s.to_owned())
    }
}

pub struct Girouette {
    config: DisplayConfig,
    cache_length: Option<Duration>,
    timeout: Duration,
    key: String,
    language: Option<String>,
}

impl Girouette {
    pub fn new(
        config: DisplayConfig,
        cache_length: Option<Duration>,
        timeout: Duration,
        key: String,
        language: Option<String>,
    ) -> Self {
        Self {
            config,
            cache_length,
            timeout,
            key,
            language,
        }
    }

    pub async fn display(
        &self,
        loc: &Location,
        offline: bool,
        out: &mut StandardStream,
    ) -> Result<()> {
        let mut renderer = Renderer::from(&self.config);

        let kinds = renderer.display_kinds()?;

        let mut response = Response::empty();
        if kinds.contains(&QueryKind::Current) || (matches!(loc, Location::Place(_))) {
            let res = WeatherClient::new(self.cache_length, self.timeout)
                .query(
                    QueryKind::Current,
                    loc,
                    self.key.clone(),
                    self.language.as_deref(),
                    self.config.units,
                    offline,
                )
                .await?;
            response.merge(res);
        }
        let new_loc = if let Location::Place(_) = loc {
            let resp = response.as_current()?;
            Location::LatLon(resp.coord.lat, resp.coord.lon)
        } else {
            loc.clone()
        };

        for kind in kinds {
            if kind == QueryKind::Current {
                continue;
            }

            let res = WeatherClient::new(self.cache_length, self.timeout)
                .query(
                    kind,
                    &new_loc,
                    self.key.clone(),
                    self.language.as_deref(),
                    self.config.units,
                    offline,
                )
                .await?;
            response.merge(res);
        }

        renderer.render(out, &response, self.language.as_deref())?;

        Ok(())
    }
}

pub struct WeatherClient {
    client: reqwest::Client,
    cache_length: Option<Duration>,
    timeout: Duration,
}

impl WeatherClient {
    pub fn new(cache_length: Option<Duration>, timeout: Duration) -> Self {
        WeatherClient {
            client: reqwest::Client::new(),
            cache_length,
            timeout,
        }
    }

    pub fn clean_cache() -> Result<()> {
        if let Some(p) = WeatherClient::directories() {
            let results = p.cache_dir().join("results");
            if results.exists() {
                std::fs::remove_dir_all(&results)?;
                println!("Cleaned cache directory ({})", results.to_string_lossy());
            }
        }
        Ok(())
    }

    pub fn directories() -> Option<ProjectDirs> {
        ProjectDirs::from("rs", "", "Girouette")
    }

    fn find_cache_for(
        &self,
        kind: QueryKind,
        location: &Location,
        language: Option<&str>,
        units: UnitMode,
    ) -> Result<std::path::PathBuf> {
        if let Some(p) = WeatherClient::directories() {
            let prefix = match kind {
                QueryKind::Current => "api",
                QueryKind::ForeCast => "oapi",
                QueryKind::Pollution => "papi",
            };

            let prefix2 = match units {
                UnitMode::Standard => "S",
                UnitMode::Metric => "",
                UnitMode::Imperial => "I",
            };

            let suffix = match location {
                Location::LatLon(lat, lon) => format!("{}_{}", lat, lon),
                Location::Place(p) => self.clean_up_for_path(p),
            };
            let f = if let Some(lang) = language {
                format!("results/{}{}-{}-{}.json", prefix, prefix2, lang, suffix)
            } else {
                format!("results/{}{}-{}.json", prefix, prefix2, suffix)
            };
            let file = p.cache_dir().join(f);
            debug!("looking for cache file at '{}'", file.display());

            if let Some(p) = file.parent() {
                std::fs::create_dir_all(p)?;
            }

            Ok(file)
        } else {
            bail!("Count not locate project directory!");
        }
    }

    fn clean_up_for_path(&self, name: &str) -> String {
        let mut buf = String::with_capacity(name.len());
        let mut parts = name.split_whitespace();
        let mut current_part = parts.next();
        while let Some(part) = current_part {
            let value = part.to_lowercase();
            buf.push_str(&value);
            current_part = parts.next();
            if current_part.is_some() {
                buf.push('_');
            }
        }
        buf
    }

    fn query_cache(
        &self,
        kind: QueryKind,
        location: &Location,
        language: Option<&str>,
        units: UnitMode,
        offline: bool,
    ) -> Result<Option<Response>> {
        if offline {
            let path = self.find_cache_for(kind, location, language, units)?;

            if path.exists() {
                return parse_cached_response(path.as_path(), kind, location);
            } else {
                bail!(
                    "failed to find a cached response for '{}', but running offline",
                    location
                );
            }
        } else if let Some(cache_length) = self.cache_length {
            let path = self.find_cache_for(kind, location, language, units)?;

            if path.exists() {
                let m = std::fs::metadata(&path)?;
                let elapsed = m.modified()?.elapsed()?;
                if elapsed <= cache_length {
                    return parse_cached_response(path.as_path(), kind, location);
                } else {
                    info!("ignoring expired cached response for {}", location);
                }
            }
        }

        Ok(None)
    }

    fn write_cache(
        &self,
        kind: QueryKind,
        location: &Location,
        language: Option<&str>,
        units: UnitMode,
        bytes: &[u8],
    ) -> Result<()> {
        let path = self.find_cache_for(kind, location, language, units)?;
        debug!("writing cache for {}", location);
        std::fs::write(path, bytes)?;

        Ok(())
    }

    pub async fn query(
        &self,
        kind: QueryKind,
        location: &Location,
        key: String,
        language: Option<&str>,
        units: UnitMode,
        offline: bool,
    ) -> Result<Response> {
        // Adapt between locales and Openweather language codes:
        // the codes OW accepts are a mix of ISO 639-1 language codes,
        // ISO 3166 country codes and locale-like codes...
        let language = language.map(make_openweather_language_codes);

        match self.query_cache(kind, location, language.as_deref(), units, offline) {
            Ok(Some(resp)) => return Ok(resp),
            Ok(None) => {}
            Err(e) => {
                if offline {
                    return Err(e);
                }
                warn!("error while looking for cache: {}", e);
            }
        }

        self.query_api(kind, location, key, language.as_deref(), units)
            .await
    }

    async fn query_api(
        &self,
        kind: QueryKind,
        location: &Location,
        key: String,
        language: Option<&str>,
        units: UnitMode,
    ) -> Result<Response> {
        debug!("querying {:?} with {:?} OpenWeather API", location, kind);
        let mut params = Vec::with_capacity(3);
        match location {
            Location::LatLon(lat, lon) => {
                params.push(("lat", lat.to_string()));
                params.push(("lon", lon.to_string()));
            }
            Location::Place(place) => params.push(("q", place.to_string())),
        };

        if let Some(language) = language {
            params.push(("lang", language.to_owned()));
        }

        params.push(("appid", key));

        params.push(("units", units.to_string()));

        let api_url = match kind {
            QueryKind::Current => CURRENT_API_URL,
            QueryKind::ForeCast => ONECALL_API_URL,
            QueryKind::Pollution => POLLUTION_API_URL,
        };

        let request = self.client.get(api_url).query(&params).send();
        let request = timeout(self.timeout, request);

        let response = request
            .await
            .context("Connection to openweathermap.org timed-out")?
            .context("Unable to connect to openweathermap.org")?;

        let bytes = timeout(self.timeout, response.bytes());

        let bytes = bytes
            .await
            .context("Connection to openweathermap.org timed-out")?
            .context("Unable to connect to openweathermap.org")?;

        if log_enabled!(Level::Trace) {
            trace!("received response: {}", std::str::from_utf8(&bytes)?);
        }

        match kind {
            QueryKind::Current => {
                let resp: CResponse = serde_json::from_slice(&bytes)?;
                match resp {
                    CResponse::Success(w) => {
                        if self.cache_length.is_some() {
                            if let Err(e) =
                                self.write_cache(kind, location, language, units, &bytes)
                            {
                                warn!("error while writing cached response: {}", e);
                            }
                        }
                        Ok(Response::from_current(w))
                    }
                    CResponse::OtherInt { cod, message } => {
                        handle_error(StatusCode::from_u16(cod)?, &message, location)
                    }
                    CResponse::OtherString { cod, message } => {
                        handle_error(cod.parse()?, &message, location)
                    }
                }
            }
            QueryKind::ForeCast => {
                let resp: OResponse = serde_json::from_slice(&bytes)?;
                match resp {
                    OResponse::Success(w) => {
                        if self.cache_length.is_some() {
                            if let Err(e) =
                                self.write_cache(kind, location, language, units, &bytes)
                            {
                                warn!("error while writing cached response: {}", e);
                            }
                        }
                        Ok(Response::from_forecast(*w))
                    }
                    OResponse::OtherInt { cod, message } => {
                        handle_error(StatusCode::from_u16(cod)?, &message, location)
                    }
                    OResponse::OtherString { cod, message } => {
                        handle_error(cod.parse()?, &message, location)
                    }
                }
            }
            QueryKind::Pollution => {
                let resp: PResponse = serde_json::from_slice(&bytes)?;
                match resp {
                    PResponse::Success(p) => {
                        if self.cache_length.is_some() {
                            if let Err(e) =
                                self.write_cache(kind, location, language, units, &bytes)
                            {
                                warn!("error while writing cached response: {}", e);
                            }
                        }
                        Ok(Response::from_pollution(p))
                    }
                    PResponse::OtherInt { cod, message } => {
                        handle_error(StatusCode::from_u16(cod)?, &message, location)
                    }
                    PResponse::OtherString { cod, message } => {
                        handle_error(cod.parse()?, &message, location)
                    }
                }
            },
        }
    }
}

fn parse_cached_response(
    path: &Path,
    kind: QueryKind,
    location: &Location,
) -> Result<Option<Response>, anyhow::Error> {
    let f = std::fs::File::open(path)?;
    Ok(match kind {
        QueryKind::Current => {
            if let CResponse::Success(resp) = serde_json::from_reader(f)? {
                info!("using cached response for {}", location);

                Some(Response::from_current(resp))
            } else {
                None
            }
        }
        QueryKind::ForeCast => {
            if let OResponse::Success(resp) = serde_json::from_reader(f)? {
                info!("using cached response for {}", location);

                Some(Response::from_forecast(*resp))
            } else {
                None
            }
        }
        QueryKind::Pollution => {
            if let PResponse::Success(resp) = serde_json::from_reader(f)? {
                info!("using cached response for {}", location);

                Some(Response::from_pollution(resp))
            } else {
                None
            }
        },
    })
}

fn handle_error(error_code: StatusCode, message: &str, location: &Location) -> Result<Response> {
    match error_code {
        StatusCode::NOT_FOUND => bail!("location error: '{}' for '{}'", message, location),
        StatusCode::TOO_MANY_REQUESTS => bail!("Too many calls to the API! If you not using your own API key, please get your own for free over at http://openweathermap.org"),
        _ => bail!("error from OpenWeather API: {}: {}", error_code, message),
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    NerdFonts,
    Unicode,
    Ascii,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryKind {
    Current,
    ForeCast,
    Pollution,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitMode {
    Standard,
    Metric,
    Imperial,
}

impl Display for UnitMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let o = match self {
            UnitMode::Standard => "standard",
            UnitMode::Metric => "metric",
            UnitMode::Imperial => "imperial",
        };

        f.write_str(o)
    }
}

fn make_openweather_language_codes(s: &str) -> Cow<str> {
    // openweather supports these directly
    if let "zh_CN" | "zh_TW" | "pt_BR" = s {
        return s.to_lowercase().into();
    };

    let l_code = s.split_once('_').map(|t| t.0).unwrap_or(s);

    // openweather uses country codes for those
    match l_code {
        "sq" => "al",        // Albanian
        "cs" => "cz",        // Czech
        "ko" => "kr",        // Korean
        "lv" => "la",        // Latvian
        "nb" | "nn" => "no", // Norwegian
        s => s,
    }
    .into()
}

#[macro_export]
macro_rules! show {
    ($level:ident, $($a:tt),*) => {
        if log_enabled!(log::Level::$level) {
            println!($($a,)*);
        }
    };
    ($($a:tt),*) => {
        if log_enabled!(log::Level::Error) {
            println!($($a,)*);
        }
    }
}
