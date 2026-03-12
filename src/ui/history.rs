use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn render_history_list(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let block = Block::default()
        .title(" Session History ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Reserve 3 rows for the date-jump input when active, otherwise just the hint bar.
    let constraints: Vec<Constraint> = if app.history_filtering {
        vec![
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ]
    } else {
        vec![Constraint::Fill(1), Constraint::Length(1)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let (table_area, filter_area, hint_area) = if app.history_filtering {
        (chunks[0], Some(chunks[1]), chunks[2])
    } else {
        (chunks[0], None, chunks[1])
    };

    // ---- Table or empty-state message ----
    if app.history_sessions.is_empty() {
        frame.render_widget(
            Paragraph::new("No workouts recorded yet.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray)),
            table_area,
        );
    } else {
        let rows: Vec<Row> = app
            .history_sessions
            .iter()
            .map(|s| {
                let label = if s.exercise_count == 1 {
                    "1 exercise".to_string()
                } else {
                    format!("{} exercises", s.exercise_count)
                };
                Row::new(vec![s.date.clone(), label])
            })
            .collect();

        let table = Table::new(rows, [Constraint::Length(12), Constraint::Fill(1)])
            .header(
                Row::new(vec!["Date", "Exercises"])
                    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let mut state = TableState::default().with_selected(Some(app.history_selected));
        frame.render_stateful_widget(table, table_area, &mut state);
    }

    // ---- Date-jump input (only when active) ----
    if let Some(area) = filter_area {
        let filter_block = Block::default()
            .title("Jump to date  (YYYY-MM-DD)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        frame.render_widget(
            Paragraph::new(app.history_filter_input.as_str())
                .block(filter_block)
                .style(Style::default().fg(Color::White)),
            area,
        );
        #[allow(clippy::cast_possible_truncation)]
        frame.set_cursor_position((
            area.x + 1 + app.history_filter_input.len() as u16,
            area.y + 1,
        ));
    }

    // ---- Hint / status bar ----
    let (hint_text, hint_style) = if let Some(msg) = &app.status_msg {
        (msg.as_str(), Style::default().fg(Color::Red))
    } else if app.history_filtering {
        (
            "[Enter] jump  [Esc] cancel",
            Style::default().fg(Color::DarkGray),
        )
    } else {
        (
            "[↑↓] navigate  [Enter] view  [f] jump to date  [Esc] back",
            Style::default().fg(Color::DarkGray),
        )
    };
    frame.render_widget(
        Paragraph::new(hint_text).style(hint_style),
        hint_area,
    );
}

pub fn render_history_detail(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Title shows the session date and total exercise count.
    let title = app
        .history_sessions
        .get(app.history_selected)
        .map_or_else(
            || " Session Detail ".to_string(),
            |s| {
                let n = app.history_detail.len();
                let label = if n == 1 { "exercise" } else { "exercises" };
                format!(" {} — {n} {label} ", s.date)
            },
        );

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

    // ---- Content lines ----
    let mut lines: Vec<Line> = Vec::new();
    for entry in &app.history_detail {
        lines.push(Line::from(Span::styled(
            format!("  {}  ({})", entry.exercise_name, entry.category_name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        for set in &entry.sets {
            lines.push(Line::from(Span::styled(
                format!(
                    "    Set {} — {} reps × {}",
                    set.set_number, set.reps, set.weight
                ),
                Style::default().fg(Color::White),
            )));
        }
        lines.push(Line::from(""));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No exercises recorded for this session.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    #[allow(clippy::cast_possible_truncation)]
    let scroll_offset = app.history_detail_scroll.min(usize::from(u16::MAX)) as u16;
    frame.render_widget(
        Paragraph::new(Text::from(lines)).scroll((scroll_offset, 0)),
        chunks[0],
    );

    // ---- Hint bar ----
    frame.render_widget(
        Paragraph::new("[↑↓] scroll  [Esc] back")
            .style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );
}
