use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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
        .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
        .split(area);

    // Header with sorting options and filters
    draw_header(f, chunks[0], &theme);

    // Commands list with enhanced styling
    draw_commands_list(f, app, chunks[1], &theme);
}

fn draw_header(f: &mut Frame, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    // Sort options
    let sort_text = vec![
        Line::from(vec![
            Span::styled(format!("{} ", Icons::GEAR), theme.style_accent()),
            Span::styled("Sort by:", theme.style_title()),
        ]),
        Line::from(vec![
            Span::styled("[T] ", theme.style_primary()),
            Span::styled("Time ", theme.style_text()),
            Span::styled("[C] ", theme.style_primary()),
            Span::styled("Count ", theme.style_text()),
            Span::styled("[H] ", theme.style_primary()),
            Span::styled("Host ", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("[D] ", theme.style_primary()),
            Span::styled("Danger ", theme.style_text()),
            Span::styled("[S] ", theme.style_primary()),
            Span::styled("Success ", theme.style_text()),
            Span::styled("[L] ", theme.style_primary()),
            Span::styled("Length", theme.style_text()),
        ]),
    ];

    let sort_paragraph = Paragraph::new(sort_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(sort_paragraph, chunks[0]);

    // Filter options
    let filter_text = vec![
        Line::from(vec![
            Span::styled(format!("{} ", Icons::SEARCH), theme.style_accent()),
            Span::styled("Filters:", theme.style_title()),
        ]),
        Line::from(vec![
            Span::styled("[F] ", theme.style_danger()),
            Span::styled("Failed ", theme.style_text()),
            Span::styled("[E] ", theme.style_warning()),
            Span::styled("Experiments ", theme.style_text()),
        ]),
        Line::from(vec![
            Span::styled("[R] ", theme.style_info()),
            Span::styled("Recent ", theme.style_text()),
            Span::styled("[A] ", theme.style_success()),
            Span::styled("All ", theme.style_text()),
        ]),
    ];

    let filter_paragraph = Paragraph::new(filter_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(filter_paragraph, chunks[1]);
}

fn draw_commands_list(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let visible_commands = app
        .get_filtered_commands()
        .iter()
        .skip(app.scroll_offset)
        .take(area.height as usize - 2); // Account for borders

    let command_items: Vec<ListItem> = visible_commands
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = app.scroll_offset + i == app.selected_index;
            let global_index = app.scroll_offset + i;

            let command_style = if is_selected {
                theme.style_selected()
            } else if cmd.is_dangerous {
                theme.style_danger()
            } else if cmd.is_experiment {
                theme.style_warning()
            } else {
                theme.style_text()
            };

            let time_str = cmd.timestamp.format("%m-%d %H:%M:%S").to_string();

            let (exit_icon, exit_style) = match cmd.exit_code {
                Some(0) => (Icons::SUCCESS, theme.style_success()),
                Some(_) => (Icons::ERROR, theme.style_danger()),
                None => (Icons::QUESTION, theme.style_text_dim()),
            };

            let host_icon = get_host_icon(&cmd.host_id);

            let duration_str = cmd
                .duration
                .map(|d| {
                    if d > 1000 {
                        format!("{}s", d / 1000)
                    } else {
                        format!("{}ms", d)
                    }
                })
                .unwrap_or_else(|| "-".to_string());

            // Truncate long commands
            let display_command = if cmd.command.len() > 60 {
                format!("{}...", &cmd.command[..57])
            } else {
                cmd.command.clone()
            };

            // Add indicators for special command types
            let mut indicators = Vec::new();
            if cmd.is_dangerous {
                indicators.push(Span::styled(
                    format!(" {}", Icons::DANGEROUS),
                    theme.style_danger(),
                ));
            }
            if cmd.is_experiment {
                indicators.push(Span::styled(
                    format!(" {}", Icons::EXPERIMENTS),
                    theme.style_warning(),
                ));
            }
            if !cmd.network_endpoints.is_empty() {
                indicators.push(Span::styled(
                    format!(" {}", Icons::NETWORK),
                    theme.style_info(),
                ));
            }
            if !cmd.packages_used.is_empty() {
                indicators.push(Span::styled(
                    format!(" {}", Icons::PACKAGES),
                    theme.style_secondary(),
                ));
            }

            let mut line_spans = vec![
                Span::styled(format!("{:3}. ", global_index + 1), theme.style_text_dim()),
                Span::styled(format!("{} ", exit_icon), exit_style),
                Span::styled(time_str, theme.style_text_dim()),
                Span::raw(" "),
                Span::styled(format!("{} ", host_icon), theme.style_secondary()),
                Span::styled(display_command, command_style),
                Span::raw(" "),
                Span::styled(format!("[{}]", duration_str), theme.style_text_dim()),
            ];

            line_spans.extend(indicators);

            ListItem::new(Line::from(line_spans))
        })
        .collect();

    let total_commands = app.get_filtered_commands().len();
    let showing_start = app.scroll_offset + 1;
    let showing_end = (app.scroll_offset + command_items.len()).min(total_commands);

    let commands_list = List::new(command_items)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::COMMANDS), theme.style_accent()),
                    Span::styled("All Commands", theme.style_title()),
                    Span::styled(
                        format!(" ({}-{} of {})", showing_start, showing_end, total_commands),
                        theme.style_text_dim(),
                    ),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(commands_list, area);
}
