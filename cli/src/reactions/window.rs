use crate::{
    bluetooth::{self, FromBleData},
    climate_data::ClimateData,
    config::*,
    reactions::{data_reaction::Trend, DataReaction},
};
use btleplug::api::Peripheral;
use chrono::Timelike;
use std::{error::Error, str::FromStr, time::Duration};
use uuid::Uuid;

#[derive(Debug)]
pub struct WindowState {
    pub is_closed: bool,
}

impl FromBleData for WindowState {
    fn from_bytes(data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            is_closed: data[0] == 1,
        })
    }
}

impl WindowState {
    async fn find_connection() -> Result<bluetooth::Connection<impl Peripheral>, Box<dyn Error>> {
        for _ in 0..30 {
            match bluetooth::connect_to(
                &BLE_WINDOW_SERVICE_LOCAL_NAME,
                Uuid::from_str(&BLE_WINDOW_SENSOR_SERVICE)?,
            )
            .await
            {
                Ok(connection) => return Ok(connection),
                Err(_) => continue,
            }
        }

        Err("Could not find window sensor".into())
    }

    pub async fn fetch_state() -> Result<WindowState, Box<dyn Error>> {
        let connection = Self::find_connection().await?;
        let data = connection
            .read_from_sensor::<WindowState>(Uuid::from_str(&BLE_WINDOW_SENSOR_SERVICE)?)
            .await?;
        connection.disconnect().await?;

        Ok(data)
    }
}

#[async_trait::async_trait]
impl DataReaction<f32> for WindowState {
    const PERIOD: Duration = Duration::from_secs(600);
    const TREND: Trend = Trend::Up;

    fn get_value(data: &ClimateData) -> f32 {
        data.light.unwrap_or(0.)
    }

    fn force_run(latest_data: &ClimateData) -> bool {
        latest_data.light.unwrap_or(0.) > 1200.
    }

    fn only_if(latest_data: &ClimateData) -> bool {
        tracing::debug!("Check if required to check window data");
        let hour = chrono::Local::now().hour();

        hour > 8 && hour < 20 && latest_data.light.unwrap_or(0.0) > 800.
    }

    async fn run() -> Result<(), Box<dyn Error>> {
        tracing::info!("Run window reaction");
        let data = WindowState::fetch_state().await?;
        tracing::info!("Window state: {:?}", data);

        if data.is_closed {
            notify_rust::Notification::new()
                .summary("Time to close the blinds")
                .body("It's too bright")
                .show()?;
        }

        Ok(())
    }
}
