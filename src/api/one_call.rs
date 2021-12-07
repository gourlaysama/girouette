#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum ApiResponse {
    Success(Box<OneCallResponse>),
    // hack: Openweather API returns some very ugly json
    OtherInt { cod: u16, message: String },
    OtherString { cod: String, message: String },
}

#[derive(serde::Deserialize, Debug)]
pub struct OneCallResponse {
    pub lat: f32,
    pub lon: f32,
    pub timezone_offset: i32,
    pub current: WeatherData,
    pub minutely: Option<Vec<MinutelyForecast>>,
    pub hourly: Option<Vec<WeatherData>>,
    pub daily: Option<Vec<WeatherData>>,
    pub alerts: Option<Vec<Alert>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct WeatherData {
    pub dt: i64,
    pub sunrise: Option<i64>,
    pub sunset: Option<i64>,
    pub temp: Temperature,
    pub feels_like: FeelsLike,
    pub pressure: u16,
    pub humidity: u8,
    pub clouds: u16,
    pub visibility: Option<u16>,
    pub wind_speed: f32,
    pub wind_deg: Option<f32>,
    pub wind_gust: Option<f32>,
    pub pop: Option<f32>,
    pub rain: Option<RainResult>,
    pub snow: Option<SnowResult>,
    pub weather: Vec<super::Weather>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum RainResult {
    Value(f32),
    Values(super::Rain),
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum SnowResult {
    Value(f32),
    Values(super::Snow),
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum Temperature {
    Value(f32),
    Values(TempValues),
}

#[derive(serde::Deserialize, Debug)]
pub struct TempValues {
    pub morn: f32,
    pub day: f32,
    pub eve: f32,
    pub night: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum FeelsLike {
    Value(f32),
    Values(FeelsLikeValues),
}

#[derive(serde::Deserialize, Debug)]
pub struct FeelsLikeValues {
    pub morn: f32,
    pub day: f32,
    pub eve: f32,
    pub night: f32,
}

#[derive(serde::Deserialize, Debug)]
pub struct MinutelyForecast {
    pub dt: i64,
    pub precipitation: f32,
}

#[derive(serde::Deserialize, Debug)]
pub struct Alert {
    pub sender_name: String,
    pub event: String,
    pub start: i64,
    pub end: i64,
    pub description: String,
    pub tags: Vec<String>,
}
