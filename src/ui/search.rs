use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::theme::{get_host_icon, Icons, Theme};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Search input and filters
            Constraint::Min(0),    // Results
            Constraint::Length(3), // Status bar
        ])
        .split(area);

    // Search input area
    draw_search_input(f, app, chunks[0], &theme);

    // Search results
    draw_search_results(f, app, chunks[1], &theme);

    // Search status
    draw_search_status(f, app, chunks[2], &theme);
}

fn draw_search_input(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Search input
            Constraint::Percentage(40), // Filters
        ])
        .split(area);

    // Search input box
    let search_text = if app.search_mode {
        format!("{}_", app.search_query) // Show cursor
    } else if app.search_query.is_empty() {
        "Type to search commands...".to_string()
    } else {
        app.search_query.clone()
    };

    let input_style = if app.search_mode {
        theme.style_primary()
    } else if app.search_query.is_empty() {
        theme.style_text_dim()
    } else {
        theme.style_text()
    };

    let search_input = Paragraph::new(search_text)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::SEARCH), theme.style_primary()),
                    Span::styled("Search", theme.style_title()),
                ]))
                .borders(Borders::ALL)
                .border_style(if app.search_mode {
                    theme.style_primary()
                } else {
                    theme.style_border()
                }),
        )
        .style(input_style);

    f.render_widget(search_input, input_chunks[0]);

    // Filter options with active filter highlighting
    let active_filter = app.get_search_filter();

    let filter_text = vec![
        Line::from(vec![
            Span::styled("Filters: ", theme.style_accent()),
            Span::styled(
                match active_filter {
                    crate::app::SearchFilter::None => "None",
                    crate::app::SearchFilter::Failed => "Failed",
                    crate::app::SearchFilter::Dangerous => "Dangerous",
                    crate::app::SearchFilter::Recent => "Recent",
                    crate::app::SearchFilter::Experiments => "Experiments",
                },
                theme.style_primary(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "[F1] ",
                if matches!(active_filter, crate::app::SearchFilter::Failed) {
                    theme.style_accent()
                } else {
                    theme.style_primary()
                },
            ),
            Span::styled(
                "Failed ",
                if matches!(active_filter, crate::app::SearchFilter::Failed) {
                    theme.style_accent()
                } else {
                    theme.style_text()
                },
            ),
            Span::styled(
                "[F2] ",
                if matches!(active_filter, crate::app::SearchFilter::Dangerous) {
                    theme.style_accent()
                } else {
                    theme.style_primary()
                },
            ),
            Span::styled(
                "Dangerous ",
                if matches!(active_filter, crate::app::SearchFilter::Dangerous) {
                    theme.style_accent()
                } else {
                    theme.style_text()
                },
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "[F3] ",
                if matches!(active_filter, crate::app::SearchFilter::Recent) {
                    theme.style_accent()
                } else {
                    theme.style_primary()
                },
            ),
            Span::styled(
                "Recent ",
                if matches!(active_filter, crate::app::SearchFilter::Recent) {
                    theme.style_accent()
                } else {
                    theme.style_text()
                },
            ),
            Span::styled(
                "[F4] ",
                if matches!(active_filter, crate::app::SearchFilter::Experiments) {
                    theme.style_accent()
                } else {
                    theme.style_primary()
                },
            ),
            Span::styled(
                "Experiments",
                if matches!(active_filter, crate::app::SearchFilter::Experiments) {
                    theme.style_accent()
                } else {
                    theme.style_text()
                },
            ),
        ]),
    ];

    let filters = Paragraph::new(filter_text)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::GEAR), theme.style_accent()),
                    Span::styled("Filters", theme.style_title()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(filters, input_chunks[1]);
}

