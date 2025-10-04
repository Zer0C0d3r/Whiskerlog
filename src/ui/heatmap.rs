use chrono::Weekday;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::analysis::heatmap::{HeatmapAnalyzer, TimeRange, ViewMode};
use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(5), // Enhanced header with controls and stats
                Constraint::Length(3), // Time range and view controls
                Constraint::Min(15),   // Main heatmap grid
                Constraint::Length(8), // Analysis panel and insights
            ]
            .as_ref(),
        )
        .split(area);

    // Enhanced header with metrics
    draw_heatmap_metrics(f, app, chunks[0]);

    // Interactive controls
    draw_heatmap_controls(f, app, chunks[1]);

    // Advanced heatmap visualization
    draw_advanced_heatmap(f, app, chunks[2]);

    // Analysis and insights panel
    draw_heatmap_insights(f, app, chunks[3]);
}

fn draw_heatmap_metrics(f: &mut Frame, app: &App, area: Rect) {
    let analyzer = HeatmapAnalyzer::new();
    let heatmap_data =
        analyzer.generate_heatmap(&app.commands, app.heatmap_time_range, app.heatmap_view_mode);
    let work_patterns = analyzer.analyze_work_patterns(&app.commands);

    // Create 4-column layout for metrics
    let metric_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(area);

    // Total Activity Metric
    let total_activity = format!("{}", heatmap_data.total_commands);
    let total_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "📊 TOTAL ACTIVITY",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                &total_activity,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" commands", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            format!("Peak: {:.0}", heatmap_data.max_activity),
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(total_block, metric_chunks[0]);

    // Peak Hours Metric
    let peak_hour = work_patterns.most_active_hour;
    let peak_day = format!("{:?}", work_patterns.most_active_day);
    let peak_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "⏰ PEAK ACTIVITY",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{}:00", peak_hour),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" hour", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            &peak_day,
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(peak_block, metric_chunks[1]);

    // Work Pattern Metric
    let work_ratio = (work_patterns.work_hours_ratio * 100.0) as u16;
    let pattern_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "💼 WORK PATTERN",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{}%", work_ratio),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" work hours", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            format!("{}% weekend", (work_patterns.weekend_ratio * 100.0) as u16),
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(pattern_block, metric_chunks[2]);

    // Activity Distribution
    let late_night_ratio = (work_patterns.late_night_ratio * 100.0) as u16;
    let distribution_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "🌙 NIGHT ACTIVITY",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{}%", late_night_ratio),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" late night", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            "22:00 - 06:00",
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(distribution_block, metric_chunks[3]);
}

