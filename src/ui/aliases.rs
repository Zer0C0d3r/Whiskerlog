use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::analysis::alias_suggest::{AliasAnalysis, AliasSuggester};
use crate::app::App;
use crate::ui::theme::{Icons, Theme};

// Cache for alias analysis to prevent flickering
static ALIAS_CACHE: OnceLock<Mutex<(AliasAnalysis, Instant)>> = OnceLock::new();

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(5), // Enhanced header with metrics
                Constraint::Min(0),    // Main content
                Constraint::Length(4), // Enhanced footer with controls
            ]
            .as_ref(),
        )
        .split(area);

    // Get cached or fresh analysis
    let analysis = get_cached_analysis(app);

    // Enhanced header with alias metrics
    draw_enhanced_header_cached(f, &analysis, chunks[0], &theme);

    // Main content with comprehensive alias analysis
    draw_enhanced_content_cached(f, &analysis, chunks[1], &theme);

    // Enhanced footer with controls and export options
    draw_enhanced_footer(f, chunks[2], &theme);
}

fn get_cached_analysis(app: &App) -> AliasAnalysis {
    let cache = ALIAS_CACHE.get_or_init(|| {
        let suggester = AliasSuggester::new();
        let analysis = suggester.analyze_alias_opportunities(&app.commands);
        Mutex::new((analysis, Instant::now()))
    });

    let mut cache_guard = cache.lock().unwrap();
    let (cached_analysis, last_update) = &mut *cache_guard;

    // Update cache every 5 seconds to prevent excessive recalculation
    if last_update.elapsed() > Duration::from_secs(5) {
        let suggester = AliasSuggester::new();
        *cached_analysis = suggester.analyze_alias_opportunities(&app.commands);
        *last_update = Instant::now();
    }

    cached_analysis.clone()
}

fn draw_enhanced_header_cached(f: &mut Frame, analysis: &AliasAnalysis, area: Rect, theme: &Theme) {
    let suggester = AliasSuggester::new();
    let efficiency_gain = suggester.calculate_efficiency_gain(analysis);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Total Suggestions
    draw_metric_card(
        f,
        chunks[0],
        theme,
        "Suggestions",
        analysis.suggestions.len().to_string(),
        theme.style_primary(),
    );

    // Potential Savings
    draw_metric_card(
        f,
        chunks[1],
        theme,
        "Char Savings",
        analysis.potential_savings.to_string(),
        theme.style_success(),
    );

    // Efficiency Gain
    draw_metric_card(
        f,
        chunks[2],
        theme,
        "Efficiency",
        format!("{:.1}%", efficiency_gain),
        theme.style_accent(),
    );

    // Existing Aliases
    draw_metric_card(
        f,
        chunks[3],
        theme,
        "In Use",
        analysis.existing_aliases_usage.len().to_string(),
        theme.style_info(),
    );
}

fn draw_metric_card(
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

fn draw_enhanced_content_cached(
    f: &mut Frame,
    analysis: &AliasAnalysis,
    area: Rect,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Alias suggestions
            Constraint::Percentage(40), // Existing aliases and efficiency
        ])
        .split(area);

    // Left panel: Enhanced alias suggestions
    draw_alias_suggestions(f, analysis, chunks[0], theme);

    // Right panel: Existing aliases and efficiency analysis
    let suggester = AliasSuggester::new();
    draw_alias_analysis(f, analysis, &suggester, chunks[1], theme);
}

fn draw_alias_suggestions(
    f: &mut Frame,
    analysis: &crate::analysis::alias_suggest::AliasAnalysis,
    area: Rect,
    theme: &Theme,
) {
    let mut items = Vec::new();

    if analysis.suggestions.is_empty() {
        items.push(ListItem::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{} ", Icons::INFO), theme.style_info()),
                Span::styled("No alias suggestions available", theme.style_text_dim()),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Run more commands to get personalized suggestions:",
                theme.style_text(),
            )]),
            Line::from(vec![Span::styled(
                "• Use longer, repetitive commands",
                theme.style_text_dim(),
            )]),
            Line::from(vec![Span::styled(
                "• Work with git, docker, kubectl",
                theme.style_text_dim(),
            )]),
            Line::from(vec![Span::styled(
                "• Repeat commands multiple times",
                theme.style_text_dim(),
            )]),
        ]));
    } else {
        // Take only the first 10 suggestions to prevent excessive rendering
        for (i, suggestion) in analysis.suggestions.iter().enumerate().take(10) {
            let priority_icon = if suggestion.total_time_saved > 100 {
                ("🔥", theme.style_danger())
            } else if suggestion.total_time_saved > 50 {
                ("⚡", theme.style_warning())
            } else if suggestion.total_time_saved > 20 {
                ("💡", theme.style_info())
            } else {
                ("📝", theme.style_text_dim())
            };

            // Truncate long commands for display - ensure consistent length
            let display_command = if suggestion.command.len() > 50 {
                format!("{}...", &suggestion.command[..47])
            } else {
                suggestion.command.clone()
            };

            items.push(ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{:2}. ", i + 1), theme.style_text_dim()),
                    Span::styled(priority_icon.0, priority_icon.1),
                    Span::raw(" "),
                    Span::styled(
                        suggestion.suggested_alias.clone(),
                        theme.style_primary().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" → ", theme.style_accent()),
                    Span::styled(display_command, theme.style_text()),
                ]),
                Line::from(vec![
                    Span::raw("     "),
                    Span::styled(
                        format!("Used {} times", suggestion.frequency),
                        theme.style_text_dim(),
                    ),
                    Span::raw(" • "),
                    Span::styled(
                        format!("Saves {} chars each", suggestion.time_saved_per_use),
                        theme.style_success(),
                    ),
                    Span::raw(" • "),
                    Span::styled(
                        format!("Total: {} chars", suggestion.total_time_saved),
                        theme.style_accent(),
                    ),
                ]),
            ]));
        }
    }

    let suggestions_list = List::new(items)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::ALIASES), theme.style_accent()),
                    Span::styled("Smart Alias Suggestions", theme.style_title()),
                    Span::styled(
                        format!(" ({})", analysis.suggestions.len()),
                        theme.style_text_dim(),
                    ),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(suggestions_list, area);
}

