use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)].as_ref())
        .split(area);

    // Top panel: Experiment summary
    draw_experiment_summary(f, app, chunks[0]);

    // Bottom panel: Experimental commands
    draw_experimental_commands(f, app, chunks[1]);
}

fn draw_experiment_summary(f: &mut Frame, app: &App, area: Rect) {
    let experiment_count = app.commands.iter().filter(|cmd| cmd.is_experiment).count();
    let total_count = app.commands.len();
    let experiment_percentage = if total_count > 0 {
        (experiment_count as f32 / total_count as f32) * 100.0
    } else {
        0.0
    };

    // Collect experiment tags
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    for cmd in &app.commands {
        if cmd.is_experiment {
            for tag in &cmd.experiment_tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
    }

    let top_tags: Vec<_> = {
        let mut tags: Vec<_> = tag_counts.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        tags.into_iter().take(3).collect()
    };

    let summary_text = vec![
        Line::from(vec![
            Span::styled("🔬 Learning Mode: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{} experimental commands ({:.1}%)", experiment_count, experiment_percentage), 
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Top Activities: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                top_tags.iter()
                    .map(|(tag, count)| format!("{} ({})", tag, count))
                    .collect::<Vec<_>>()
                    .join(", "),
                Style::default().fg(Color::White)
            ),
        ]),
        Line::from(vec![
            Span::styled("Tip: ", Style::default().fg(Color::Green)),
            Span::raw("Experimental sessions are auto-detected based on help usage and exploration patterns"),
        ]),
    ];

    let summary = Paragraph::new(summary_text)
        .block(
            Block::default()
                .title("Experiment Overview")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(summary, area);
}

fn draw_experimental_commands(f: &mut Frame, app: &App, area: Rect) {
    let experimental_commands: Vec<_> = app
        .commands
        .iter()
        .filter(|cmd| cmd.is_experiment)
        .collect();

    let command_items: Vec<ListItem> = experimental_commands
        .iter()
        .skip(app.scroll_offset)
        .take(area.height as usize - 2)
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = app.scroll_offset + i == app.selected_index;

            let style = if is_selected {
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow)
            };

            let time_str = cmd.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

            let tags_str = if !cmd.experiment_tags.is_empty() {
                format!(" [{}]", cmd.experiment_tags.join(", "))
            } else {
                String::new()
            };

            let host_indicator = if cmd.host_id != "local" {
                format!(" @{}", cmd.host_id)
            } else {
                String::new()
            };

            ListItem::new(Line::from(vec![
                Span::styled("🔬 ", Style::default().fg(Color::Yellow)),
                Span::styled(time_str, Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled(cmd.command.clone(), style),
                Span::styled(tags_str, Style::default().fg(Color::Cyan)),
                Span::styled(host_indicator, Style::default().fg(Color::Blue)),
            ]))
        })
        .collect();

    let commands_list = List::new(command_items)
        .block(
            Block::default()
                .title(format!(
                    "Experimental Commands ({} found)",
                    experimental_commands.len()
                ))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(commands_list, area);
}
