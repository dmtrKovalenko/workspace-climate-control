use std::{error::Error, str::FromStr};

use crate::bluetooth::Connection;
use crate::config::*;
use btleplug::api::Peripheral;
use tokio::sync::mpsc;
use uuid::Uuid;

#[allow(dead_code)]
pub enum BleAction {
    CalibrateCo2,
    CalibrateTemperature(i32),
    Stop,
}

pub async fn run_ble_mpsc<TPeripheral: Peripheral>(
    connection: &Connection<TPeripheral>,
    mut ble_action_receiver: mpsc::Receiver<BleAction>,
) -> Result<(), Box<dyn Error>> {
    while let Some(action) = ble_action_receiver.recv().await {
        match action {
            BleAction::CalibrateCo2 => {
                tracing::info!("Calibrating CO2 sensor");
                connection
                    .write_to_sennsor(
                        "what the fuck".as_bytes(),
                        Uuid::from_str(&BLE_MAIN_SENSOR_CO2_CALIBRATION_CHAR).unwrap(),
                    )
                    .await?
            }
            BleAction::CalibrateTemperature(_temperature) => {
                tracing::info!("Calibrating temperature sensor");
            }
            BleAction::Stop => {
                connection.disconnect().await?;
                tracing::info!("Stopping BLE actions");
                break;
            }
        }
    }

    Ok(())
}
