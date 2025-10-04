use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::analysis::network_analyzer::NetworkAnalyzer;
use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(5), // Header with real-time metrics
                Constraint::Length(3), // Interactive controls
                Constraint::Min(10),   // Main content area
                Constraint::Length(8), // Real-time analytics footer
            ]
            .as_ref(),
        )
        .split(area);

    // Real-time network metrics header
    draw_network_metrics(f, app, chunks[0]);

    // Interactive controls
    draw_network_controls(f, app, chunks[1]);

    // Main content with endpoints and analysis
    draw_network_content(f, app, chunks[2]);

    // Real-time analytics and insights
    draw_network_analytics(f, app, chunks[3]);
}

fn draw_network_metrics(f: &mut Frame, app: &App, area: Rect) {
    let analyzer = NetworkAnalyzer::new();
    let analysis = analyzer.analyze_network_activity(&app.commands);

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

    // Total Endpoints Metric
    let endpoints_count = analysis.unique_endpoints;
    let endpoints_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "🌐 ENDPOINTS",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{}", endpoints_count),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" unique", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            "Discovered",
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(endpoints_block, metric_chunks[0]);

    // Security Score Metric
    let security_score = analyzer.calculate_network_security_score(&analysis);
    let security_color = if security_score > 80.0 {
        Color::Green
    } else if security_score > 60.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    let security_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "🔒 SECURITY",
            Style::default()
                .fg(security_color)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{:.0}%", security_score),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" score", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            "Safety rating",
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(security_block, metric_chunks[1]);

    // Protocol Distribution Metric
    let https_count = analysis.protocol_breakdown.get("https").unwrap_or(&0);
    let total_protocols = analysis.protocol_breakdown.values().sum::<usize>();
    let https_ratio = if total_protocols > 0 {
        (*https_count as f32 / total_protocols as f32 * 100.0) as u16
    } else {
        0
    };
    let protocol_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "📊 PROTOCOLS",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{}%", https_ratio),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" secure", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            "HTTPS usage",
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(protocol_block, metric_chunks[2]);

    // Security Issues Metric
    let issues_count = analysis.security_issues.len();
    let issues_color = if issues_count == 0 {
        Color::Green
    } else if issues_count < 5 {
        Color::Yellow
    } else {
        Color::Red
    };
    let issues_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "⚠️  ISSUES",
            Style::default()
                .fg(issues_color)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{}", issues_count),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" found", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            "Security alerts",
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(issues_block, metric_chunks[3]);
}

fn draw_network_controls(f: &mut Frame, _app: &App, area: Rect) {
    let controls_text = vec![Line::from(vec![
        Span::styled("Filter: ", Style::default().fg(Color::Cyan)),
        Span::styled("[S]", Style::default().fg(Color::Green)),
        Span::styled("ecure ", Style::default().fg(Color::White)),
        Span::styled("[I]", Style::default().fg(Color::Red)),
        Span::styled("nsecure ", Style::default().fg(Color::White)),
        Span::styled("[A]", Style::default().fg(Color::Blue)),
        Span::styled("ll ", Style::default().fg(Color::White)),
        Span::raw("  |  "),
        Span::styled("Sort: ", Style::default().fg(Color::Cyan)),
        Span::styled("[U]", Style::default().fg(Color::Yellow)),
        Span::styled("sage ", Style::default().fg(Color::White)),
        Span::styled("[T]", Style::default().fg(Color::Magenta)),
        Span::styled("ime ", Style::default().fg(Color::White)),
        Span::styled("[R]", Style::default().fg(Color::Green)),
        Span::styled("isk", Style::default().fg(Color::White)),
    ])];

    let controls = Paragraph::new(controls_text)
        .block(
            Block::default()
                .title("🎛️  Network Controls")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(controls, area);
}

fn draw_network_content(f: &mut Frame, app: &App, area: Rect) {
    // Split into three panels: endpoints, security issues, and connection patterns
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(40), // Endpoints list
                Constraint::Percentage(30), // Security issues
                Constraint::Percentage(30), // Connection patterns
            ]
            .as_ref(),
        )
        .split(area);

    // Left panel: Enhanced endpoints list
    draw_enhanced_endpoints_list(f, app, content_chunks[0]);

    // Middle panel: Security issues
    draw_security_issues_panel(f, app, content_chunks[1]);

    // Right panel: Connection patterns
    draw_connection_patterns_panel(f, app, content_chunks[2]);
}

