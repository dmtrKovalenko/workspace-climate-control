use crate::climate_data::Timestamp;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset},
    Frame,
};
use std::fmt::Display;

pub struct ChartOptions<'a, TMeasure: Display> {
    pub unit_of_measurement: &'static str,
    pub label: &'static str,
    pub current_measure: Option<TMeasure>,
    pub color: Color,
    pub datasets: Vec<Dataset<'a>>,
    pub bounds: [f64; 2],
    pub window: Option<[Timestamp; 2]>,
    pub area: Rect,
}

pub fn render_chart<TMeasure: Display>(frame: &mut Frame, opts: ChartOptions<TMeasure>) {
    let ChartOptions {
        area,
        bounds,
        color,
        datasets,
        label,
        unit_of_measurement,
        current_measure,
        window,
    } = opts;
    let window = window.unwrap_or([Timestamp::default(), Timestamp::default()]);

    let start_time = window[0]
        .format("%H:%M:%S")
        .unwrap_or("first measurement".to_string());
    let end_time = window[1]
        .format("%H:%M:%S")
        .unwrap_or("first measurement".to_string());

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

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title_alignment(Alignment::Center)
                .title(Span::styled(
                    format!(
                        " {label} {} {unit_of_measurement} ",
                        match current_measure {
                            Some(measure) => format!("{measure:.2}"),
                            None => "N/A".to_string(),
                        },
                        label = label
                    ),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("time")
                .style(Style::default().fg(Color::Gray))
                .labels(x_labels)
                .bounds([window[0].as_f64(), window[1].as_f64()]),
        )
        .y_axis(
            Axis::default()
                .title(unit_of_measurement)
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled(
                        format!("{:.0}", bounds[0]),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:.0}", bounds[1]),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ])
                .bounds(bounds),
        );

    frame.render_widget(chart, area);
}
