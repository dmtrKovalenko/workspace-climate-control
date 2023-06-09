use crossterm::event::{self, Event, KeyCode};
use std::error::Error;
use tui::{
    backend::Backend,
    layout::{self, Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::bluetooth::ClimateData;

pub enum UiState {
    Spinner(String),
    Connected,
}

pub struct TerminalUi {
    data: Vec<(f64, f64)>,
    last_climate_data: Option<ClimateData>,
    window: [f64; 2],
    state: UiState,
}

impl TerminalUi {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let now = chrono::offset::Local::now().timestamp_millis() as f64;

        Ok(Self {
            data: vec![],
            window: [now, now],
            last_climate_data: None,
            state: UiState::Spinner("Connecting to sensor...".to_string()),
        })
    }

    pub fn capture_measurements(&mut self, climate_data: ClimateData) {
        let now = chrono::offset::Local::now().timestamp_millis();

        self.state = UiState::Connected;
        self.data.push((now as f64, climate_data.e_co2 as f64));
        self.window[1] = now as f64;
        self.last_climate_data = Some(climate_data);
    }

    fn render_dashboard<B: Backend>(&self, f: &mut Frame<B>) {
        let app = self;
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)].as_ref())
            .split(size);

        if let Some(climate_overview) = self.last_climate_data.as_ref() {
            self.render_overview(climate_overview, f, &chunks);
        };

        let start_time = chrono::NaiveDateTime::from_timestamp_millis(app.window[0] as i64)
            .map(|time| time.format("%H:%M:%S").to_string())
            .unwrap_or("now".to_string());

        let end_time = chrono::NaiveDateTime::from_timestamp_millis(app.window[1] as i64)
            .map(|time| time.format("%H:%M:%S").to_string())
            .unwrap_or("now".to_string());

        let x_labels = vec![
            Span::styled(
                start_time.as_str(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                end_time.as_str(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];
        let datasets = vec![Dataset::default()
            .name("ppm")
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Cyan))
            .data(&app.data)];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        " eCO2 ppm ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("time")
                    .style(Style::default().fg(Color::Gray))
                    .labels(x_labels)
                    .bounds(app.window),
            )
            .y_axis(
                Axis::default()
                    .title("ppm")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![
                        Span::styled("400", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("1400.0", Style::default().add_modifier(Modifier::BOLD)),
                    ])
                    .bounds([400.0, 1400.0]),
            );
        f.render_widget(chart, chunks[1]);
    }

    fn render_placeholder(&self, title: &str, f: &mut Frame<impl Backend>) {
        let text = vec![
            Spans::from(vec![
                Span::raw("First"),
                Span::styled("line", Style::default().add_modifier(Modifier::ITALIC)),
                Span::raw("."),
            ]),
            Spans::from(Span::styled("Second line", Style::default().fg(Color::Red))),
        ];

        let block = Paragraph::new(text)
            .block(Block::default().title("Paragraph").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(block, f.size());
    }

    fn render_view<B: Backend>(&self, f: &mut Frame<B>) {
        match &self.state {
            UiState::Spinner(title) => self.render_placeholder(title.as_str(), f),
            UiState::Connected => {
                self.render_dashboard(f);
            }
        }
    }

    pub fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) {
        if cfg!(debug_assertions) && option_env!("RUST_LOG") == Some("debug") {
            return;
        }

        terminal
            .draw(|f| {
                self.render_view(f);
            })
            .unwrap();
    }

    pub fn poll_interactions(&self, timeout: core::time::Duration) {
        loop {
            if crossterm::event::poll(timeout).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if let KeyCode::Char('q') = key.code {
                        std::process::exit(0);
                    }
                }
            }
        }
    }

    fn render_overview<B: Backend>(
        &self,
        last_climate_data: &ClimateData,
        f: &mut Frame<B>,
        chunks: &[layout::Rect],
    ) {
        let text = vec![
            Spans::from(vec![
                Span::from("Temperature: "),
                Span::styled(
                    format!("{} °C", last_climate_data.temperature),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Spans::from(vec![
                Span::from("Humidity: "),
                Span::styled(
                    format!("{}%", last_climate_data.humidity),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Spans::from(vec![
                Span::from("Pressure: "),
                Span::styled(
                    format!("{} pa", last_climate_data.pressure),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Spans::from(vec![
                Span::from("TVOC: "),
                Span::styled(
                    format!("{} ppb", last_climate_data.tvoc),
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Spans::from(vec![
                Span::from("eCO2: "),
                Span::styled(
                    format!("{} ppm", last_climate_data.e_co2),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Spans::from(vec![
                Span::from("Light: "),
                Span::styled(
                    format!("{} lux ", last_climate_data.light),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::from(match last_climate_data.light {
                    light if light > 400.0 => "🌞",
                    light if light > 100.0 => "️⛅",
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
        f.render_widget(paragraph, chunks[0]);
    }
}

// fn run_app<B: Backend>(
//     terminal: &mut Terminal<B>,
//     mut app: TerminalUi,
//     tick_rate: Duration,
// ) -> io::Result<()> {
//     let mut last_tick = Instant::now();
//     loop {
//         terminal.draw(|f| ui(f, &app))?;

//         let timeout = tick_rate
//             .checked_sub(last_tick.elapsed())
//             .unwrap_or_else(|| Duration::from_secs(0));
//         if crossterm::event::poll(timeout)? {
//             if let Event::Key(key) = event::read()? {
//                 if let KeyCode::Char('q') = key.code {
//                     return Ok(());
//                 }
//             }
//         }
//         if last_tick.elapsed() >= tick_rate {
//             app.on_tick();
//             last_tick = Instant::now();
//         }
//     }
// }
