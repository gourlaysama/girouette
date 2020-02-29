const API_URL: &str = "https://api.openweathermap.org/data/2.5/weather?units=metric";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::Client::new()
        .get(API_URL)
        .query(&[
            ("lat", "48.873"),
            ("lon", "2.295"),
            ("appid", "85a4e3c55b73909f42c6a23ec35b7147"),
        ])
        .send()
        .await?
        .json::<WeatherResponse>()
        .await?;

    println!("{:#?}", resp);

    Ok(())
}

#[derive(serde::Deserialize, Debug)]
struct WeatherResponse {
    weather: Vec<Weather>,
    main: Main,
    visibility: u16,
    wind: Option<Wind>,
    rain: Option<Rain>,
    snow: Option<Snow>,
    clouds: Option<Clouds>,
    dt: u64,
    timezone: u16,
    id: u32,
    name: String,
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
    deg: u8,
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
    sunrise: u64,
    sunset: u64,
}