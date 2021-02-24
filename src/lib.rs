pub mod cli;
pub mod config;
#[cfg(feature = "geoclue")]
pub mod geoclue;
pub mod response;
pub mod segments;
mod serde_utils;

use anyhow::*;
use directories_next::ProjectDirs;
use log::*;
use response::{ApiResponse, WeatherResponse};
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.openweathermap.org/data/2.5/weather?units=metric";

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

#[derive(Default)]
pub struct WeatherClient {
    client: reqwest::Client,
    cache_length: Option<String>,
}

impl WeatherClient {
    pub fn new(cache_length: Option<String>) -> Self {
        WeatherClient {
            client: reqwest::Client::new(),
            cache_length,
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

    fn find_cache_for(&self, location: &Location) -> Result<std::path::PathBuf> {
        if let Some(p) = WeatherClient::directories() {
            let suffix = match location {
                Location::LatLon(lat, lon) => format!("{}_{}", lat, lon),
                Location::Place(p) => self.clean_up_for_path(&p),
            };
            let file = p.cache_dir().join(format!("results/api-{}.json", suffix));
            debug!("looking at cache file at '{}'", file.display());

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

    fn query_cache(&self, location: &Location) -> Result<Option<WeatherResponse>> {
        if let Some(ref cache_length) = self.cache_length {
            let duration = humantime::parse_duration(cache_length)?;
            let path = self.find_cache_for(&location)?;

            if path.exists() {
                let m = std::fs::metadata(&path)?;
                let elapsed = m.modified()?.elapsed()?;
                if elapsed <= duration {
                    let f = std::fs::File::open(&path)?;
                    if let ApiResponse::Success(resp) = serde_json::from_reader(f)? {
                        info!("using cached response for {}", location);
                        return Ok(Some(resp));
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

    fn write_cache(&self, location: Location, bytes: &[u8]) -> Result<()> {
        let path = self.find_cache_for(&location)?;
        debug!("writing cache for {}", location);
        std::fs::write(path, bytes)?;

        Ok(())
    }

    pub async fn query(
        &self,
        location: Location,
        key: String,
        language: Option<String>,
    ) -> Result<WeatherResponse> {
        match self.query_cache(&location) {
            Ok(Some(resp)) => return Ok(resp),
            Ok(None) => {}
            Err(e) => {
                warn!("error while looking for cache: {}", e);
            }
        }

        self.query_api(location, key, language).await
    }

    async fn query_api(
        &self,
        location: Location,
        key: String,
        language: Option<String>,
    ) -> Result<WeatherResponse> {
        debug!("querying {:?}", location);
        let mut params = Vec::with_capacity(3);
        match &location {
            Location::LatLon(lat, lon) => {
                params.push(("lat", lat.to_string()));
                params.push(("lon", lon.to_string()));
            }
            Location::Place(place) => params.push(("q", place.to_string())),
        };

        if let Some(language) = language {
            params.push(("lang", language));
        }

        params.push(("appid", key));

        let bytes = self
            .client
            .get(API_URL)
            .query(&params)
            .send()
            .await?
            .bytes()
            .await
            .context("Unable to connect to openweathermap.org")?;

        if log_enabled!(Level::Trace) {
            trace!("received response: {}", std::str::from_utf8(&bytes)?);
        }

        let resp: ApiResponse = serde_json::from_slice(&bytes)?;

        match resp {
            ApiResponse::Success(w) => {
                if self.cache_length.is_some() {
                    if let Err(e) = self.write_cache(location, &bytes) {
                        warn!("error while writing cached response: {}", e);
                    }
                }
                Ok(w)
            }
            ApiResponse::OtherInt { cod, message } => handle_error(cod, &message, location),
            ApiResponse::OtherString { cod, message } => {
                handle_error(cod.parse().unwrap_or_default(), &message, location)
            }
        }
    }
}

fn handle_error(error_code: u32, message: &str, location: Location) -> Result<WeatherResponse> {
    match error_code {
        404 => bail!("location error: '{}' for '{}'", message, location),
        429 => bail!("Too many calls to the API! If you not using your own API key, please get your own for free over at http://openweathermap.org"),
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
