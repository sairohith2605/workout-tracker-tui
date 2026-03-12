use crate::app::{App, SetField};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, TableState},
    Frame,
};

// ---------------------------------------------------------------------------
// WorkoutDate
// ---------------------------------------------------------------------------

pub fn render_date(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let block = Block::default()
        .title(" Log Workout — Select Date ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let date_block = Block::default()
        .title("Date  (YYYY-MM-DD)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    frame.render_widget(
        Paragraph::new(app.session_date.as_str())
            .block(date_block)
            .style(Style::default().fg(Color::White)),
        chunks[1],
    );

    // Cursor inside date input
    #[allow(clippy::cast_possible_truncation)]
    frame.set_cursor_position((
        chunks[1].x + 1 + app.session_date.len() as u16,
        chunks[1].y + 1,
    ));

    frame.render_widget(
        Paragraph::new("[Enter] confirm  [Esc] back")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        chunks[2],
    );

    // Status bar at bottom
    if let Some(msg) = &app.status_msg {
        frame.render_widget(
            Paragraph::new(msg.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center),
            chunks[4],
        );
    }
}

// ---------------------------------------------------------------------------
// ExercisePicker
// ---------------------------------------------------------------------------

pub fn render_picker(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let filter_label = filter_label(app.picker_filter_idx, &app.categories);
    let added = app.session_exercises.len();
    let title = format!(" Pick Exercise  [filter: {filter_label}]  [{added} added] ");

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .split(inner);

    let filtered = app.filtered_picker_catalog();
    let rows: Vec<Row> = filtered
        .iter()
        .map(|ex| {
            let check = if app.is_exercise_in_session(ex) { "✓" } else { " " };
            Row::new(vec![
                check.to_string(),
                ex.name.clone(),
                ex.category_name.clone(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(22),
        ],
    )
    .header(
        Row::new(vec!["", "Exercise", "Category"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    let mut state = TableState::default().with_selected(Some(app.picker_selected));
    frame.render_stateful_widget(table, chunks[0], &mut state);

    let bar_text;
    let bar_style;
    if let Some(msg) = &app.status_msg {
        bar_text = msg.as_str();
        bar_style = Style::default().fg(Color::Red);
    } else {
        bar_text = "[↑↓] navigate  [Enter] select  [f] filter  [d] done  [Esc] back";
        bar_style = Style::default().fg(Color::DarkGray);
    }
    frame.render_widget(Paragraph::new(bar_text).style(bar_style), chunks[1]);
}

// ---------------------------------------------------------------------------
// SetLogger
// ---------------------------------------------------------------------------

pub fn render_set_logger(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let exercise_name = app
        .current_exercise
        .as_ref()
        .map_or("—", |e| e.name.as_str());
    let set_count = app.current_sets.len();
    let title = format!(" Logging: {exercise_name}  [{set_count} set(s)] ");

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // Sets logged
            Constraint::Length(5), // Input row
            Constraint::Length(1), // Hint / status
        ])
        .split(inner);

    // --- Sets logged so far ---
    let set_block = Block::default()
        .title(" Sets recorded ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));

    let set_items: Vec<ListItem> = app
        .current_sets
        .iter()
        .enumerate()
        .map(|(i, s)| {
            ListItem::new(format!(
                "  Set {:>2}: {:>4} reps @ {:>7.2}",
                i + 1,
                s.reps,
                s.weight
            ))
        })
        .collect();

    frame.render_widget(List::new(set_items).block(set_block), chunks[0]);

    // --- Input fields (side by side) ---
    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let reps_style = focused_style(app.set_focus == SetField::Reps);
    let weight_style = focused_style(app.set_focus == SetField::Weight);

    frame.render_widget(
        Paragraph::new(app.set_reps_input.as_str()).block(
            Block::default()
                .title("Reps")
                .borders(Borders::ALL)
                .border_style(reps_style),
        ),
        input_chunks[0],
    );
    frame.render_widget(
        Paragraph::new(app.set_weight_input.as_str()).block(
            Block::default()
                .title("Weight")
                .borders(Borders::ALL)
                .border_style(weight_style),
        ),
        input_chunks[1],
    );

    // Cursor in active input
    if app.set_focus == SetField::Reps {
        #[allow(clippy::cast_possible_truncation)]
        frame.set_cursor_position((
            input_chunks[0].x + 1 + app.set_reps_input.len() as u16,
            input_chunks[0].y + 1,
        ));
    } else {
        #[allow(clippy::cast_possible_truncation)]
        frame.set_cursor_position((
            input_chunks[1].x + 1 + app.set_weight_input.len() as u16,
            input_chunks[1].y + 1,
        ));
    }

    // --- Hint / status ---
    let bar_text;
    let bar_style;
    if let Some(msg) = &app.status_msg {
        bar_text = msg.as_str();
        bar_style = Style::default().fg(Color::Green);
    } else {
        bar_text = "[Tab] switch field  [Enter] record set  [d] done  [Esc] discard";
        bar_style = Style::default().fg(Color::DarkGray);
    }
    frame.render_widget(Paragraph::new(bar_text).style(bar_style), chunks[2]);
}

// ---------------------------------------------------------------------------
// WorkoutSummary
// ---------------------------------------------------------------------------

pub fn render_summary(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let title = format!(" Workout Summary — {} ", app.session_date);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .split(inner);

    let mut items: Vec<ListItem> = Vec::new();
    for se in &app.session_exercises {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("▸ {}", se.exercise.name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  [{}]", se.exercise.category_name),
                Style::default().fg(Color::DarkGray),
            ),
        ])));
        for (i, s) in se.sets.iter().enumerate() {
            items.push(ListItem::new(format!(
                "    Set {:>2}: {:>4} reps @ {:>7.2}",
                i + 1,
                s.reps,
                s.weight
            )));
        }
    }

    frame.render_widget(List::new(items), chunks[0]);

    let bar_text;
    let bar_style;
    if let Some(msg) = &app.status_msg {
        bar_text = msg.as_str();
        bar_style = Style::default().fg(Color::Green);
    } else {
        bar_text = "[s] save workout  [Esc] back to exercise picker";
        bar_style = Style::default().fg(Color::DarkGray);
    }
    frame.render_widget(Paragraph::new(bar_text).style(bar_style), chunks[1]);
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn focused_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    }
}

fn filter_label(idx: Option<usize>, categories: &[crate::models::Category]) -> String {
    match idx {
        None => "All".to_string(),
        Some(i) => categories
            .get(i)
            .map_or_else(|| "All".to_string(), |c| c.name.clone()),
    }
}