fn draw_enhanced_endpoints_list(f: &mut Frame, app: &App, area: Rect) {
    let analyzer = NetworkAnalyzer::new();
    let analysis = analyzer.analyze_network_activity(&app.commands);

    let mut items = Vec::new();

    for (i, endpoint_stats) in analysis.top_endpoints.iter().enumerate() {
        let is_selected = i == app.selected_index;

        // Protocol icon and security indicator
        let (protocol_icon, security_color) = match endpoint_stats.protocol.as_str() {
            "https" => ("🔒", Color::Green),
            "http" => ("🔓", Color::Red),
            "ssh" => ("🔗", Color::Blue),
            "ftp" => ("📁", Color::Yellow),
            "telnet" => ("⚠️", Color::Red),
            _ => ("🌐", Color::Gray),
        };

        // Risk level indicator
        let risk_indicator = if !endpoint_stats.is_secure {
            " 🚨"
        } else if endpoint_stats.success_rate < 0.9 {
            " ⚠️"
        } else {
            ""
        };

        // Format endpoint with truncation
        let display_endpoint = if endpoint_stats.endpoint.len() > 35 {
            format!("{}...", &endpoint_stats.endpoint[..32])
        } else {
            endpoint_stats.endpoint.clone()
        };

        let item_style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        items.push(
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(protocol_icon, Style::default().fg(security_color)),
                    Span::raw(" "),
                    Span::styled(display_endpoint, Style::default().fg(Color::White)),
                    Span::styled(risk_indicator, Style::default().fg(Color::Red)),
                ]),
                Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        format!("{}× used", endpoint_stats.usage_count),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::raw(" • "),
                    Span::styled(
                        format!("{:.1}% success", endpoint_stats.success_rate * 100.0),
                        Style::default().fg(Color::Green),
                    ),
                ]),
            ])
            .style(item_style),
        );

        if i >= 9 {
            break;
        } // Limit to 10 endpoints for display
    }

    if items.is_empty() {
        items.push(ListItem::new(vec![
            Line::from(vec![Span::styled(
                "🔍 No network endpoints found",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![Span::styled(
                "   Run commands with network activity",
                Style::default().fg(Color::Gray),
            )]),
        ]));
    }

    let endpoints_list = List::new(items)
        .block(
            Block::default()
                .title("🌐 Network Endpoints")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(endpoints_list, area);
}

fn draw_security_issues_panel(f: &mut Frame, app: &App, area: Rect) {
    let analyzer = NetworkAnalyzer::new();
    let analysis = analyzer.analyze_network_activity(&app.commands);

    let mut items = Vec::new();

    for (i, issue) in analysis.security_issues.iter().enumerate() {
        let (severity_icon, severity_color) = match issue.severity {
            crate::analysis::network_analyzer::SecuritySeverity::Critical => ("🚨", Color::Red),
            crate::analysis::network_analyzer::SecuritySeverity::High => ("⚠️", Color::Red),
            crate::analysis::network_analyzer::SecuritySeverity::Medium => ("⚡", Color::Yellow),
            crate::analysis::network_analyzer::SecuritySeverity::Low => ("ℹ️", Color::Blue),
        };

        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled(severity_icon, Style::default().fg(severity_color)),
                Span::raw(" "),
                Span::styled(
                    &issue.issue_type,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("   "),
                Span::styled(&issue.description, Style::default().fg(Color::Gray)),
            ]),
        ]));

        if i >= 7 {
            break;
        } // Limit to 8 issues for display
    }

    if items.is_empty() {
        items.push(ListItem::new(vec![
            Line::from(vec![Span::styled(
                "✅ No security issues found",
                Style::default().fg(Color::Green),
            )]),
            Line::from(vec![Span::styled(
                "   Your network usage looks secure",
                Style::default().fg(Color::Gray),
            )]),
        ]));
    }

    let issues_list = List::new(items)
        .block(
            Block::default()
                .title("🔒 Security Issues")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(issues_list, area);
}

