mod tui_app;
use btleplug::api::CharPropFlags;
use climate_data::ClimateData;
use spinners::{Spinner, Spinners};
use tui_app::TerminalUi;
use uuid::Uuid;
mod bluetooth;
use std::error::Error;
use tui::{backend::CrosstermBackend, Terminal};

use crate::reactions::run_reactions;
mod climate_data;
mod reactions;

const CORE_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x0000FFE1_0000_1000_8000_00805F9B34FB);

fn set_terminal_title(climate_data: &ClimateData) {
    use std::io::Write;

    print!(
        "\x1B]0;T {}°C; CO2 {} ppm; H {:.1}%\x07",
        climate_data.temperature, climate_data.co2, climate_data.humidity
    );
    std::io::stdout().flush().unwrap();
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut app = TerminalUi::new()?;
    let mut terminal = Terminal::new(backend)?;
    let mut history = Vec::new();

    loop {
        let mut spinner = Spinner::new(Spinners::Dots9, "Connecting to sensor".to_owned());
        tracing::debug!("Looking for a sensor...");

        if let Ok(connection) =
            bluetooth::find_sensor(CORE_CHARACTERISTIC_UUID, CharPropFlags::NOTIFY).await
        {
            let mut spinner_stopped = false;
            match connection
                .subscribe_to_sensor(|data| {
                    tracing::debug!("New climate data: {:?}", data);
                    if !spinner_stopped {
                        spinner.stop();
                        terminal.clear().unwrap();
                        spinner_stopped = true
                    }

                    set_terminal_title(&data);
                    app.capture_measurements(&data);
                    app.draw(&mut terminal);
                    history.push(data);

                    run_reactions(&history);
                })
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    println!("Connection error. Trying to reconnect: {e:?}");
                }
            }

            connection.disconnect_with_timeout().await;
        }

        terminal.clear()?;
    }
}
