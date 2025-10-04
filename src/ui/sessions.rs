use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;

use crate::app::App;
use crate::ui::theme::{Icons, Theme};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    // Left panel: Sessions list
    draw_sessions_list(f, app, chunks[0], &theme);

    // Right panel: Session details and timeline
    draw_session_details(f, app, chunks[1], &theme);
}

fn draw_sessions_list(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    // Group commands by session
    let mut sessions: HashMap<String, Vec<&crate::history::Command>> = HashMap::new();
    for cmd in &app.commands {
        sessions
            .entry(cmd.session_id.clone())
            .or_default()
            .push(cmd);
    }

    let mut session_data: Vec<_> = sessions
        .into_iter()
        .map(|(session_id, commands)| {
            let start_time = commands.iter().map(|c| c.timestamp).min().unwrap();
            let end_time = commands.iter().map(|c| c.timestamp).max().unwrap();
            let duration = (end_time - start_time).num_minutes();
            let command_count = commands.len();
            let dangerous_count = commands.iter().filter(|c| c.is_dangerous).count();
            let experiment_count = commands.iter().filter(|c| c.is_experiment).count();
            let hosts: std::collections::HashSet<_> = commands.iter().map(|c| &c.host_id).collect();

            (
                session_id,
                start_time,
                duration,
                command_count,
                dangerous_count,
                experiment_count,
                hosts.len(),
            )
        })
        .collect();

    // Sort by start time (most recent first)
    session_data.sort_by(|a, b| b.1.cmp(&a.1));

    let session_count = session_data.len();
    let session_items: Vec<ListItem> = session_data
        .into_iter()
        .enumerate()
        .take(area.height as usize - 2)
        .map(
            |(i, (session_id, start_time, duration, cmd_count, dangerous, experiments, hosts))| {
                let is_selected = i == app.selected_index;

                let style = if is_selected {
                    theme.style_selected()
                } else {
                    theme.style_text()
                };

                let session_type = if session_id.starts_with("bash") {
                    ("", theme.style_accent())
                } else if session_id.starts_with("zsh") {
                    ("", theme.style_secondary())
                } else if session_id.starts_with("fish") {
                    ("", theme.style_primary())
                } else {
                    (Icons::TERMINAL, theme.style_text())
                };

                let time_str = start_time.format("%m-%d %H:%M").to_string();
                let duration_str = if duration > 60 {
                    format!("{}h{}m", duration / 60, duration % 60)
                } else {
                    format!("{}m", duration)
                };

                let activity_level = (cmd_count as f32 / 50.0).min(1.0);
                let activity_icon = crate::ui::theme::get_activity_icon(activity_level);

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:2}. ", i + 1), theme.style_text_dim()),
                    Span::styled(format!("{} ", session_type.0), session_type.1),
                    Span::styled(time_str, theme.style_text_dim()),
                    Span::raw(" "),
                    Span::styled(duration_str, style),
                    Span::raw(" "),
                    Span::styled(format!("{}cmd", cmd_count), theme.style_info()),
                    if dangerous > 0 {
                        Span::styled(format!(" {}⚠", dangerous), theme.style_danger())
                    } else {
                        Span::raw("")
                    },
                    if experiments > 0 {
                        Span::styled(format!(" {}🔬", experiments), theme.style_warning())
                    } else {
                        Span::raw("")
                    },
                    if hosts > 1 {
                        Span::styled(format!(" {}🌐", hosts), theme.style_secondary())
                    } else {
                        Span::raw("")
                    },
                    Span::styled(format!(" {}", activity_icon), theme.style_accent()),
                ]))
            },
        )
        .collect();

    let sessions_list = List::new(session_items)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::SESSIONS), theme.style_accent()),
                    Span::styled("Terminal Sessions", theme.style_title()),
                    Span::styled(format!(" ({})", session_count), theme.style_text_dim()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(sessions_list, area);
}

fn draw_session_details(f: &mut Frame, _app: &App, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Session info
            Constraint::Length(6), // Activity gauge
            Constraint::Min(0),    // Commands timeline
        ])
        .split(area);

    // Session info (mock data for selected session)
    let session_info = vec![
        Line::from(vec![
            Span::styled(format!("{} ", Icons::CALENDAR), theme.style_accent()),
            Span::styled("Session Details", theme.style_title()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Started: ", theme.style_text_dim()),
            Span::styled("2025-07-14 20:43", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("Duration: ", theme.style_text_dim()),
            Span::styled("2h 15m", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("Shell: ", theme.style_text_dim()),
            Span::styled(" zsh", theme.style_secondary()),
        ]),
        Line::from(vec![
            Span::styled("Commands: ", theme.style_text_dim()),
            Span::styled("89", theme.style_text()),
            Span::styled(" (", theme.style_text_dim()),
            Span::styled("3 failed", theme.style_danger()),
            Span::styled(")", theme.style_text_dim()),
        ]),
    ];

    let info_paragraph = Paragraph::new(session_info)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(info_paragraph, chunks[0]);

    // Activity intensity gauge
    let activity_ratio = 0.73; // Mock data
    let activity_gauge = Gauge::default()
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::LIGHTNING), theme.style_accent()),
                    Span::styled("Activity Level", theme.style_title()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .gauge_style(if activity_ratio > 0.8 {
            theme.style_danger()
        } else if activity_ratio > 0.5 {
            theme.style_warning()
        } else {
            theme.style_success()
        })
        .ratio(activity_ratio)
        .label(format!("{:.0}% - High Activity", activity_ratio * 100.0));

    f.render_widget(activity_gauge, chunks[1]);

    // Commands timeline (recent commands from this session)
    let timeline_commands: Vec<ListItem> = vec![
        ("20:43", "cd ~/projects/whiskerlog", false, false),
        ("20:44", "git status", false, false),
        ("20:45", "cargo build", false, false),
        ("20:47", "cargo run", true, false),
        ("20:48", "vim src/main.rs", false, false),
        ("20:52", "cargo check", false, false),
        ("20:53", "git add .", false, false),
        ("20:54", "git commit -m 'fix ui'", false, false),
        ("20:55", "rm -rf target/", true, false),
        ("20:56", "man cargo", false, true),
        ("20:57", "cargo --help", false, true),
        ("20:58", "cargo build --release", false, false),
    ]
    .into_iter()
    .map(|(time, cmd, is_dangerous, is_experiment)| {
        let style = if is_dangerous {
            theme.style_danger()
        } else if is_experiment {
            theme.style_warning()
        } else {
            theme.style_text()
        };

        let status_icon = if is_dangerous {
            Icons::DANGEROUS
        } else if is_experiment {
            Icons::EXPERIMENTS
        } else {
            Icons::SUCCESS
        };

        ListItem::new(Line::from(vec![
            Span::styled(
                format!("{} ", status_icon),
                if is_dangerous {
                    theme.style_danger()
                } else if is_experiment {
                    theme.style_warning()
                } else {
                    theme.style_success()
                },
            ),
            Span::styled(time, theme.style_text_dim()),
            Span::raw(" "),
            Span::styled(cmd, style),
        ]))
    })
    .collect();

    let timeline_list = List::new(timeline_commands)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::TIME), theme.style_accent()),
                    Span::styled("Session Timeline", theme.style_title()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(timeline_list, chunks[2]);
}
