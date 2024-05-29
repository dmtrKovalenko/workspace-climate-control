mod buttons;
mod calibration_popup;
mod chart;
mod dashboard;
mod dumb_advice;

use self::{
    buttons::handle_dashboard_key_event, calibration_popup::CalibrationPopup,
    dashboard::DashboardView,
};
use crate::{ble_actions::BleAction, history::History};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    error::Error,
    io::Stdout,
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
};
use tokio::sync::mpsc::Sender;

pub enum View {
    Dashboard,
    Calibrate(CalibrationPopup),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Reconnect,
    Exit,
    ClearHistory,
    OpenDashboard,
    OpenCalibrateCo2Popup,
    OpenCalibrateTemperaturePopup,
    CalibrateCo2,
}

pub struct TerminalUi {
    pub state: Arc<RwLock<View>>,
    history: Arc<RwLock<History>>,
}

impl TerminalUi {
    pub fn start_event_polling(
        self: Arc<Self>,
        ble_sender: Sender<BleAction>,
        terminal: Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>>,
    ) -> tokio::task::JoinHandle<std::io::Result<()>> {
        let me = Arc::clone(&self);
        tokio::task::spawn(async move {
            loop {
                if crossterm::event::poll(std::time::Duration::from_millis(16))? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('c')
                                if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                            {
                                // a bit of ugly way to exit but we do not deal with any
                                // graceful resource cleanup as system will do that for us
                                // and we can encapsulate this thread in the terminal ui
                                std::process::exit(0);
                            }
                            keycode => {
                                let action = {
                                    let state_read = &me.state.read().unwrap();

                                    match state_read.deref() {
                                        View::Dashboard => handle_dashboard_key_event(keycode),
                                        View::Calibrate(ref popup) => popup.handle_key(key),
                                    }
                                };

                                match action {
                                    Some(Action::OpenCalibrateTemperaturePopup) => {
                                        *me.state.write().unwrap() =
                                            View::Calibrate(CalibrationPopup::temperature());
                                    }
                                    Some(Action::OpenCalibrateCo2Popup) => {
                                        *me.state.write().unwrap() =
                                            View::Calibrate(CalibrationPopup::co2());
                                    }
                                    Some(Action::CalibrateCo2) => {
                                        if let Err(err) =
                                            ble_sender.send(BleAction::CalibrateCo2).await
                                        {
                                            tracing::error!(
                                                "Failed to send CO2 calibration request: {:?}",
                                                err
                                            );
                                        }

                                        *me.state.write().unwrap() = View::Dashboard
                                    }
                                    Some(Action::Reconnect) => {
                                        ble_sender.send(BleAction::Stop).await.unwrap();
                                    }
                                    Some(Action::ClearHistory) => {
                                        todo!()
                                    }
                                    Some(Action::Exit) => {
                                        std::process::exit(0);
                                    }
                                    Some(Action::OpenDashboard) => {
                                        *me.state.write().unwrap() = View::Dashboard
                                    }
                                    None => {}
                                }
                            }
                        }

                        me.draw(&mut terminal.lock().unwrap());
                    }
                }
            }
        })
    }

    pub fn new(history: Arc<RwLock<History>>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            history,
            state: Arc::new(RwLock::new(View::Dashboard)),
        })
    }

    pub fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) {
        if cfg!(debug_assertions) && option_env!("RUST_LOG") == Some("debug") {
            return;
        }

        terminal
            .draw(|f| {
                match *self.state.read().unwrap() {
                    View::Dashboard => {
                        DashboardView::render_dashboard(&self.history.read().unwrap(), f)
                    }
                    View::Calibrate(ref popup) => {
                        DashboardView::render_dashboard(&self.history.read().unwrap(), f);
                        popup.render(f);
                    }
                };
            })
            .unwrap();
    }
}
