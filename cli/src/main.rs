mod history;
mod tui_app;
use btleplug::api::CharPropFlags;
use climate_data::ClimateData;
use crossterm::{
    terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use futures::FutureExt;
use history::History;
use spinners::{Spinner, Spinners};
use tui_app::TerminalUi;
use uuid::Uuid;
mod bluetooth;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, fmt::Display, io::stdout, os::unix::process};

mod climate_data;
mod reactions;

const CORE_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x0000FFE1_0000_1000_8000_00805F9B34FB);

fn set_terminal_tab_title(climate_data: impl AsRef<str> + Display) {
    use std::io::Write;

    print!("\x1B]0;{}\x07", climate_data);
    if let Err(e) = std::io::stdout().flush() {
        tracing::error!("Failed to update title of the console: {:?}", e);
    }
}

#[tokio::main()]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_appender = tracing_appender::rolling::hourly("/tmp/co2cicka", "cli.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(tracing::Level::TRACE)
        .pretty()
        .init();

    let backend = CrosstermBackend::new(stdout());
    let mut history = History::new();
    let mut terminal = Terminal::new(backend)?;
    let mut app = TerminalUi::new()?;

    loop {
        let mut spinner_stopped = false;
        let mut spinner = Spinner::new(Spinners::Pong, "Connecting to sensor".to_owned());
        tracing::debug!("Looking for a sensor...");
        set_terminal_tab_title("Connecting to a sensor...");

        if let Ok(connection) = bluetooth::find_sensor(
            "CO2CICKA Sensor",
            CORE_CHARACTERISTIC_UUID,
            CharPropFlags::NOTIFY,
        )
        .await
        {
            stdout().execute(EnterAlternateScreen)?;
            crossterm::terminal::enable_raw_mode()?;
            // Exit of the app can happen only from the event poller:
            app.start_event_polling();

            let result = connection
                .subscribe_to_sensor(|data: ClimateData| {
                    tracing::debug!("New climate data: {:?}", data);
                    if !spinner_stopped {
                        spinner.stop();
                        terminal.clear().unwrap();
                        spinner_stopped = true
                    }

                    set_terminal_tab_title(format!(
                        "T {}°C; CO2 {} ppm; H {}%",
                        data.temperature,
                        data.co2.unwrap_or(400),
                        data.humidity.round()
                    ));

                    history.capture_measurement(&data);

                    app.capture_measurements(&data);
                    app.draw(&history, &mut terminal);

                    if cfg!(debug_assertions) {
                        reactions::run_reactions(history.flat.as_slice());
                    }
                })
                .await;

            match result {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(error=?e, "Error while subscribing to sensor");
                }
            }

            connection.disconnect_with_timeout().await;
            stdout().execute(LeaveAlternateScreen)?;
            disable_raw_mode()?;
        }

        terminal.clear()?;
    }
}