fn draw_search_results(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let results = perform_search(app);

    if results.is_empty() && !app.search_query.is_empty() {
        // No results found
        let no_results = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{} ", Icons::SEARCH), theme.style_text_dim()),
                Span::styled(
                    "No commands found matching your search",
                    theme.style_text_dim(),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("Try: ", theme.style_accent())]),
            Line::from(vec![
                Span::styled("• ", theme.style_text_dim()),
                Span::styled("Different keywords", theme.style_text()),
            ]),
            Line::from(vec![
                Span::styled("• ", theme.style_text_dim()),
                Span::styled("Partial command names", theme.style_text()),
            ]),
            Line::from(vec![
                Span::styled("• ", theme.style_text_dim()),
                Span::styled("Use filters (F1-F4)", theme.style_text()),
            ]),
        ];

        let paragraph = Paragraph::new(no_results)
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(format!("{} ", Icons::SEARCH), theme.style_accent()),
                        Span::styled("Search Results", theme.style_title()),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(theme.style_border()),
            )
            .alignment(Alignment::Center)
            .style(theme.style_text());

        f.render_widget(paragraph, area);
        return;
    }

    // Display search results
    let results_count = results.len();
    let result_items: Vec<ListItem> = results
        .into_iter()
        .enumerate()
        .take(area.height as usize - 2) // Account for borders
        .map(|(i, (cmd, score))| {
            let is_selected = i == app.selected_index;

            let style = if is_selected {
                theme.style_selected()
            } else if cmd.is_dangerous {
                theme.style_danger()
            } else if cmd.is_experiment {
                theme.style_warning()
            } else {
                theme.style_text()
            };

            let time_str = cmd.timestamp.format("%m-%d %H:%M").to_string();
            let host_icon = get_host_icon(&cmd.host_id);

            let (status_icon, status_style) = match cmd.exit_code {
                Some(0) => (Icons::SUCCESS, theme.style_success()),
                Some(_) => (Icons::ERROR, theme.style_danger()),
                None => ("", theme.style_text_dim()),
            };

            // Highlight matching parts (simplified)
            let highlighted_command = highlight_matches(&cmd.command, &app.search_query, theme);

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:2}. ", i + 1), theme.style_text_dim()),
                Span::styled(format!("{} ", status_icon), status_style),
                Span::styled(time_str, theme.style_text_dim()),
                Span::raw(" "),
                Span::styled(format!("{} ", host_icon), theme.style_secondary()),
                Span::styled(highlighted_command, style),
                Span::styled(format!(" ({:.0}%)", score * 100.0), theme.style_text_dim()),
            ]))
        })
        .collect();

    let results_list = List::new(result_items)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::SEARCH), theme.style_accent()),
                    Span::styled("Search Results", theme.style_title()),
                    Span::styled(format!(" ({})", results_count), theme.style_text_dim()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(results_list, area);
}

fn draw_search_status(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let status_text = if app.search_query.is_empty() {
        vec![Line::from(vec![
            Span::styled("Start typing to search • ", theme.style_text_dim()),
            Span::styled("F1-F4", theme.style_primary()),
            Span::styled(" for filters • ", theme.style_text_dim()),
            Span::styled("Enter", theme.style_primary()),
            Span::styled(" to select • ", theme.style_text_dim()),
            Span::styled("Esc", theme.style_primary()),
            Span::styled(" to clear", theme.style_text_dim()),
        ])]
    } else {
        let results_count = perform_search(app).len();
        vec![Line::from(vec![
            Span::styled(
                format!("{} results for '", results_count),
                theme.style_text(),
            ),
            Span::styled(&app.search_query, theme.style_primary()),
            Span::styled("' • ", theme.style_text()),
            Span::styled("↑↓", theme.style_primary()),
            Span::styled(" navigate • ", theme.style_text_dim()),
            Span::styled("Enter", theme.style_primary()),
            Span::styled(" select • ", theme.style_text_dim()),
            Span::styled("Esc", theme.style_primary()),
            Span::styled(" clear", theme.style_text_dim()),
        ])]
    };

    let status = Paragraph::new(status_text)
        .style(theme.style_text())
        .alignment(Alignment::Center);

    f.render_widget(status, area);
}

fn perform_search(app: &App) -> Vec<(&crate::history::Command, f64)> {
    if app.search_query.is_empty() {
        return Vec::new();
    }

    let matcher = SkimMatcherV2::default();

    // First apply search filter
    let filtered_commands: Vec<_> = match app.search_filter {
        crate::app::SearchFilter::None => app.commands.iter().collect(),
        crate::app::SearchFilter::Failed => app
            .commands
            .iter()
            .filter(|cmd| cmd.exit_code.is_some() && cmd.exit_code.unwrap() != 0)
            .collect(),
        crate::app::SearchFilter::Dangerous => {
            app.commands.iter().filter(|cmd| cmd.is_dangerous).collect()
        }
        crate::app::SearchFilter::Recent => {
            let mut recent: Vec<_> = app.commands.iter().collect();
            recent.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            recent.into_iter().take(100).collect()
        }
        crate::app::SearchFilter::Experiments => app
            .commands
            .iter()
            .filter(|cmd| cmd.is_experiment)
            .collect(),
    };

    let mut results: Vec<_> = filtered_commands
        .into_iter()
        .filter_map(|cmd| {
            matcher
                .fuzzy_match(&cmd.command, &app.search_query)
                .map(|score| (cmd, score as f64 / 100.0))
        })
        .collect();

    // Sort by score (highest first)
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Limit results
    results.truncate(100);

    results
}

fn highlight_matches(text: &str, query: &str, _theme: &Theme) -> String {
    // Simple highlighting - in a real implementation, you'd want more sophisticated matching
    if query.is_empty() {
        return text.to_string();
    }

    // For now, just return the original text
    // In a full implementation, you'd highlight matching characters
    text.to_string()
}
