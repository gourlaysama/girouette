pub mod current;

#[derive(serde::Deserialize, Debug)]
pub struct Weather {
    pub id: u16,
    pub main: String,
    pub description: String,
    pub icon: String,
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