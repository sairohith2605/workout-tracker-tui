pub mod catalog;
pub mod history;
pub mod workout;

use crate::app::{App, Screen};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::CatalogList => catalog::render_catalog_list(frame, app),
        Screen::AddExercise => catalog::render_add_exercise(frame, app),
        Screen::MainMenu => render_main_menu(frame, app),
        Screen::WorkoutDate => workout::render_date(frame, app),
        Screen::ExercisePicker => workout::render_picker(frame, app),
        Screen::SetLogger => workout::render_set_logger(frame, app),
        Screen::WorkoutSummary => workout::render_summary(frame, app),
        Screen::HistoryList => history::render_history_list(frame, app),
        Screen::HistoryDetail => history::render_history_detail(frame, app),
    }
}

fn render_main_menu(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let block = Block::default()
        .title(" Fitness Tracker ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let menu = Paragraph::new("[l]  Log Workout\n[c]  Manage Catalog\n[h]  History\n[q]  Quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));
    frame.render_widget(menu, chunks[1]);

    render_status_bar(frame, app, chunks[3], Color::Green);
}

// ---------------------------------------------------------------------------
// Shared helpers used by submodules
// ---------------------------------------------------------------------------

pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect, color: Color) {
    let (text, style) = if let Some(msg) = &app.status_msg {
        (msg.as_str(), Style::default().fg(color))
    } else {
        ("", Style::default().fg(Color::DarkGray))
    };
    frame.render_widget(Paragraph::new(text).style(style), area);
}
