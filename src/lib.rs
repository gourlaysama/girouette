pub mod response;

const API_URL: &str = "https://api.openweathermap.org/data/2.5/weather?units=metric";

use response::WeatherResponse;

#[derive(Debug)]
pub enum Location {
    LatLon(f64, f64),
    Place(String),
}

impl Location {
    pub fn new(s: &str) -> Location {
        let sp: Vec<_> = s.split(',').collect();
        if sp.len() == 2 {
            if let (Ok(lat), Ok(lon)) = (sp[0].parse(), sp[1].parse()) {
                return Location::LatLon(lat, lon);
            }
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
