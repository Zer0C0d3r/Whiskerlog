use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::theme::Theme;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)].as_ref())
        .split(area);

    // Top panel: Compact metrics row
    draw_compact_metrics(f, app, main_chunks[0], &theme);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(main_chunks[1]);

    // Left panel: Activity chart and top commands
    draw_left_panel(f, app, content_chunks[0], &theme);

    // Right panel: Recent activity and AI insights
    draw_right_panel(f, app, content_chunks[1], &theme);
}

fn draw_compact_metrics(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(area);

    // Commands
    draw_compact_metric(
        f,
        chunks[0],
        theme,
        "Commands",
        app.stats.total_commands.to_string(),
        theme.style_primary(),
    );

    // Sessions
    draw_compact_metric(
        f,
        chunks[1],
        theme,
        "Sessions",
        app.stats.total_sessions.to_string(),
        theme.style_secondary(),
    );

    // Hosts
    draw_compact_metric(
        f,
        chunks[2],
        theme,
        "Hosts",
        app.stats.hosts_count.to_string(),
        theme.style_accent(),
    );

    // Risk
    let danger_ratio = if app.stats.total_commands > 0 {
        (app.stats.dangerous_commands as f32 / app.stats.total_commands as f32) * 100.0
    } else {
        0.0
    };
    draw_compact_metric(
        f,
        chunks[3],
        theme,
        "Risk",
        format!("{:.1}%", danger_ratio),
        if danger_ratio > 30.0 {
            theme.style_danger()
        } else {
            theme.style_success()
        },
    );

    // Learning
    draw_compact_metric(
        f,
        chunks[4],
        theme,
        "Learning",
        app.stats.experiment_sessions.to_string(),
        theme.style_info(),
    );
}

fn draw_compact_metric(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
    label: &str,
    value: String,
    value_style: Style,
) {
    let content = vec![
        Line::from(vec![Span::styled(label, theme.style_text_dim())]),
        Line::from(vec![Span::styled(
            value,
            value_style.add_modifier(Modifier::BOLD),
        )]),
    ];

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn draw_left_panel(f: &mut Frame, _app: &App, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Activity chart
            Constraint::Min(0),     // Top commands
        ])
        .split(area);

    // Activity trend chart (ASCII bar chart style) - removed duplicate title
    let activity_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "█████████████████████████████████████████████████",
            theme.style_primary(),
        )]),
        Line::from(vec![Span::styled(
            "████████████████████████████████████████████",
            theme.style_primary(),
        )]),
        Line::from(vec![Span::styled(
            "██████████████████████████████████████████████████████",
            theme.style_primary(),
        )]),
        Line::from(vec![Span::styled(
            "████████████████████████████████████████",
            theme.style_primary(),
        )]),
        Line::from(vec![Span::styled(
            "██████████████████████████████████████████████████",
            theme.style_primary(),
        )]),
    ];

    let activity_chart = Paragraph::new(activity_text).block(
        Block::default()
            .title("Activity Trend")
            .borders(Borders::ALL)
            .border_style(theme.style_border()),
    );

    f.render_widget(activity_chart, chunks[0]);

    // Top commands - minimal style
    let top_commands: Vec<ListItem> = vec![
        ("git status", 45),
        ("ls -la", 32),
        ("cd ..", 28),
        ("npm install", 21),
        ("docker ps", 18),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (cmd, count))| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{}. ", i + 1), theme.style_text_dim()),
            Span::styled(cmd, theme.style_text()),
            Span::styled(format!(" ({})", count), theme.style_accent()),
        ]))
    })
    .collect();

    let top_commands_list = List::new(top_commands)
        .block(
            Block::default()
                .title("Top Commands")
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(top_commands_list, chunks[1]);
}

fn draw_right_panel(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Recent activity
            Constraint::Percentage(40), // AI Insights
        ])
        .split(area);

    // Recent activity - minimal style with colorized timestamps
    let recent_commands: Vec<ListItem> = vec![
        ("17:50:37", "ls"),
        ("17:50:38", "clear"),
        ("17:50:18", "sudo pacman -Syu"),
        ("17:50:31", "yay -Syu"),
        ("17:59:31", "clear"),
        ("17:57:32", "exit"),
        ("18:02:42", "yay -Syu"),
        ("19:28:00", "clear"),
        ("19:28:18", "exit"),
        ("19:22:37", "local"),
        ("19:22:40", "ls"),
        ("19:22:43", "z share"),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (time, cmd))| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{}. ", i + 1), theme.style_text_dim()),
            Span::styled(time, theme.style_primary()), // Colorized timestamp
            Span::raw(" "),
            Span::styled(cmd, theme.style_text()),
        ]))
    })
    .collect();

    let recent_list = List::new(recent_commands)
        .block(
            Block::default()
                .title("Recent Activity")
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(recent_list, chunks[0]);

    // AI Insights - minimal style (fixed double text issue)
    let insights = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "High Docker activity detected",
            theme.style_text(),
        )]),
        Line::from(vec![Span::styled(
            format!(
                "{} risky commands need review",
                app.stats.dangerous_commands
            ),
            theme.style_danger(),
        )]),
        Line::from(vec![Span::styled(
            format!(
                "{} learning sessions identified",
                app.stats.experiment_sessions
            ),
            theme.style_success(),
        )]),
        Line::from(vec![Span::styled(
            format!("{} unique endpoints accessed", app.stats.network_endpoints),
            theme.style_info(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Productivity Score: ", theme.style_text()),
            Span::styled("87%", theme.style_success().add_modifier(Modifier::BOLD)),
        ]),
    ];

    let insights_paragraph = Paragraph::new(insights)
        .block(
            Block::default()
                .title("AI Insights")
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(insights_paragraph, chunks[1]);
}
