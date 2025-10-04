use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, Tab};

pub mod theme;
pub use theme::{Icons, Theme};

pub mod aliases;
pub mod commands;
pub mod dangerous;
pub mod experiments;
pub mod heatmap;
pub mod hosts;
pub mod network;
pub mod packages;
pub mod search;
pub mod sessions;
pub mod summary;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = Theme::default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.area());

    // Draw tabs with enhanced styling
    draw_tabs(f, app, chunks[0], &theme);

    // Draw main content based on current tab
    match app.current_tab {
        Tab::Summary => summary::draw(f, app, chunks[1]),
        Tab::Commands => commands::draw(f, app, chunks[1]),
        Tab::Sessions => sessions::draw(f, app, chunks[1]),
        Tab::Search => search::draw(f, app, chunks[1]),
        Tab::Hosts => hosts::draw(f, app, chunks[1]),
        Tab::Heatmap => heatmap::draw(f, app, chunks[1]),
        Tab::Aliases => aliases::draw(f, app, chunks[1]),
        Tab::Dangerous => dangerous::draw(f, app, chunks[1]),
        Tab::Network => network::draw(f, app, chunks[1]),
        Tab::Packages => packages::draw(f, app, chunks[1]),
        Tab::Experiments => experiments::draw(f, app, chunks[1]),
    }

    // Draw bottom navigation bar
    draw_bottom_nav(f, app, chunks[2], &theme);

    // Draw help overlay if visible
    if app.help_visible {
        draw_help_overlay(f, &theme);
    }

    // Search overlay removed - search is now integrated into the Search tab
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| {
            let icon = match t {
                Tab::Summary => "",
                Tab::Commands => "",
                Tab::Sessions => "",
                Tab::Search => "",
                Tab::Hosts => "",
                Tab::Heatmap => "",
                Tab::Aliases => "",
                Tab::Dangerous => "",
                Tab::Network => "",
                Tab::Packages => "",
                Tab::Experiments => "",
            };
            Line::from(vec![
                Span::styled(format!("{} ", icon), theme.style_accent()),
                Span::styled(t.title(), theme.style_text()),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.style_border())
                .title(Line::from(vec![
                    Span::styled("🐱 ", theme.style_primary()),
                    Span::styled("Whiskerlog", theme.style_primary()),
                    Span::styled(" - Terminal Intelligence", theme.style_text_dim()),
                ])),
        )
        .style(theme.style_text())
        .highlight_style(theme.style_primary())
        .select(app.tab_index);

    f.render_widget(tabs, area);
}

fn draw_help_overlay(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(70, 80, f.area());

    let help_text = vec![
        Line::from(vec![
            Span::styled(format!("{} ", Icons::WHISKER), theme.style_primary()),
            Span::styled("Whiskerlog", theme.style_title()),
            Span::styled(" - Terminal History Intelligence", theme.style_text()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", Icons::GEAR), theme.style_accent()),
            Span::styled("Global Shortcuts:", theme.style_highlight()),
        ]),
        Line::from(vec![
            Span::styled("  q", theme.style_primary()),
            Span::styled("       - Quit application", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("  /", theme.style_primary()),
            Span::styled("       - Go to Search tab", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("  ?", theme.style_primary()),
            Span::styled("       - Toggle this help", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("  Tab", theme.style_primary()),
            Span::styled("     - Next tab", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("  S-Tab", theme.style_primary()),
            Span::styled("   - Previous tab", theme.style_text()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", Icons::ARROW_UP), theme.style_accent()),
            Span::styled("Navigation:", theme.style_highlight()),
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓", theme.style_primary()),
            Span::styled("     - Scroll up/down", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("  ←/→", theme.style_primary()),
            Span::styled("     - Scroll left/right", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("  Enter", theme.style_primary()),
            Span::styled("   - Select/Action", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("  Esc", theme.style_primary()),
            Span::styled("     - Cancel/Back", theme.style_text()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", Icons::INFO), theme.style_info()),
            Span::styled(
                "Tab-specific shortcuts shown in each tab",
                theme.style_text_dim(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", theme.style_text_dim()),
            Span::styled("?", theme.style_primary()),
            Span::styled(" or ", theme.style_text_dim()),
            Span::styled("Esc", theme.style_primary()),
            Span::styled(" to close this help", theme.style_text_dim()),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::QUESTION), theme.style_info()),
                    Span::styled("Help", theme.style_title()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text())
        .alignment(Alignment::Left);

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

// Search overlay function removed - search is now integrated into the Search tab

fn draw_bottom_nav(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let nav_text = vec![Line::from(vec![
        Span::styled(
            format!("{} commands", app.stats.total_commands),
            theme.style_text(),
        ),
        Span::styled(" | ", theme.style_text_dim()),
        Span::styled(
            format!("{} hosts", app.stats.hosts_count),
            theme.style_text(),
        ),
        Span::styled(" | ", theme.style_text_dim()),
        Span::styled(
            format!("{} risky", app.stats.dangerous_commands),
            theme.style_danger(),
        ),
        Span::styled(" | ", theme.style_text_dim()),
        Span::styled(
            format!("{} experiments", app.stats.experiment_sessions),
            theme.style_accent(),
        ),
        Span::styled("     ", theme.style_text_dim()),
        Span::styled("[Tab]", theme.style_primary()),
        Span::styled(" Switch", theme.style_text_dim()),
        Span::styled(" [↑↓hjkl]", theme.style_primary()),
        Span::styled(" Navigate", theme.style_text_dim()),
        Span::styled(" [Enter]", theme.style_primary()),
        Span::styled(" Select", theme.style_text_dim()),
        Span::styled(" [1-9]", theme.style_primary()),
        Span::styled(" Jump", theme.style_text_dim()),
        Span::styled(" [/]", theme.style_primary()),
        Span::styled(" Go to Search", theme.style_text_dim()),
        Span::styled(" [?]", theme.style_primary()),
        Span::styled(" Help", theme.style_text_dim()),
        Span::styled(" [q]", theme.style_danger()),
        Span::styled(" Quit", theme.style_text_dim()),
    ])];

    let paragraph = Paragraph::new(nav_text)
        .style(theme.style_text())
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
