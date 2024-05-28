use ratatui::{
    layout::{Alignment, Rect},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::climate_data::ClimateData;

fn create_dumb_climate_advice(&climate_data: &ClimateData) -> String {
    let mut advice = String::from("\n");
    if climate_data.co2 > Some(1000) {
        advice.push_str("Open the window, it's getting stuffy in here!");
    }
    if climate_data.temperature > 25.0 {
        advice.push_str("It's getting hot in here, open the window!");
    }
    if climate_data.humidity > 70.0 {
        advice.push_str("It's getting humid in here, open the window!");
    }
    if climate_data.co2 < Some(800) {
        advice.push_str("It's getting cold in here, close the window!");
    }
    if climate_data.temperature < 20.0 {
        advice.push_str("It's getting cold in here, close the window!");
    }
    if climate_data.humidity < 30.0 {
        advice.push_str("It's getting dry in here, close the window!");
    }

    advice
}

pub fn render_dumb_advice_block(climate_data: &ClimateData, area: &Rect, frame: &mut Frame) {
    let paragraph = Paragraph::new(Text::from(create_dumb_climate_advice(climate_data)))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Advice you didn't ask for ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
        );

    frame.render_widget(paragraph, *area);
}
