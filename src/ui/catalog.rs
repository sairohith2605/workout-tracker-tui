use crate::app::{AddField, App};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn render_catalog_list(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let filter_label = filter_label(app.catalog_filter_idx, &app.categories);
    let title = format!(" Catalog  [filter: {filter_label}] ");

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

    let filtered = app.filtered_catalog();

    if filtered.is_empty() {
        let msg = if app.catalog.is_empty() {
            "No exercises yet — press [a] to add one"
        } else {
            "No exercises in this category — press [f] to change filter"
        };
        let p = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        let y_mid = chunks[0].y + chunks[0].height / 2;
        frame.render_widget(
            p,
            Rect {
                x: chunks[0].x,
                y: y_mid,
                width: chunks[0].width,
                height: 1,
            },
        );
    } else {
        let rows: Vec<Row> = filtered
            .iter()
            .map(|ex| Row::new(vec![ex.name.clone(), ex.category_name.clone()]))
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Fill(1), Constraint::Length(22)],
        )
        .header(
            Row::new(vec!["Exercise", "Category"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

        let mut state = TableState::default().with_selected(Some(app.catalog_selected));
        frame.render_stateful_widget(table, chunks[0], &mut state);
    }

    // Status / hint bar
    let bar_text;
    let bar_style;
    if let Some(msg) = &app.status_msg {
        bar_text = msg.as_str();
        bar_style = if app.delete_confirm {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
    } else {
        bar_text = "[↑↓] navigate  [a] add  [d] delete  [f] filter  [Enter] continue  [q] quit";
        bar_style = Style::default().fg(Color::DarkGray);
    }
    frame.render_widget(
        Paragraph::new(bar_text).style(bar_style),
        chunks[1],
    );
}

pub fn render_add_exercise(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let block = Block::default()
        .title(" Add Exercise ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name input
            Constraint::Fill(1),   // Category list
            Constraint::Length(1), // Status / hint
        ])
        .split(inner);

    // --- Name input ---
    let name_border_style = if app.add_focus == AddField::Name {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    let name_block = Block::default()
        .title("Exercise Name")
        .borders(Borders::ALL)
        .border_style(name_border_style);
    let name_para = Paragraph::new(app.add_name.as_str())
        .block(name_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(name_para, chunks[0]);

    // Cursor inside the name field when focused
    if app.add_focus == AddField::Name {
        #[allow(clippy::cast_possible_truncation)]
        frame.set_cursor_position((
            chunks[0].x + 1 + app.add_name.len() as u16,
            chunks[0].y + 1,
        ));
    }

    // --- Category picker ---
    let cat_border_style = if app.add_focus == AddField::Category {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    let cat_block = Block::default()
        .title("Category  [↑↓ to select]")
        .borders(Borders::ALL)
        .border_style(cat_border_style);

    let items: Vec<ListItem> = app
        .categories
        .iter()
        .map(|c| ListItem::new(c.name.clone()))
        .collect();

    let list = List::new(items)
        .block(cat_block)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default().with_selected(Some(app.add_category_idx));
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    // --- Status / hint ---
    let bar_text;
    let bar_style;
    if let Some(msg) = &app.status_msg {
        bar_text = msg.as_str();
        bar_style = Style::default().fg(Color::Red);
    } else {
        bar_text = "[Tab] switch field  [Enter] save  [Esc] cancel";
        bar_style = Style::default().fg(Color::DarkGray);
    }
    frame.render_widget(Paragraph::new(bar_text).style(bar_style), chunks[2]);
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn filter_label(idx: Option<usize>, categories: &[crate::models::Category]) -> String {
    match idx {
        None => "All".to_string(),
        Some(i) => categories
            .get(i)
            .map_or_else(|| "All".to_string(), |c| c.name.clone()),
    }
}
