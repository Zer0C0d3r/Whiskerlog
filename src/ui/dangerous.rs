use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
        .split(area);

    // Header with risk summary
    draw_risk_summary(f, app, chunks[0]);

    // Dangerous commands list
    draw_dangerous_commands(f, app, chunks[1]);
}

fn draw_risk_summary(f: &mut Frame, app: &App, area: Rect) {
    let dangerous_count = app.stats.dangerous_commands;
    let total_count = app.stats.total_commands;
    let risk_percentage = if total_count > 0 {
        (dangerous_count as f32 / total_count as f32) * 100.0
    } else {
        0.0
    };

    let risk_level = match risk_percentage {
        x if x > 30.0 => ("🔴 HIGH RISK", Color::Red),
        x if x > 10.0 => ("🟡 MEDIUM RISK", Color::Yellow),
        x if x > 0.0 => ("🟢 LOW RISK", Color::Green),
        _ => ("✅ NO RISK", Color::Green),
    };

    let summary_text = vec![
        Line::from(vec![
            Span::styled("Risk Assessment: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                risk_level.0,
                Style::default()
                    .fg(risk_level.1)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Dangerous Commands: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!(
                    "{} / {} ({:.1}%)",
                    dangerous_count, total_count, risk_percentage
                ),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Tip: ", Style::default().fg(Color::Yellow)),
            Span::raw("Review commands below and consider safer alternatives"),
        ]),
    ];

    let summary = Paragraph::new(summary_text)
        .block(
            Block::default()
                .title("Security Overview")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(summary, area);
}

fn draw_dangerous_commands(f: &mut Frame, app: &App, area: Rect) {
    let dangerous_commands: Vec<_> = app.commands.iter().filter(|cmd| cmd.is_dangerous).collect();

    let command_items: Vec<ListItem> = dangerous_commands
        .iter()
        .skip(app.scroll_offset)
        .take(area.height as usize - 2)
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = app.scroll_offset + i == app.selected_index;

            let style = if is_selected {
                Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Red)
            };

            let time_str = cmd.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            let risk_score = format!("{:.1}", cmd.danger_score);

            let host_indicator = if cmd.host_id != "local" {
                format!(" [{}]", cmd.host_id)
            } else {
                String::new()
            };

            let reasons = if !cmd.danger_reasons.is_empty() {
                format!(" ({})", cmd.danger_reasons.join(", "))
            } else {
                String::new()
            };

            ListItem::new(Line::from(vec![
                Span::styled("⚠️ ", Style::default().fg(Color::Red)),
                Span::styled(time_str, Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled(
                    format!("[{}] ", risk_score),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(cmd.command.clone(), style),
                Span::styled(host_indicator, Style::default().fg(Color::Blue)),
                Span::styled(reasons, Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let commands_list = List::new(command_items)
        .block(
            Block::default()
                .title(format!(
                    "Dangerous Commands ({} found)",
                    dangerous_commands.len()
                ))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(commands_list, area);
}
