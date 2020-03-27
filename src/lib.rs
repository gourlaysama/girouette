pub mod config;
pub mod response;
pub mod segments;

use log::debug;
use response::WeatherResponse;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.openweathermap.org/data/2.5/weather?units=metric";

#[derive(Clone, Debug, Serialize, Deserialize)]
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

            debug!("could not parse '{}' as 'lat,lon', assuming it is a place", s);
        }

        Location::Place(s.to_owned())
    }
}

#[derive(Default)]
pub struct WeatherClient {
    client: reqwest::Client,
}

impl WeatherClient {
    pub fn new() -> Self {
        WeatherClient {
            client: reqwest::Client::new(),
        }
    }

    pub async fn query(
        &self,
        location: Location,
        key: String,
    ) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
        debug!("querying {:?}", location);
        let mut params = Vec::with_capacity(3);
        match location {
            Location::LatLon(lat, lon) => {
                params.push(("lat", lat.to_string()));
                params.push(("lon", lon.to_string()));
            }
            Location::Place(place) => params.push(("q", place)),
        };

        params.push(("appid", key));

        self.client
            .get(API_URL)
            .query(&params)
            .send()
            .await?
            .json::<WeatherResponse>()
            .await
            .map_err(|e| e.into())
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    NerdFonts,
    Unicode,
    Ascii,
}
