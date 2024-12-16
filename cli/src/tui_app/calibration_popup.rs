use super::Action;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, Default, EnumIter, Display)]
enum Tab {
    #[default]
    #[strum(to_string = "CO2 Calibration")]
    Co2,
    #[strum(to_string = "Temperature Calibration")]
    Temperature { input: String },
}

impl Tab {
    fn get_char(&self) -> char {
        match self {
            Tab::Co2 => 'c',
            Tab::Temperature { .. } => 't',
        }
    }

    fn title(self) -> Line<'static> {
        Line::from(vec![
            Span::from(" "),
            Span::styled(
                format!("[{}]", self.get_char()),
                Style::default().fg(Color::Blue),
            ),
            Span::from(" "),
            Span::styled(format!("{self}"), Style::default().fg(Color::White)),
            Span::from(" "),
        ])
    }
}

#[derive(Debug, Default)]
pub struct CalibrationPopup {
    tab: Tab,
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

impl CalibrationPopup {
    pub fn co2() -> Self {
        Self { tab: Tab::Co2 }
    }
    pub fn temperature() -> Self {
        Self {
            tab: Tab::Temperature {
                input: "".to_string(),
            },
        }
    }

    fn render_co2_tab(&self, area: Rect, f: &mut Frame) {
        let text = Paragraph::new(vec![
          Line::from(""),
          Line::from("Calibration of CO2 is performed by a controlled measurement which will be used as a 400ppm point. For better results put the sensor outside or in a well ventilated room."),
        ]).wrap(Wrap { trim: true });

        f.render_widget(text, area);
    }

    fn render_temperature_tab(&self, area: Rect, f: &mut Frame) {
        let text = Paragraph::new(vec![
            Line::from(""),
            Line::from(
                "To calibrate temperature please enter the desired temperature in Celsius. It will be used as an ideal temperature point and all the values will be scaled accordingly.",
            ),
        ]).wrap(Wrap { trim: true });

        f.render_widget(text, area);
    }

    fn render_selected_tab(&self, area: Rect, f: &mut Frame) {
        match self.tab {
            Tab::Co2 => self.render_co2_tab(area, f),
            Tab::Temperature { .. } => self.render_temperature_tab(area, f),
        }
    }

    fn render_tabs(&self, area: Rect, f: &mut Frame) {
        let titles = Tab::iter().map(Tab::title);
        let selected_tab_index = match self.tab {
            Tab::Co2 => 0,
            Tab::Temperature { .. } => 1,
        };

        let tabs = ratatui::widgets::Tabs::new(titles)
            .highlight_style(
                Style::default()
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ");

        f.render_widget(tabs, area)
    }

    fn render_control(&self, area: Rect, f: &mut Frame) {
        let area = centered_rect(40, 100, area);
        let text = Paragraph::new(vec![Line::from(" Press [Enter] to calibrate ")])
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .style(
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(text, area);
    }

    pub fn render(&self, f: &mut Frame) {
        let popup_block = Block::default()
            .title("Enter a new key-value pair")
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Black))
            .style(Style::default().bg(Color::DarkGray));

        let area = centered_rect(60, 25, f.area());
        f.render_widget(Clear, area);
        f.render_widget(popup_block, area);

        let [tabs_area, body_area, control_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(area);
        self.render_tabs(tabs_area, f);
        self.render_control(control_area, f);

        let [_, body_area, _] = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(body_area);
        self.render_selected_tab(body_area, f);
    }

    pub fn handle_key(&self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('t') => Some(Action::OpenCalibrateTemperaturePopup),
            KeyCode::Char('c') => Some(Action::OpenCalibrateCo2Popup),
            KeyCode::Esc => Some(Action::OpenDashboard),
            KeyCode::Enter => match self.tab {
                Tab::Co2 => Some(Action::CalibrateCo2),
                Tab::Temperature { .. } => todo!(),
            },
            _ => None,
        }
    }
}
