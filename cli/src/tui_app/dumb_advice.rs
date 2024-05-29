use ratatui::{
    layout::{Alignment, Rect},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::climate_data::ClimateData;

fn create_dumb_climate_advice(climate_data: &ClimateData) -> String {
    let mut advice = String::from("\n");

    // CO2 levels
    if let Some(co2) = climate_data.co2 {
        if co2 > 1000 {
            advice
                .push_str("With CO2 this high, are we hosting a dinosaur summit? Open a window!\n");
        } else if co2 < 800 {
            advice.push_str("CO2 levels like a sci-fi space drama—all clear for now!\n");
        }
    }

    // Temperature extremes
    if climate_data.temperature > 25.0 {
        advice.push_str("It's hotter than a tech startup in here. Crack a window!\n");
    } else if climate_data.temperature < 20.0 {
        advice.push_str("It's colder than my ex's heart. Seal those leaks!\n");
    }

    // Humidity adventures
    if climate_data.humidity > 70.0 {
        advice.push_str("Humidity's higher than a hippie at a concert. Time for some fresh air!\n");
    } else if climate_data.humidity < 30.0 {
        advice.push_str("Drier than a British comedy—might wanna close that window.\n");
    }

    // Air quality based on eco2 and etvoc
    if climate_data.eco2 > 1000 {
        advice.push_str("eCO2 through the roof—did someone say 'volcano'? Open up!\n");
    }
    if climate_data.etvoc > 500 {
        advice.push_str("eTVOCs creeping up; it's witchcraft or just bad air—air it out!\n");
    }

    // Pressure decisions
    if climate_data.pressure < 1000.0 {
        advice.push_str("Pressure's dropping—storm's coming or just bad news brewing?\n");
    } else if climate_data.pressure > 1020.0 {
        advice.push_str("High pressure like exam day. Keep calm and don't open the windows.\n");
    }

    // Lighting and visibility
    if let Some(light) = climate_data.light {
        if light < 100.0 {
            advice.push_str("It's gloomier than a horror movie set. Maybe turn on a lamp?\n");
        } else if light > 800.0 {
            advice.push_str("Bright as a supernova—blinds down or get those shades!\n");
        }
    }

    advice
}

pub fn render_dumb_advice_block(climate_data: &ClimateData, area: &Rect, frame: &mut Frame) {
    let paragraph = Paragraph::new(Text::from(create_dumb_climate_advice(climate_data)))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Advice you didn't ask for ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
        );

    frame.render_widget(paragraph, *area);
}
