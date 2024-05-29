mod ble_actions;
mod config;
mod history;
mod tui_app;
use climate_data::ClimateData;
use crossterm::{
    terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use history::History;
use spinners::{Spinner, Spinners};
use tui_app::TerminalUi;
use uuid::Uuid;
mod bluetooth;
use config::*;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    error::Error,
    fmt::Display,
    io::stdout,
    str::FromStr,
    sync::{Arc, RwLock},
};

use crate::ble_actions::run_ble_mpsc;

mod climate_data;
mod reactions;

fn set_terminal_tab_title(climate_data: impl AsRef<str> + Display) {
    use std::io::Write;

    print!("\x1B]0;{}\x07", climate_data);
    if let Err(e) = std::io::stdout().flush() {
        tracing::error!("Failed to update title of the console: {:?}", e);
    }
}

#[tokio::main()]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_appender = tracing_appender::rolling::hourly("/tmp/co2nsole", "cli.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(tracing::Level::TRACE)
        .pretty()
        .init();

    let backend = CrosstermBackend::new(stdout());
    let history = Arc::new(RwLock::new(History::new()));
    let mut terminal = Terminal::new(backend)?;
    let mut app = TerminalUi::new(Arc::clone(&history))?;

    loop {
        let mut spinner_stopped = false;
        let mut spinner = Spinner::new(Spinners::Pong, "Connecting to sensor".to_owned());
        tracing::debug!("Looking for a sensor...");
        set_terminal_tab_title("Connecting to a sensor...");

        if let Ok(connection) = bluetooth::connect_to(
            &BLE_MAIN_SERVICE_LOCAL_NAME,
            Uuid::from_str(&BLE_MAIN_SENSOR_SERVICE)?,
        )
        .await
        {
            stdout().execute(EnterAlternateScreen)?;
            crossterm::terminal::enable_raw_mode()?;
            let (tx, rx) = tokio::sync::mpsc::channel(100);

            // Exit of the app can happen only from the event poller:
            TerminalUi::start_event_polling(app.state.clone(), tx);

            let ble_action_handler = run_ble_mpsc(&connection, rx);
            let ble_subscription = connection.subscribe(
                Uuid::from_str(&BLE_MAIN_SENSOR_STREAM_CHAR)?,
                |data: ClimateData| {
                    tracing::debug!("New climate data: {:?}", data);
                    if !spinner_stopped {
                        spinner.stop();
                        terminal.clear().unwrap();
                        spinner_stopped = true
                    }

                    set_terminal_tab_title(format!(
                        "T {:.2}°C; CO2 {} ppm; H {}%",
                        data.temperature,
                        data.co2.unwrap_or(400),
                        data.humidity.round()
                    ));

                    {
                        history.write().unwrap().capture_measurement(&data);
                    }

                    app.capture_measurements(&data);
                    app.draw(&mut terminal);

                    if cfg!(debug_assertions) {
                        reactions::run_reactions(history.read().unwrap().flat.as_slice());
                    }
                },
            );

            match tokio::try_join!(ble_action_handler, ble_subscription) {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Error in BLE connection: {:?}", e);
                }
            }

            connection.disconnect_with_timeout().await;
            stdout().execute(LeaveAlternateScreen)?;
            disable_raw_mode()?;
        }

        terminal.clear()?;
    }
}
