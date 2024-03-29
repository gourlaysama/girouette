use anyhow::*;

pub mod current;
pub mod one_call;
pub mod pollution;

#[derive(serde::Deserialize, Debug)]
pub struct Weather {
    pub id: u16,
    pub main: String,
    pub description: Option<String>,
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

#[derive(serde::Deserialize, Debug)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug)]
pub struct Response {
    current: Option<current::CurrentResponse>,
    forecast: Option<one_call::OneCallResponse>,
    pollution: Option<pollution::PollutionResponse>,
}

impl Response {
    pub fn from_current(current: current::CurrentResponse) -> Self {
        Self {
            current: Some(current),
            forecast: None,
            pollution: None
        }
    }

    pub fn from_forecast(forecast: one_call::OneCallResponse) -> Self {
        Self {
            current: None,
            forecast: Some(forecast),
            pollution: None,
        }
    }

    pub fn from_pollution(pollution: pollution::PollutionResponse) -> Self {
        Self {
            current: None,
            forecast: None,
            pollution: Some(pollution),
        }
    }

    pub fn empty() -> Self {
        Self {
            current: None,
            forecast: None,
            pollution: None
        }
    }

    pub fn merge(&mut self, other: Response) {
        if let Some(c) = other.current {
            self.current = Some(c);
        }
        if let Some(f) = other.forecast {
            self.forecast = Some(f);
        }
        if let Some(f) = other.pollution {
            self.pollution = Some(f);
        }
    }

    pub fn as_current(&self) -> Result<&current::CurrentResponse> {
        self.current
            .as_ref()
            .ok_or_else(|| anyhow!("internal error: missing current api data"))
    }

    pub fn as_forecast(&self) -> Result<&one_call::OneCallResponse> {
        self.forecast
            .as_ref()
            .ok_or_else(|| anyhow!("internal error: missing forecast api data"))
    }

    pub fn as_pollution(&self) -> Result<&pollution::PollutionResponse> {
        self.pollution
            .as_ref()
            .ok_or_else(|| anyhow!("internal error: missing pollution api data"))
    }
}
