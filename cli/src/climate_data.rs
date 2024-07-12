use crate::bluetooth::FromBleData;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Clone, Copy, Deserialize)]
#[repr(transparent)]
pub struct Timestamp(f64);

impl Default for Timestamp {
    fn default() -> Self {
        let now = chrono::offset::Local::now().timestamp_millis() as f64;
        Self(now)
    }
}

impl Timestamp {
    pub fn as_f64(&self) -> f64 {
        self.0
    }

    pub fn format(&self, format_str: &str) -> Option<String> {
        chrono::NaiveDateTime::from_timestamp_millis(self.0 as i64)
            .map(|time| time.format(format_str).to_string())
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ClimateData {
    pub co2: Option<i32>,
    pub temperature: f32,
    pub eco2: i16,
    pub etvoc: i16,
    pub pressure: f32,
    pub humidity: f32,
    pub light: Option<f32>,
    #[serde(default)]
    pub timestamp: Timestamp,
}

impl FromBleData for ClimateData {
    fn from_bytes(data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        Ok(serde_json::from_slice(&data)?)
    }
}
