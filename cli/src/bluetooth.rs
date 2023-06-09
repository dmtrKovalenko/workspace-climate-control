use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::error::Error;

use std::str::FromStr;
use std::time::Duration;
use tokio::time::{self, sleep, timeout};
use uuid::Uuid;

/// Only devices whose name contains this string will be tried.
const PERIPHERAL_NAME_MATCH_FILTER: &str = "CO2CICKA";

#[derive(Debug)]
pub struct ClimateData {
    pub temperature: f32,
    pub e_co2: i32,
    pub tvoc: i32,
    pub pressure: f32,
    pub humidity: f32,
    pub light: f32,
}

impl ClimateData {
    fn from_bytes(data: Vec<u8>) -> Self {
        let str = String::from_utf8(data).unwrap();

        let encoded_data = str.split(',').collect::<Vec<&str>>();
        let co2 = encoded_data[0].parse::<i32>().unwrap_or_default();
        let tvoc = encoded_data[1].parse::<i32>().unwrap_or_default();
        let temp = encoded_data[2].parse::<f32>().unwrap_or_default();
        let pressure = encoded_data[3].parse::<f32>().unwrap_or_default();
        let humidity = encoded_data[4].parse::<f32>().unwrap_or_default();
        let light = encoded_data[5].parse::<f32>().unwrap_or_default();

        Self {
            temperature: temp,
            e_co2: co2,
            tvoc,
            pressure,
            humidity,
            light,
        }
    }
}

pub struct Connection<TPeripheral: Peripheral> {
    manager: Manager,
    peripheral: TPeripheral,
    characteristic: btleplug::api::Characteristic,
}

const TIMEOUT: Duration = Duration::from_secs(10);

impl<TPer: Peripheral> Connection<TPer> {
    pub async fn disconnect(&self) -> Result<(), Box<dyn Error>> {
        tracing::debug!("Disconnecting from sensor");
        self.peripheral.disconnect().await?;

        Ok(())
    }

    pub async fn disconnect_with_timeout(&self) {
        match timeout(TIMEOUT, self.peripheral.is_connected()).await {
            Ok(Ok(false)) => {
                return;
            }
            e => {
                tracing::error!("Can not understand if peripheral is connected: {:?}", e);
            }
        }

        loop {
            if let Err(e) = timeout(TIMEOUT, self.disconnect()).await {
                tracing::error!("Error while disconnecting: {e:?}. Will try again in 5 seconds");
            } else {
                break;
            }

            sleep(Duration::from_secs(5)).await;
        }
    }

    pub async fn subscribe_to_sensor<TFun: FnMut(ClimateData)>(
        &self,
        mut fun: TFun,
    ) -> Result<(), Box<dyn Error>> {
        tracing::debug!("Subscribing to sensor");
        self.peripheral.subscribe(&self.characteristic).await?;

        let mut notification_stream = self.peripheral.notifications().await?;

        while let Some(data) = timeout(TIMEOUT, notification_stream.next()).await? {
            let data = ClimateData::from_bytes(data.value);
            tracing::debug!("Received data from sensor {data:?}");

            fun(data);
            sleep(Duration::from_millis(5000)).await;
            tracing::debug!("Awake");

            let is_connected = timeout(TIMEOUT, self.peripheral.is_connected())
                .await
                .map_err(|_| "Connection lost")??;

            if !is_connected {
                return Err("BLE connection was lost".into());
            }
        }

        Ok(())
    }
}

pub async fn find_sensor() -> Result<Connection<impl Peripheral>, Box<dyn Error>> {
    let notify_characteristic_uuid = Uuid::from_str("0000FFE1-0000-1000-8000-00805F9B34FB")?;

    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        tracing::debug!("No Bluetooth adapters found");
    }

    for adapter in adapter_list.iter() {
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");

        time::sleep(Duration::from_secs(2)).await;
        let peripherals = adapter.peripherals().await?;

        if peripherals.is_empty() {
            tracing::error!("No BLE peripherals found")
        } else {
            // All peripheral devices in range.
            for peripheral in peripherals.into_iter() {
                let properties = peripheral.properties().await?;
                let is_connected = peripheral.is_connected().await?;
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));
                tracing::debug!("Connected to peripheral {:?}.", &local_name);

                // Check if it's the peripheral we want.
                if local_name.contains(PERIPHERAL_NAME_MATCH_FILTER) {
                    if !is_connected {
                        // Connect if we aren't already connected.
                        if let Err(err) = peripheral.connect().await {
                            eprintln!("Error connecting to peripheral, skipping: {}", err);
                            continue;
                        }
                    }
                    let is_connected = peripheral.is_connected().await?;
                    tracing::debug!(
                        "Connected ({:?}) to peripheral {:?}.",
                        is_connected,
                        &local_name
                    );

                    if is_connected {
                        peripheral.discover_services().await?;
                        for characteristic in peripheral.characteristics().into_iter() {
                            if characteristic.uuid == notify_characteristic_uuid {
                                return Ok(Connection {
                                    manager,
                                    peripheral,
                                    characteristic,
                                });
                            }
                        }

                        tracing::debug!(
                            "Peripheral {:?} does not have the required characteristic.",
                            &local_name
                        );
                        peripheral.disconnect().await?;
                    }
                }
            }
        }
    }

    Err("No O found".into())
}