fn draw_connection_patterns_panel(f: &mut Frame, app: &App, area: Rect) {
    let analyzer = NetworkAnalyzer::new();
    let analysis = analyzer.analyze_network_activity(&app.commands);

    let mut items = Vec::new();

    for (i, pattern) in analysis.connection_patterns.iter().enumerate() {
        let (risk_icon, risk_color) = match pattern.risk_level {
            crate::analysis::network_analyzer::SecuritySeverity::Critical => ("🚨", Color::Red),
            crate::analysis::network_analyzer::SecuritySeverity::High => ("⚠️", Color::Red),
            crate::analysis::network_analyzer::SecuritySeverity::Medium => ("⚡", Color::Yellow),
            crate::analysis::network_analyzer::SecuritySeverity::Low => ("✅", Color::Green),
        };

        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled(risk_icon, Style::default().fg(risk_color)),
                Span::raw(" "),
                Span::styled(
                    &pattern.pattern_type,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("   "),
                Span::styled(&pattern.description, Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled(
                    format!("({}×)", pattern.frequency),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ]));

        if i >= 6 {
            break;
        } // Limit to 7 patterns for display
    }

    if items.is_empty() {
        items.push(ListItem::new(vec![
            Line::from(vec![Span::styled(
                "📊 No patterns detected",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![Span::styled(
                "   More data needed for analysis",
                Style::default().fg(Color::Gray),
            )]),
        ]));
    }

    let patterns_list = List::new(items)
        .block(
            Block::default()
                .title("📊 Connection Patterns")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(patterns_list, area);
}

fn draw_network_analytics(f: &mut Frame, app: &App, area: Rect) {
    let analyzer = NetworkAnalyzer::new();
    let analysis = analyzer.analyze_network_activity(&app.commands);

    let analytics_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(40), // Protocol distribution chart
                Constraint::Percentage(30), // Security gauge
                Constraint::Percentage(30), // Real-time insights
            ]
            .as_ref(),
        )
        .split(area);

    // Left: Protocol distribution chart
    draw_protocol_chart(f, &analysis, analytics_chunks[0]);

    // Middle: Security gauge
    draw_security_gauge(f, &analysis, &analyzer, analytics_chunks[1]);

    // Right: Real-time insights
    draw_realtime_insights(f, &analysis, analytics_chunks[2]);
}

fn draw_protocol_chart(
    f: &mut Frame,
    analysis: &crate::analysis::network_analyzer::NetworkAnalysis,
    area: Rect,
) {
    let mut chart_data = Vec::new();

    for (protocol, count) in &analysis.protocol_breakdown {
        if *count > 0 {
            chart_data.push((protocol.as_str(), *count as u64));
        }
    }

    // Sort by count and take top 6
    chart_data.sort_by(|a, b| b.1.cmp(&a.1));
    chart_data.truncate(6);

    if !chart_data.is_empty() {
        let bar_chart = BarChart::default()
            .block(
                Block::default()
                    .title("📊 Protocol Usage")
                    .borders(Borders::ALL),
            )
            .data(&chart_data)
            .bar_width(8)
            .bar_style(Style::default().fg(Color::Cyan))
            .value_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(bar_chart, area);
    } else {
        let no_data = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "📊 Protocol Usage",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "No protocol data available",
                Style::default().fg(Color::Gray),
            )]),
        ])
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

        f.render_widget(no_data, area);
    }
}

fn draw_security_gauge(
    f: &mut Frame,
    analysis: &crate::analysis::network_analyzer::NetworkAnalysis,
    analyzer: &NetworkAnalyzer,
    area: Rect,
) {
    let security_score = analyzer.calculate_network_security_score(analysis);
    let security_percentage = security_score as u16;

    let gauge_color = if security_score > 80.0 {
        Color::Green
    } else if security_score > 60.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let security_gauge = Gauge::default()
        .block(
            Block::default()
                .title("🔒 Security Score")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(gauge_color))
        .percent(security_percentage)
        .label(format!("{}% Secure", security_percentage));

    f.render_widget(security_gauge, area);
}

fn draw_realtime_insights(
    f: &mut Frame,
    analysis: &crate::analysis::network_analyzer::NetworkAnalysis,
    area: Rect,
) {
    let mut insights = Vec::new();

    insights.push(Line::from(vec![Span::styled(
        "⚡ Real-time Insights",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));
    insights.push(Line::from(""));

    // Network activity insight
    if analysis.total_network_commands > 0 {
        insights.push(Line::from(vec![
            Span::styled("📈 Activity: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{} network commands", analysis.total_network_commands),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    // Security insight
    let secure_ratio = if analysis.unique_endpoints > 0 {
        let secure_count = analysis
            .top_endpoints
            .iter()
            .filter(|e| e.is_secure)
            .count();
        (secure_count as f32 / analysis.unique_endpoints as f32 * 100.0) as u16
    } else {
        100
    };

    insights.push(Line::from(vec![
        Span::styled("🔒 Security: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{}% endpoints secure", secure_ratio),
            Style::default().fg(Color::Green),
        ),
    ]));

    // Protocol insight
    let https_count = analysis.protocol_breakdown.get("https").unwrap_or(&0);
    let total_protocols = analysis.protocol_breakdown.values().sum::<usize>();
    if total_protocols > 0 {
        let https_ratio = (*https_count as f32 / total_protocols as f32 * 100.0) as u16;
        insights.push(Line::from(vec![
            Span::styled("🌐 HTTPS: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{}% of traffic", https_ratio),
                Style::default().fg(Color::Green),
            ),
        ]));
    }

    // Issues insight
    if !analysis.security_issues.is_empty() {
        insights.push(Line::from(vec![
            Span::styled("⚠️  Issues: ", Style::default().fg(Color::Red)),
            Span::styled(
                format!("{} security alerts", analysis.security_issues.len()),
                Style::default().fg(Color::Red),
            ),
        ]));
    }

    let insights_panel = Paragraph::new(insights)
        .block(Block::default().title("💡 Insights").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(insights_panel, area);
}
