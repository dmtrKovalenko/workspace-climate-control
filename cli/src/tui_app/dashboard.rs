use super::{
    buttons::render_buttons,
    chart::{render_chart, ChartOptions},
    dumb_advice::render_dumb_advice_block,
};
use crate::{climate_data::ClimateData, history::History};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Dataset, Paragraph, Wrap},
    Frame,
};

pub struct DashboardView {}

impl DashboardView {
    fn render_overview(last_climate_data: &ClimateData, f: &mut Frame, area: Rect) {
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
    pub fn render_dashboard(history: &History, f: &mut Frame) {
        let size = f.size();

        let latest_climate_data = if let Some(latest_climate_data) = history.latest_climate_data {
            latest_climate_data
        } else {
            return;
        };

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .spacing(1)
            .constraints(vec![
                Constraint::Length(10),
                Constraint::Max(70),
                Constraint::Percentage(40),
            ])
            .split(size);

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .spacing(1)
            .constraints(match size.width {
                0..=30 => vec![Constraint::Fill(1)],
                31..=50 => vec![Constraint::Percentage(50), Constraint::Percentage(50)],
                _ => vec![
                    Constraint::Ratio(1, 3),
                    Constraint::Min(30),
                    Constraint::Ratio(1, 3),
                ],
            })
            .split(main_layout[0]);

        Self::render_overview(&latest_climate_data, f, top_layout[1]);
        if let Some(advice_layout) = top_layout.first() {
            render_dumb_advice_block(&latest_climate_data, advice_layout, f)
        }

        if let Some(buttons_layout) = top_layout.get(2) {
            render_buttons(*buttons_layout, f)
        }

        if let Some(co2_layout) = main_layout.get(1) {
            render_chart(
                f,
                ChartOptions {
                    unit_of_measurement: "ppm",
                    current_measure: latest_climate_data.co2,
                    label: "CO2",
                    color: Color::Cyan,
                    bounds: [400.0, 2000.],
                    area: *co2_layout,
                    window: history.eco2_history.get_window(|(ts, _)| *ts),
                    datasets: vec![
                        Dataset::default()
                            .name("eCO2 ppm")
                            .marker(symbols::Marker::Braille)
                            .style(Style::default().fg(Color::Gray))
                            .data(history.eco2_history.as_ratatui_dataset()),
                        Dataset::default()
                            .name("CO2 ppm")
                            .marker(symbols::Marker::Braille)
                            .style(Style::default().fg(Color::Cyan))
                            .data(history.co2_history.as_ratatui_dataset()),
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
                f,
                ChartOptions {
                    unit_of_measurement: "°C",
                    label: "Temperature",
                    current_measure: Some(latest_climate_data.temperature),
                    color: Color::LightRed,
                    window: history.temperature_history.get_window(|(ts, _)| *ts),
                    bounds: history
                        .temperature_minmax
                        .as_ref()
                        .map(|r| [(r.start - 3.).floor(), (r.end + 3.).ceil()])
                        .unwrap_or([0.0, 40.0]),
                    area: horizontal_charts_layout[0],
                    datasets: vec![Dataset::default()
                        .name("°C")
                        .marker(symbols::Marker::Braille)
                        .style(Style::default().fg(Color::LightRed))
                        .data(history.temperature_history.as_ratatui_dataset())],
                },
            );

            if let Some(pressure_layout) = horizontal_charts_layout.get(1) {
                render_chart(
                    f,
                    ChartOptions {
                        unit_of_measurement: "hPa",
                        current_measure: Some(latest_climate_data.pressure),
                        label: "Atmospheric Pressure",
                        color: Color::Blue,
                        window: history.pressure_history.get_window(|(ts, _)| *ts),
                        bounds: history
                            .pressure_minmax
                            .as_ref()
                            .map(|r| [(r.start - 5.).floor(), (r.end + 5.).ceil()])
                            .unwrap_or([950.0, 1050.0]),
                        area: *pressure_layout,
                        datasets: vec![Dataset::default()
                            .name("hectoPascals")
                            .marker(symbols::Marker::Bar)
                            .style(Style::default().fg(Color::Blue))
                            .data(history.pressure_history.as_ratatui_dataset())],
                    },
                );
            }
        }
    }
}
