pub mod api;
pub mod cli;
pub mod config;
#[cfg(feature = "geoclue")]
pub mod geoclue;
pub mod segments;
mod serde_utils;

use std::{borrow::Cow, time::Duration};

use crate::config::DisplayConfig;
use anyhow::*;
use api::{current::ApiResponse as CResponse, one_call::ApiResponse as OResponse, Response};
use directories_next::ProjectDirs;
use log::*;
use reqwest::StatusCode;
use segments::Renderer;
use serde::{Deserialize, Serialize};
use termcolor::StandardStream;
use tokio::time::timeout;

const CURRENT_API_URL: &str = "https://api.openweathermap.org/data/2.5/weather?units=metric";
const ONECALL_API_URL: &str = "https://api.openweathermap.org/data/2.5/onecall?units=metric";

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

    pub async fn display(&self, loc: &Location, out: &mut StandardStream) -> Result<()> {
        let mut renderer = Renderer::from(&self.config);

        let kind = renderer.display_kind()?;

        let mut response = Response::empty();
        if kind != QueryKind::ForeCast {
            let res = WeatherClient::new(self.cache_length, self.timeout)
                .query(
                    QueryKind::Current,
                    loc,
                    self.key.clone(),
                    self.language.as_deref(),
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

        if kind != QueryKind::Current {
            let res = WeatherClient::new(self.cache_length, self.timeout)
                .query(
                    QueryKind::ForeCast,
                    &new_loc,
                    self.key.clone(),
                    self.language.as_deref(),
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
    ) -> Result<std::path::PathBuf> {
        if let Some(p) = WeatherClient::directories() {
            let prefix = match kind {
                QueryKind::Current => "api",
                QueryKind::ForeCast => "oapi",
                QueryKind::Both => bail!("internal error: find_cache_for(Both)"),
            };

            let suffix = match location {
                Location::LatLon(lat, lon) => format!("{}_{}", lat, lon),
                Location::Place(p) => self.clean_up_for_path(p),
            };
            let f = if let Some(lang) = language {
                format!("results/{}-{}-{}.json", prefix, lang, suffix)
            } else {
                format!("results/{}-{}.json", prefix, suffix)
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
    ) -> Result<Option<Response>> {
        if let Some(cache_length) = self.cache_length {
            let path = self.find_cache_for(kind, location, language)?;

            if path.exists() {
                let m = std::fs::metadata(&path)?;
                let elapsed = m.modified()?.elapsed()?;
                if elapsed <= cache_length {
                    let f = std::fs::File::open(&path)?;
                    match kind {
                        QueryKind::Current => {
                            if let CResponse::Success(resp) = serde_json::from_reader(f)? {
                                info!("using cached response for {}", location);

                                return Ok(Some(Response::from_current(resp)));
                            }
                        }
                        QueryKind::ForeCast => {
                            if let OResponse::Success(resp) = serde_json::from_reader(f)? {
                                info!("using cached response for {}", location);

                                return Ok(Some(Response::from_forecast(*resp)));
                            }
                        }
                        QueryKind::Both => bail!("internal error: query_cache(Both)"),
                    }
                } else {
                    info!("ignoring expired cached response for {}", location);
                }
            } else {
                info!("no cached response found for {}", location);
            }
        }

        Ok(None)
    }

    fn write_cache(
        &self,
        kind: QueryKind,
        location: &Location,
        language: Option<&str>,
        bytes: &[u8],
    ) -> Result<()> {
        let path = self.find_cache_for(kind, location, language)?;
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
    ) -> Result<Response> {
        // Adapt between locales and Openweather language codes:
        // the codes OW accepts are a mix of ISO 639-1 language codes,
        // ISO 3166 country codes and locale-like codes...
        let language = language.map(make_openweather_language_codes);

        match self.query_cache(kind, location, language.as_deref()) {
            Ok(Some(resp)) => return Ok(resp),
            Ok(None) => {}
            Err(e) => {
                warn!("error while looking for cache: {}", e);
            }
        }

        self.query_api(kind, location, key, language.as_deref())
            .await
    }

    async fn query_api(
        &self,
        kind: QueryKind,
        location: &Location,
        key: String,
        language: Option<&str>,
    ) -> Result<Response> {
        debug!("querying {:?} with '{:?}' OpenWeather API", location, kind);
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

        let api_url = match kind {
            QueryKind::Current => CURRENT_API_URL,
            QueryKind::ForeCast => ONECALL_API_URL,
            QueryKind::Both => bail!("internal error: query_api(Both)"),
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
                            if let Err(e) = self.write_cache(kind, location, language, &bytes) {
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
                            if let Err(e) = self.write_cache(kind, location, language, &bytes) {
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
            QueryKind::Both => bail!("internal error: query_api(Both)"),
        }
    }
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryKind {
    Current,
    ForeCast,
    Both,
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
