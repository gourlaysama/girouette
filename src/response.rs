#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum ApiResponse {
    Success(WeatherResponse),
    // hack: Openweather API returns some very ugly json
    OtherInt { cod: u16, message: String },
    OtherString { cod: String, message: String },
}

#[derive(serde::Deserialize, Debug)]
pub struct WeatherResponse {
    pub coord: Coord,
    pub weather: Vec<Weather>,
    pub main: Main,
    pub visibility: Option<u16>,
    pub wind: Option<Wind>,
    pub rain: Option<Rain>,
    pub snow: Option<Snow>,
    pub clouds: Option<Clouds>,
    pub dt: i64,
    pub sys: Sys,
    pub timezone: i32,
    pub id: u32,
    pub name: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Coord {
    pub lat: f32,
    pub lon: f32,
}

#[derive(serde::Deserialize, Debug)]
pub struct Weather {
    pub id: u16,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Main {
    pub temp: f32,
    pub feels_like: f32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub pressure: u16,
    pub humidity: u8,
}

#[derive(serde::Deserialize, Debug)]
pub struct Wind {
    pub speed: f32,
    pub deg: Option<f32>,
    pub gale: Option<f32>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Rain {
    #[serde(rename(deserialize = "1h"))]
    pub one_h: Option<f32>,
    #[serde(rename(deserialize = "3h"))]
    pub three_h: Option<f32>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Snow {
    #[serde(rename(deserialize = "1h"))]
    pub one_h: Option<f32>,
    #[serde(rename(deserialize = "3h"))]
    pub three_h: Option<f32>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Clouds {
    pub all: u16,
}

#[derive(serde::Deserialize, Debug)]
pub struct Sys {
    pub country: String,
    pub sunrise: i64,
    pub sunset: i64,
}
