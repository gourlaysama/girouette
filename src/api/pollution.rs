use super::*;

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiResponse {
    Success(PollutionResponse),
    // hack: Openweather API returns some very ugly json
    OtherInt { cod: u16, message: String },
    OtherString { cod: String, message: String },
}

#[derive(serde::Deserialize, Debug)]
pub struct PollutionResponse {
    pub coord: Coord,
    pub list: Vec<PollutionData>,
}

#[derive(serde::Deserialize, Debug)]
pub struct PollutionData {
    pub dt: i64,
    pub main: AQIndex,
    pub components: Components,
}

#[derive(serde::Deserialize, Debug)]
pub struct AQIndex {
    pub aqi: u16
}


#[derive(serde::Deserialize, Debug)]
pub struct Components {
    pub co: f64,
    pub no: f64,
    pub no2: f64,
    pub o3: f64,
    pub so2: f64,
    pub pm2_5: f64,
    pub pm10: f64,
    pub nh3: f64,
}