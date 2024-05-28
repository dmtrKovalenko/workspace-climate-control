mod buttons;
mod chart;
mod dumb_advice;

use crate::{climate_data::ClimateData, history::History};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Dataset, Paragraph, Wrap},
    Frame, Terminal,
};

use self::{
    buttons::{match_button, render_buttons},
    chart::{render_chart, ChartOptions},
    dumb_advice::render_dumb_advice_block,
};
use std::{
    error::Error,
    sync::{Arc, RwLock},
};

pub enum UiState {
    Spinner(String),
    Connected,
}

pub struct TerminalUi {
    last_climate_data: Option<ClimateData>,
    state: Arc<RwLock<UiState>>,
}

impl TerminalUi {
    pub fn start_event_polling(&self) -> tokio::task::JoinHandle<std::io::Result<()>> {
        tokio::task::spawn_blocking(|| -> std::io::Result<()> {
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
                            KeyCode::Char(char) => {
                                let action = match_button(char);

                                if let Some(action) = action {
                                    todo!("Handle the button {:?} action", action);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        })
    }

    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            last_climate_data: None,
            state: Arc::new(RwLock::new(UiState::Spinner(
                "Connecting to sensor...".to_string(),
            ))),
        })
    }

    pub fn capture_measurements(&mut self, climate_data: &ClimateData) {
        *self.state.write().unwrap() = UiState::Connected;
        self.last_climate_data = Some(*climate_data);
    }

    fn render_dashboard(&self, history: &History, f: &mut Frame) {
        let size = f.size();
        let latest_climate_data = if let Some(latest_climate_data) = self.last_climate_data {
            latest_climate_data
        } else {
            return;
        };

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .spacing(1)
            .constraints(match size.height {
                0..=20 => vec![Constraint::Length(10)],
                21..=60 => vec![Constraint::Length(10), Constraint::Fill(1)],
                _ => vec![
                    Constraint::Length(10),
                    Constraint::Max(70),
                    Constraint::Percentage(40),
                ],
            })
            .split(size);

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .spacing(1)
            .constraints(match size.width {
                0..=40 => vec![Constraint::Fill(1)],
                41..=60 => vec![Constraint::Percentage(50), Constraint::Percentage(50)],
                _ => vec![
                    Constraint::Ratio(1, 3),
                    Constraint::Min(30),
                    Constraint::Ratio(1, 3),
                ],
            })
            .split(main_layout[0]);

        self.render_overview(&latest_climate_data, f, top_layout[1]);
        if let Some(advice_layout) = top_layout.first() {
            render_dumb_advice_block(&latest_climate_data, advice_layout, f)
        }

        if let Some(buttons_layout) = top_layout.get(2) {
            render_buttons(*buttons_layout, f)
        }

        if let Some(co2_layout) = main_layout.get(1) {
            render_chart(
                history,
                f,
                ChartOptions {
                    unit_of_measurement: "ppm",
                    current_measure: latest_climate_data.co2,
                    label: "CO2",
                    color: Color::Cyan,
                    bounds: [400.0, 2000.],
                    area: *co2_layout,
                    datasets: vec![
                        Dataset::default()
                            .name("eCO2 ppm")
                            .marker(symbols::Marker::Braille)
                            .style(Style::default().fg(Color::Gray))
                            .data(history.eco2_history.as_slice()),
                        Dataset::default()
                            .name("CO2 ppm")
                            .marker(symbols::Marker::Braille)
                            .style(Style::default().fg(Color::Cyan))
                            .data(history.co2_history.as_slice()),
                    ],
                },
            );
        }

        if let Some(horizontal_layout) = main_layout.get(2) {
            let horizontal_charts_layout = Layout::default()
                .direction(if horizontal_layout.width > 80 {
                    Direction::Horizontal
                } else {
                    Direction::Vertical
                })
                .spacing(2)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(*horizontal_layout);

            render_chart(
                history,
                f,
                ChartOptions {
                    unit_of_measurement: "°C",
                    label: "Temperature",
                    current_measure: Some(latest_climate_data.temperature),
                    color: Color::LightRed,
                    bounds: [0.0, 40.0],
                    area: horizontal_charts_layout[0],
                    datasets: vec![Dataset::default()
                        .name("°C")
                        .marker(symbols::Marker::Braille)
                        .style(Style::default().fg(Color::LightRed))
                        .data(history.temperature_history.as_slice())],
                },
            );

            if let Some(pressure_layout) = horizontal_charts_layout.get(1) {
                render_chart(
                    history,
                    f,
                    ChartOptions {
                        unit_of_measurement: "hPa",
                        current_measure: Some(latest_climate_data.pressure),
                        label: "Atmospheric Pressure",
                        color: Color::Blue,
                        bounds: [760., 1100.],
                        area: *pressure_layout,
                        datasets: vec![Dataset::default()
                            .name("hectoPascals")
                            .marker(symbols::Marker::Bar)
                            .style(Style::default().fg(Color::Blue))
                            .data(history.pressure_history.as_slice())],
                    },
                );
            }
        }
    }

    fn render_placeholder(&self, title: &str, f: &mut Frame) {
        let text = vec![Line::from(Span::styled(
            "Please wait for the sensor to connect..",
            Style::default().fg(Color::Red),
        ))];

        let block = Paragraph::new(text)
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(block, f.size());
    }

    pub fn draw<B: Backend>(&self, history: &History, terminal: &mut Terminal<B>) {
        if cfg!(debug_assertions) && option_env!("RUST_LOG") == Some("debug") {
            return;
        }

        terminal
            .draw(|f| {
                match *self.state.read().unwrap() {
                    UiState::Spinner(ref title) => self.render_placeholder(title.as_str(), f),
                    UiState::Connected => {
                        self.render_dashboard(history, f);
                    }
                };
            })
            .unwrap();
    }

    fn render_overview(&self, last_climate_data: &ClimateData, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::from(" CO2: "),
                Span::styled(
                    format!("{} ppm ", last_climate_data.co2.unwrap_or(400)),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::from(match last_climate_data.co2 {
                    Some(co2) if co2 > 1000 => "🥵",
                    Some(co2) if co2 > 800 => "😨",
                    Some(co2) if co2 > 600 => "😗",
                    Some(co2) if co2 > 400 => "😊",
                    Some(_) => "😌",
                    None => "🤷",
                }),
            ]),
            Line::from(vec![
                Span::from(" eCO2: "),
                Span::styled(
                    format!("{} ppm", last_climate_data.eco2),
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::from(" TVOC: "),
                Span::styled(
                    format!("{:.0} ppb", last_climate_data.etvoc),
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::from(" Temperature: "),
                Span::styled(
                    format!(
                        "{:.1}°C ({:.2}°F)",
                        last_climate_data.temperature,
                        last_climate_data.temperature * 9.0 / 5.0 + 32.0
                    ),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::from(" Humidity: "),
                Span::styled(
                    format!("{:.1}%", last_climate_data.humidity),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::from(" Pressure: "),
                Span::styled(
                    format!(
                        "{:.2}mm Hg ({:.2} hPa)",
                        last_climate_data.pressure * 0.750_063_8,
                        last_climate_data.pressure,
                    ),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::from(" Light: "),
                Span::styled(
                    format!("{:.0} lux ", last_climate_data.light.unwrap_or(0.0)),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::from(match last_climate_data.light {
                    Some(light) if light > 400.0 => "🌞",
                    Some(light) if light > 100.0 => "️⛅",
                    _ => "🌚",
                }),
            ]),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .title(Span::styled(
                " Climate Right Now ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }
}
