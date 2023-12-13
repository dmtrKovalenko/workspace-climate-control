use crate::bluetooth::FromBleData;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ClimateData {
    pub co2: i32,
    pub temperature: f32,
    pub eco2: i16,
    pub etvoc: i16,
    pub pressure: f32,
    pub humidity: f32,
    pub light: Option<f32>,
}

impl FromBleData for ClimateData {
    fn from_bytes(data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        Ok(serde_json::from_slice(&data)?)
    }
}
