mod tui_app;
use tui_app::TerminalUi;
mod bluetooth;
use crate::bluetooth::find_sensor;
use std::error::Error;
use tui::{backend::CrosstermBackend, Terminal};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut app = TerminalUi::new()?;
    let mut terminal = Terminal::new(backend)?;

    loop {
        // let mut spinner = Spinner::new(Spinners::Dots9, "Connecting to sensor".to_owned());
        tracing::debug!("Looking for a sensor...");

        if let Ok(connection) = find_sensor().await {
            let mut spinner_stopped = false;
            match connection
                .subscribe_to_sensor(|data| {
                    if !spinner_stopped {
                        // spinner.stop();
                        terminal.clear().unwrap();
                        spinner_stopped = true
                    }

                    app.capture_measurements(data);
                    app.draw(&mut terminal);
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
