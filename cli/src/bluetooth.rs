use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::error::Error;

use std::time::Duration;
use tokio::time::{self, sleep, timeout};
use uuid::Uuid;

pub struct Connection<TPeripheral: Peripheral> {
    _manager: Manager,
    peripheral: TPeripheral,
    characteristic: btleplug::api::Characteristic,
}

pub trait FromBleData {
    fn from_bytes(data: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
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

        for _ in 1..10 {
            const DISCONNECT_INTERVAL: Duration = Duration::from_secs(2);

            if let Err(e) = timeout(TIMEOUT, self.disconnect()).await {
                tracing::error!(
                    "Error while disconnecting: {e:?}. Will try again in {} seconds",
                    DISCONNECT_INTERVAL.as_secs()
                );
            } else {
                break;
            }

            sleep(DISCONNECT_INTERVAL).await;
        }
    }

    pub async fn subscribe_to_sensor<TData: FromBleData, TFun: FnMut(TData)>(
        &self,
        mut fun: TFun,
    ) -> Result<(), Box<dyn Error>> {
        tracing::debug!("Subscribing to sensor");
        self.peripheral.subscribe(&self.characteristic).await?;
        let mut notification_stream = self.peripheral.notifications().await?;

        while let Some(data) = timeout(TIMEOUT, notification_stream.next()).await? {
            tracing::debug!("Received data from sensor {data:?}");
            match TData::from_bytes(data.value) {
                Ok(data) => fun(data),
                Err(e) => tracing::error!("Error decodring data from sensor {}", e),
            }

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

    pub async fn read_from_sensor<TData: FromBleData>(&self) -> Result<TData, Box<dyn Error>> {
        tracing::debug!("Reading sensor");

        Ok(TData::from_bytes(
            self.peripheral.read(&self.characteristic).await?,
        )?)
    }
}

pub async fn find_sensor(
    name: &str,
    characteristic_uuid: Uuid,
    property: CharPropFlags,
) -> Result<Connection<impl Peripheral>, Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        tracing::debug!("No Bluetooth adapters found");
    }

    for adapter in adapter_list.iter() {
        adapter.start_scan(ScanFilter::default()).await?;

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
                if local_name.contains(name) {
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
                            if characteristic.uuid == characteristic_uuid
                                && characteristic.properties.contains(property)
                            {
                                tracing::debug!("Found characteristic {:?}", characteristic.uuid,);
                                return Ok(Connection {
                                    _manager: manager,
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

        adapter.stop_scan().await?;
    }

    Err("No devices found".into())
}