fn draw_alias_analysis(
    f: &mut Frame,
    analysis: &crate::analysis::alias_suggest::AliasAnalysis,
    suggester: &AliasSuggester,
    area: Rect,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Efficiency gauge
            Constraint::Min(0),    // Existing aliases
        ])
        .split(area);

    // Efficiency gauge
    draw_efficiency_gauge(f, analysis, suggester, chunks[0], theme);

    // Existing aliases usage
    draw_existing_aliases(f, analysis, chunks[1], theme);
}

fn draw_efficiency_gauge(
    f: &mut Frame,
    analysis: &crate::analysis::alias_suggest::AliasAnalysis,
    suggester: &AliasSuggester,
    area: Rect,
    theme: &Theme,
) {
    let efficiency_gain = suggester.calculate_efficiency_gain(analysis);
    let efficiency_percentage = (efficiency_gain as u16).min(100);

    let gauge_color = if efficiency_gain > 70.0 {
        theme.style_success()
    } else if efficiency_gain > 40.0 {
        theme.style_warning()
    } else {
        theme.style_info()
    };

    let efficiency_gauge = Gauge::default()
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::LIGHTNING), theme.style_accent()),
                    Span::styled("Efficiency Potential", theme.style_title()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .gauge_style(gauge_color)
        .percent(efficiency_percentage)
        .label(format!("{}% Improvement Possible", efficiency_percentage));

    f.render_widget(efficiency_gauge, area);
}

fn draw_existing_aliases(
    f: &mut Frame,
    analysis: &crate::analysis::alias_suggest::AliasAnalysis,
    area: Rect,
    theme: &Theme,
) {
    let mut items = Vec::new();

    if analysis.existing_aliases_usage.is_empty() {
        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("{} ", Icons::INFO), theme.style_info()),
                Span::styled("No existing aliases detected", theme.style_text_dim()),
            ]),
            Line::from(vec![Span::styled(
                "Create aliases to see usage here",
                theme.style_text_dim(),
            )]),
        ]));
    } else {
        // Create a stable sorted vector to prevent flickering
        let mut aliases: Vec<(String, usize)> = analysis
            .existing_aliases_usage
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        // Sort by usage count (descending) then by name for stability
        aliases.sort_by(|a, b| {
            match b.1.cmp(&a.1) {
                std::cmp::Ordering::Equal => a.0.cmp(&b.0), // Secondary sort by name for stability
                other => other,
            }
        });

        for (i, (alias, usage_count)) in aliases.iter().enumerate().take(8) {
            let usage_icon = if *usage_count > 20 {
                ("🔥", theme.style_danger())
            } else if *usage_count > 10 {
                ("⚡", theme.style_warning())
            } else if *usage_count > 5 {
                ("💡", theme.style_info())
            } else {
                ("📝", theme.style_text_dim())
            };

            items.push(ListItem::new(vec![Line::from(vec![
                Span::styled(format!("{:2}. ", i + 1), theme.style_text_dim()),
                Span::styled(usage_icon.0, usage_icon.1),
                Span::raw(" "),
                Span::styled(
                    alias.clone(),
                    theme.style_primary().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(format!("({}×)", usage_count), theme.style_accent()),
            ])]));
        }
    }

    let aliases_list = List::new(items)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::STAR), theme.style_accent()),
                    Span::styled("Existing Aliases", theme.style_title()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(aliases_list, area);
}

fn draw_enhanced_footer(f: &mut Frame, area: Rect, theme: &Theme) {
    let footer_text = vec![
        Line::from(vec![
            Span::styled("Navigation: ", theme.style_accent()),
            Span::styled("↑↓", theme.style_primary()),
            Span::styled(" Navigate ", theme.style_text()),
            Span::styled("Enter", theme.style_primary()),
            Span::styled(" Select ", theme.style_text()),
            Span::styled("R", theme.style_primary()),
            Span::styled(" Refresh", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("Export: ", theme.style_accent()),
            Span::styled("B", theme.style_primary()),
            Span::styled("ash ", theme.style_text()),
            Span::styled("Z", theme.style_primary()),
            Span::styled("sh ", theme.style_text()),
            Span::styled("F", theme.style_primary()),
            Span::styled("ish ", theme.style_text()),
            Span::styled("C", theme.style_primary()),
            Span::styled(" Copy", theme.style_text()),
        ]),
    ];

    let footer = Paragraph::new(footer_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(footer, area);
}
