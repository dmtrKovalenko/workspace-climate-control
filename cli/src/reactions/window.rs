use crate::{
    bluetooth::{self, FromBleData},
    climate_data::ClimateData,
    reactions::{data_reaction::Trend, DataReaction},
};
use btleplug::api::{CharPropFlags, Peripheral};
use chrono::Timelike;
use std::{error::Error, time::Duration};
use tokio::process::Command;
use uuid::Uuid;

const WINDOWS_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x19B10000_E8F2_537E_4F6C_D104768A1214);

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
            match bluetooth::find_sensor(
                WINDOWS_CHARACTERISTIC_UUID,
                CharPropFlags::NOTIFY | CharPropFlags::READ,
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
        let data = connection.read_from_sensor::<WindowState>().await?;
        connection.disconnect().await?;

        Ok(data)
    }
}

#[async_trait::async_trait]
impl DataReaction<f32> for WindowState {
    const PERIOD: Duration = Duration::from_secs(60);
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
            Command::new("shortcuts")
                .args(["run", "Закрой шторы"])
                .output()
                .await?;
        }

        Ok(())
    }
}