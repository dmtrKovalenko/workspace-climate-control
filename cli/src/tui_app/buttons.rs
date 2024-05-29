use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use super::Action;

struct Button {
    label: &'static str,
    control: Action,
    char: char,
}

const BUTTONS: [Button; 4] = [
    Button {
        label: "Calibrate",
        control: Action::OpenCalibrateCo2Popup,
        char: 'c',
    },
    Button {
        label: "Reconnect",
        control: Action::Reconnect,
        char: 'r',
    },
    Button {
        label: "Clear history",
        control: Action::ClearHistory,
        char: 'x',
    },
    Button {
        label: "Exit",
        control: Action::Exit,
        char: 'q',
    },
];

pub fn handle_dashboard_key_event(keycode: KeyCode) -> Option<Action> {
    match keycode {
        KeyCode::Char(char) => BUTTONS.iter().find_map(|button| {
            if button.char == char {
                Some(button.control)
            } else {
                None
            }
        }),
        _ => None,
    }
}

pub fn render_buttons(area: Rect, frame: &mut Frame) {
    let buttons_per_row: u16 = match area.width {
        0..=25 => 1,
        26..=50 => 2,
        51..=75 => 3,
        _ => 4,
    };

    let button_width = area.width / buttons_per_row;
    let button_style = Style::default().fg(Color::Gray);
    let char_style = Style::default().fg(Color::LightBlue);

    for (i, button) in BUTTONS.iter().enumerate() {
        let i = i as u16;

        let x = (i % buttons_per_row) * button_width;
        let y = (i / buttons_per_row) * 3;
        let button_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width: button_width,
            height: 3,
        };

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("[{}] ", button.char), char_style),
                Span::styled(button.label, button_style),
            ]))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(button_style),
            ),
            button_area,
        );
    }
}