fn draw_heatmap_controls(f: &mut Frame, app: &App, area: Rect) {
    let time_range_text = match app.heatmap_time_range {
        TimeRange::Day => "Day",
        TimeRange::Week => "Week",
        TimeRange::Month => "Month",
        TimeRange::Year => "Year",
    };

    let view_mode_text = match app.heatmap_view_mode {
        ViewMode::All => "All Commands",
        ViewMode::Dangerous => "Dangerous Only",
        ViewMode::Experiments => "Experiments Only",
        ViewMode::Failed => "Failed Commands",
    };

    let controls_text = vec![
        Line::from(vec![
            Span::styled("Time Range: ", Style::default().fg(Color::Cyan)),
            Span::styled("[D]", Style::default().fg(Color::Yellow)),
            Span::styled(
                "ay ",
                Style::default().fg(if matches!(app.heatmap_time_range, TimeRange::Day) {
                    Color::Yellow
                } else {
                    Color::White
                }),
            ),
            Span::styled("[W]", Style::default().fg(Color::Yellow)),
            Span::styled(
                "eek ",
                Style::default().fg(if matches!(app.heatmap_time_range, TimeRange::Week) {
                    Color::Yellow
                } else {
                    Color::White
                }),
            ),
            Span::styled("[M]", Style::default().fg(Color::Yellow)),
            Span::styled(
                "onth ",
                Style::default().fg(if matches!(app.heatmap_time_range, TimeRange::Month) {
                    Color::Yellow
                } else {
                    Color::White
                }),
            ),
            Span::styled("[Y]", Style::default().fg(Color::Yellow)),
            Span::styled(
                "ear",
                Style::default().fg(if matches!(app.heatmap_time_range, TimeRange::Year) {
                    Color::Yellow
                } else {
                    Color::White
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("View Mode: ", Style::default().fg(Color::Cyan)),
            Span::styled("[A]", Style::default().fg(Color::Green)),
            Span::styled(
                "ll ",
                Style::default().fg(if matches!(app.heatmap_view_mode, ViewMode::All) {
                    Color::Green
                } else {
                    Color::White
                }),
            ),
            Span::styled("[R]", Style::default().fg(Color::Red)),
            Span::styled(
                "isky ",
                Style::default().fg(if matches!(app.heatmap_view_mode, ViewMode::Dangerous) {
                    Color::Red
                } else {
                    Color::White
                }),
            ),
            Span::styled("[E]", Style::default().fg(Color::Blue)),
            Span::styled(
                "xperiments ",
                Style::default().fg(if matches!(app.heatmap_view_mode, ViewMode::Experiments) {
                    Color::Blue
                } else {
                    Color::White
                }),
            ),
            Span::styled("[F]", Style::default().fg(Color::Magenta)),
            Span::styled(
                "ailed",
                Style::default().fg(if matches!(app.heatmap_view_mode, ViewMode::Failed) {
                    Color::Magenta
                } else {
                    Color::White
                }),
            ),
            Span::raw("  |  "),
            Span::styled("Current: ", Style::default().fg(Color::Gray)),
            Span::styled(
                time_range_text,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" • "),
            Span::styled(
                view_mode_text,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" • "),
            Span::styled(
                format!(
                    "State: {:?}/{:?}",
                    app.heatmap_time_range, app.heatmap_view_mode
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
    ];

    let controls = Paragraph::new(controls_text)
        .block(
            Block::default()
                .title("🎛️  Interactive Controls")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(controls, area);
}

fn draw_advanced_heatmap(f: &mut Frame, app: &App, area: Rect) {
    let analyzer = HeatmapAnalyzer::new();
    let heatmap_data =
        analyzer.generate_heatmap(&app.commands, app.heatmap_time_range, app.heatmap_view_mode);

    let mut heatmap_lines = Vec::new();

    // Enhanced header with day abbreviations and better spacing
    heatmap_lines.push(Line::from(vec![
        Span::raw("      "),
        Span::styled(
            "Mon  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Tue  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Wed  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Thu  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Fri  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Sat  ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Sun  ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    heatmap_lines.push(Line::from(""));

    // Generate heatmap grid with enhanced visualization
    for hour in 0..24 {
        let mut line_spans = vec![Span::styled(
            format!("{:2}:00 ", hour),
            Style::default().fg(Color::Gray),
        )];

        // Real activity data for each day of the week
        for day in 0..7 {
            let activity_level = heatmap_data.grid[hour][day];
            let (color, _symbol) = get_activity_visualization(activity_level);

            // Add tooltip-like information for high activity periods
            let display_symbol = if activity_level > 0.8 {
                "██"
            } else if activity_level > 0.6 {
                "▓▓"
            } else if activity_level > 0.4 {
                "▒▒"
            } else if activity_level > 0.2 {
                "░░"
            } else if activity_level > 0.0 {
                "··"
            } else {
                "  "
            };

            line_spans.push(Span::styled(
                format!("{} ", display_symbol),
                Style::default().fg(color),
            ));
        }

        // Add activity level indicator
        let hour_total: f32 = heatmap_data.grid[hour].iter().sum();
        let hour_avg = hour_total / 7.0;
        let intensity_bar = get_intensity_bar(hour_avg);
        line_spans.push(Span::styled(
            format!(" {}", intensity_bar),
            Style::default().fg(Color::Gray),
        ));

        heatmap_lines.push(Line::from(line_spans));
    }

    // Enhanced legend with more detail
    heatmap_lines.push(Line::from(""));
    heatmap_lines.push(Line::from(vec![
        Span::styled(
            "Activity Levels: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default().fg(Color::Gray)),
        Span::raw("None  "),
        Span::styled("··", Style::default().fg(Color::Blue)),
        Span::raw("Low  "),
        Span::styled("░░", Style::default().fg(Color::Green)),
        Span::raw("Med  "),
        Span::styled("▒▒", Style::default().fg(Color::Yellow)),
        Span::raw("High  "),
        Span::styled("▓▓", Style::default().fg(Color::Red)),
        Span::raw("Very High  "),
        Span::styled("██", Style::default().fg(Color::Magenta)),
        Span::raw("Extreme"),
    ]));

    // Add time range specific information with fallback indication
    let now = chrono::Utc::now();
    let cutoff_time = match app.heatmap_time_range {
        TimeRange::Day => now - chrono::Duration::days(1),
        TimeRange::Week => now - chrono::Duration::days(7),
        TimeRange::Month => now - chrono::Duration::days(30),
        TimeRange::Year => now - chrono::Duration::days(365),
    };

    let recent_count = app
        .commands
        .iter()
        .filter(|cmd| cmd.timestamp >= cutoff_time)
        .count();
    let is_fallback = recent_count == 0 && heatmap_data.total_commands > 0;

    let time_info = match app.heatmap_time_range {
        TimeRange::Day => {
            if is_fallback {
                format!(
                    "📅 Last 24 Hours • {} recent commands (showing {} historical)",
                    recent_count, heatmap_data.total_commands
                )
            } else {
                format!(
                    "📅 Last 24 Hours • {} commands",
                    heatmap_data.total_commands
                )
            }
        }
        TimeRange::Week => {
            if is_fallback {
                format!(
                    "📅 Last 7 Days • {} recent commands (showing {} historical)",
                    recent_count, heatmap_data.total_commands
                )
            } else {
                format!("📅 Last 7 Days • {} commands", heatmap_data.total_commands)
            }
        }
        TimeRange::Month => {
            if is_fallback {
                format!(
                    "📅 Last 30 Days • {} recent commands (showing {} historical)",
                    recent_count, heatmap_data.total_commands
                )
            } else {
                format!("📅 Last 30 Days • {} commands", heatmap_data.total_commands)
            }
        }
        TimeRange::Year => {
            format!(
                "📅 Last 365 Days • {} commands",
                heatmap_data.total_commands
            )
        }
    };

    heatmap_lines.push(Line::from(vec![
        Span::styled(&time_info, Style::default().fg(Color::Yellow)),
        Span::raw("  |  "),
        Span::styled(
            format!(
                "Peak Activity: {:.0} commands/hour",
                heatmap_data.max_activity
            ),
            Style::default().fg(Color::Green),
        ),
    ]));

    // Add helpful message if showing fallback data
    if is_fallback {
        heatmap_lines.push(Line::from(vec![
            Span::styled(
                "ℹ️  No recent commands found - showing historical data",
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" (try running some commands to see recent activity)"),
        ]));
    }

    let title = format!(
        "🔥 Activity Heatmap - {} View",
        match app.heatmap_view_mode {
            ViewMode::All => "All Commands",
            ViewMode::Dangerous => "Dangerous Commands",
            ViewMode::Experiments => "Experiments",
            ViewMode::Failed => "Failed Commands",
        }
    );

    let heatmap = Paragraph::new(heatmap_lines)
        .block(Block::default().title(title).borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(heatmap, area);
}

fn draw_heatmap_insights(f: &mut Frame, app: &App, area: Rect) {
    let analyzer = HeatmapAnalyzer::new();
    let work_patterns = analyzer.analyze_work_patterns(&app.commands);
    let peak_periods = analyzer.get_peak_activity_periods(
        &analyzer.generate_heatmap(&app.commands, app.heatmap_time_range, app.heatmap_view_mode),
        0.6, // threshold for "peak" activity
    );

    // Split into two columns
    let insight_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    // Left panel: Work Pattern Analysis
    let mut pattern_lines = Vec::new();
    pattern_lines.push(Line::from(vec![Span::styled(
        "📊 Work Pattern Analysis",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    pattern_lines.push(Line::from(""));

    // Work vs Personal time
    let work_percentage = (work_patterns.work_hours_ratio * 100.0) as u16;
    pattern_lines.push(Line::from(vec![
        Span::styled("Business Hours (9-17): ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{}%", work_percentage),
            Style::default().fg(Color::Green),
        ),
    ]));

    let weekday_percentage = (work_patterns.weekday_ratio * 100.0) as u16;
    pattern_lines.push(Line::from(vec![
        Span::styled("Weekday Activity:     ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{}%", weekday_percentage),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    let weekend_percentage = (work_patterns.weekend_ratio * 100.0) as u16;
    pattern_lines.push(Line::from(vec![
        Span::styled("Weekend Activity:     ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{}%", weekend_percentage),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    let night_percentage = (work_patterns.late_night_ratio * 100.0) as u16;
    pattern_lines.push(Line::from(vec![
        Span::styled("Late Night (22-06):   ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{}%", night_percentage),
            Style::default().fg(Color::Magenta),
        ),
    ]));

    pattern_lines.push(Line::from(""));
    pattern_lines.push(Line::from(vec![
        Span::styled("Most Active Day: ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{:?}", work_patterns.most_active_day),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    pattern_lines.push(Line::from(vec![
        Span::styled("Peak Hour:       ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{}:00", work_patterns.most_active_hour),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let pattern_panel = Paragraph::new(pattern_lines)
        .block(Block::default().title("📈 Insights").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(pattern_panel, insight_chunks[0]);

    // Right panel: Peak Activity Periods
    let mut peak_lines = Vec::new();
    peak_lines.push(Line::from(vec![Span::styled(
        "🔥 Peak Activity Periods",
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )]));
    peak_lines.push(Line::from(""));

    for (i, period) in peak_periods.iter().take(5).enumerate() {
        let day_name = match period.day_of_week {
            Weekday::Mon => "Monday",
            Weekday::Tue => "Tuesday",
            Weekday::Wed => "Wednesday",
            Weekday::Thu => "Thursday",
            Weekday::Fri => "Friday",
            Weekday::Sat => "Saturday",
            Weekday::Sun => "Sunday",
        };

        let intensity_color = if period.activity_level > 0.9 {
            Color::Red
        } else if period.activity_level > 0.8 {
            Color::Yellow
        } else {
            Color::Green
        };

        peak_lines.push(Line::from(vec![
            Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Gray)),
            Span::styled(day_name, Style::default().fg(Color::White)),
            Span::styled(
                format!(" {}:00", period.hour),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" - "),
            Span::styled(
                format!("{} cmds", period.command_count),
                Style::default()
                    .fg(intensity_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    if peak_periods.is_empty() {
        peak_lines.push(Line::from(vec![Span::styled(
            "No significant peak periods found",
            Style::default().fg(Color::Gray),
        )]));
    }

    let peak_panel = Paragraph::new(peak_lines)
        .block(
            Block::default()
                .title("⚡ Top Periods")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(peak_panel, insight_chunks[1]);
}

fn get_activity_visualization(level: f32) -> (Color, &'static str) {
    match level {
        x if x > 0.9 => (Color::Magenta, "██"),
        x if x > 0.8 => (Color::Red, "▓▓"),
        x if x > 0.6 => (Color::Yellow, "▒▒"),
        x if x > 0.4 => (Color::Green, "░░"),
        x if x > 0.2 => (Color::Blue, "··"),
        x if x > 0.0 => (Color::Gray, "··"),
        _ => (Color::Gray, "  "),
    }
}

fn get_intensity_bar(intensity: f32) -> String {
    let bars = (intensity * 5.0) as usize;
    let filled = "▰".repeat(bars);
    let empty = "▱".repeat(5 - bars);
    format!("{}{}", filled, empty)
}
