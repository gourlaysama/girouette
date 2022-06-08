use super::*;

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum ApiResponse {
    Success(CurrentResponse),
    // hack: Openweather API returns some very ugly json
    OtherInt { cod: u16, message: String },
    OtherString { cod: String, message: String },
}

#[derive(serde::Deserialize, Debug)]
pub struct CurrentResponse {
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
pub struct Clouds {
    pub all: u16,
}

#[derive(serde::Deserialize, Debug)]
pub struct Sys {
    pub country: String,
    pub sunrise: i64,
    pub sunset: i64,
}
