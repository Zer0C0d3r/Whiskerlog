use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;

use crate::analysis::package_tracker::PackageTracker;
use crate::app::App;
use crate::ui::theme::get_manager_info;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum PackageFilter {
    All,
    Linux,
    Programming,
    Container,
    System,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum PackageSortMode {
    Usage,
    Trends,
    Health,
    Recent,
}

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(5), // Header with package ecosystem metrics
                Constraint::Length(3), // Interactive controls
                Constraint::Min(10),   // Main content area
                Constraint::Length(8), // Analytics and insights footer
            ]
            .as_ref(),
        )
        .split(area);

    // Compute analysis once for efficiency
    let tracker = PackageTracker::new();
    let analysis = tracker.analyze_package_usage(&app.commands);

    // Apply filtering based on current filter mode
    let filtered_analysis = apply_package_filter(&analysis, &PackageFilter::All);

    // Enhanced header with package ecosystem metrics
    draw_package_metrics(f, app, &filtered_analysis, chunks[0]);

    // Interactive controls
    draw_package_controls(f, app, chunks[1]);

    // Main content with managers, packages, and trends
    draw_package_content(f, app, &filtered_analysis, chunks[2]);

    // Analytics and insights footer
    draw_package_analytics(f, app, &filtered_analysis, &tracker, chunks[3]);
}

fn draw_package_metrics(
    f: &mut Frame,
    _app: &App,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
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

    // Total Operations Metric
    let operations_count = analysis.total_package_operations;
    let operations_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "📦 OPERATIONS",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{}", operations_count),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" total", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            "Package actions",
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(operations_block, metric_chunks[0]);

    // Package Managers Metric
    let managers_count = analysis.managers_used.len();
    let managers_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "🛠️  MANAGERS",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{}", managers_count),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" active", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            "Ecosystems",
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(managers_block, metric_chunks[1]);

    // Package Health Metric
    let tracker = PackageTracker::new();
    let health_score = tracker.calculate_package_health_score(analysis);
    let health_color = if health_score > 80.0 {
        Color::Green
    } else if health_score > 60.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    let health_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "💊 HEALTH",
            Style::default()
                .fg(health_color)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{:.0}%", health_score),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" score", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            "Ecosystem health",
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(health_block, metric_chunks[2]);

    // Security & Issues Metric
    let trends_count = analysis.package_trends.len();
    let conflicts_count = analysis.version_conflicts.len();
    let security_issues = count_security_issues(analysis);
    let total_issues = trends_count + conflicts_count + security_issues;
    let issues_color = if total_issues == 0 {
        Color::Green
    } else if total_issues < 3 {
        Color::Yellow
    } else {
        Color::Red
    };
    let issues_block = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "🔒 SECURITY",
            Style::default()
                .fg(issues_color)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{}", total_issues),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" issues", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![Span::styled(
            "Security & conflicts",
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(issues_block, metric_chunks[3]);
}

fn draw_package_controls(f: &mut Frame, _app: &App, area: Rect) {
    let current_filter = PackageFilter::All; // Would be stored in app state
    let current_sort = PackageSortMode::Usage; // Would be stored in app state

    let filter_indicator = match current_filter {
        PackageFilter::All => ("🌐", "All Ecosystems", Color::White),
        PackageFilter::Linux => ("🐧", "Linux Packages", Color::Blue),
        PackageFilter::Programming => ("💻", "Programming", Color::Green),
        PackageFilter::Container => ("🐳", "Containers", Color::Cyan),
        PackageFilter::System => ("⚙️", "System Tools", Color::Yellow),
    };

    let sort_indicator = match current_sort {
        PackageSortMode::Usage => ("📊", "Usage", Color::Green),
        PackageSortMode::Trends => ("📈", "Trends", Color::Yellow),
        PackageSortMode::Health => ("💊", "Health", Color::Blue),
        PackageSortMode::Recent => ("🕒", "Recent", Color::Magenta),
    };

    let controls_text = vec![Line::from(vec![
        Span::styled("Active Filter: ", Style::default().fg(Color::Cyan)),
        Span::styled(filter_indicator.0, Style::default().fg(filter_indicator.2)),
        Span::raw(" "),
        Span::styled(
            filter_indicator.1,
            Style::default()
                .fg(filter_indicator.2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  "),
        Span::styled("Sort: ", Style::default().fg(Color::Cyan)),
        Span::styled(sort_indicator.0, Style::default().fg(sort_indicator.2)),
        Span::raw(" "),
        Span::styled(
            sort_indicator.1,
            Style::default()
                .fg(sort_indicator.2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  "),
        Span::styled("Controls: ", Style::default().fg(Color::Gray)),
        Span::styled("[L]", Style::default().fg(Color::Blue)),
        Span::styled("inux ", Style::default().fg(Color::White)),
        Span::styled("[P]", Style::default().fg(Color::Green)),
        Span::styled("rog ", Style::default().fg(Color::White)),
        Span::styled("[C]", Style::default().fg(Color::Cyan)),
        Span::styled("ontainer ", Style::default().fg(Color::White)),
        Span::styled("[S]", Style::default().fg(Color::Yellow)),
        Span::styled("ystem", Style::default().fg(Color::White)),
    ])];

    let controls = Paragraph::new(controls_text)
        .block(
            Block::default()
                .title("🎛️  Package Controls & Filters")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(controls, area);
}

fn draw_package_content(
    f: &mut Frame,
    _app: &App,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
    // Split into three panels: managers, packages, and trends
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(35), // Package managers
                Constraint::Percentage(35), // Top packages
                Constraint::Percentage(30), // Trends and issues
            ]
            .as_ref(),
        )
        .split(area);

    // Left panel: Enhanced package managers
    draw_enhanced_managers_list(f, _app, analysis, content_chunks[0]);

    // Middle panel: Top packages with details
    draw_enhanced_packages_list(f, _app, analysis, content_chunks[1]);

    // Right panel: Trends and version conflicts
    draw_trends_and_conflicts(f, _app, analysis, content_chunks[2]);
}

fn draw_enhanced_managers_list(
    f: &mut Frame,
    _app: &App,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
    let mut items = Vec::new();

    for (i, manager_stats) in analysis.managers_used.iter().enumerate() {
        let is_selected = i == 0; // Remove selection logic for now

        // Enhanced manager icons with ecosystem classification
        let (manager_icon, ecosystem_color) = get_manager_info(&manager_stats.manager);

        // Calculate activity score
        let total_activity = manager_stats.total_operations;
        let activity_level = if total_activity > 50 {
            "🔥"
        } else if total_activity > 20 {
            "⚡"
        } else {
            "💡"
        };

        let item_style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        items.push(
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(manager_icon, Style::default().fg(ecosystem_color)),
                    Span::raw(" "),
                    Span::styled(
                        &manager_stats.manager,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(activity_level, Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        format!("↗{}", manager_stats.installs),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("↘{}", manager_stats.removes),
                        Style::default().fg(Color::Red),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("↻{}", manager_stats.updates),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" • "),
                    Span::styled(
                        format!("{} pkgs", manager_stats.top_packages.len()),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
            ])
            .style(item_style),
        );

        if i >= 9 {
            break;
        } // Limit to 10 managers for display
    }

    if items.is_empty() {
        items.push(ListItem::new(vec![
            Line::from(vec![Span::styled(
                "📦 No package managers detected",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![Span::styled(
                "   Run package installation commands",
                Style::default().fg(Color::Gray),
            )]),
        ]));
    }

    let managers_list = List::new(items)
        .block(
            Block::default()
                .title("🛠️  Package Managers")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(managers_list, area);
}

fn draw_enhanced_packages_list(
    f: &mut Frame,
    _app: &App,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
    let mut items = Vec::new();

    // Collect all packages from all managers
    let mut all_packages = Vec::new();
    for manager_stats in &analysis.managers_used {
        for package_stats in &manager_stats.top_packages {
            all_packages.push((manager_stats.manager.clone(), package_stats));
        }
    }

    // Sort by total usage (install + remove counts)
    all_packages.sort_by(|a, b| {
        let usage_a = a.1.install_count + a.1.remove_count;
        let usage_b = b.1.install_count + b.1.remove_count;
        usage_b.cmp(&usage_a)
    });

    for (i, (manager, package_stats)) in all_packages.iter().enumerate() {
        let is_selected = i == 0; // Remove selection logic for now

        // Get manager info for styling
        let (manager_icon, ecosystem_color) = get_manager_info(manager);

        // Calculate package popularity
        let total_usage = package_stats.install_count + package_stats.remove_count;
        let popularity = if total_usage > 10 {
            "🌟"
        } else if total_usage > 5 {
            "⭐"
        } else {
            "💫"
        };

        // Version stability indicator
        let stability = if package_stats.versions_seen.len() > 3 {
            ("🔄", Color::Red) // Version churn
        } else if package_stats.versions_seen.len() > 1 {
            ("📈", Color::Yellow) // Some versions
        } else {
            ("✅", Color::Green) // Stable
        };

        let item_style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        items.push(
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(manager_icon, Style::default().fg(ecosystem_color)),
                    Span::raw(" "),
                    Span::styled(
                        &package_stats.name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(popularity, Style::default().fg(Color::Yellow)),
                    Span::raw(" "),
                    Span::styled(stability.0, Style::default().fg(stability.1)),
                ]),
                Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        format!("[{}]", manager),
                        Style::default().fg(ecosystem_color),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{}× used", total_usage),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::raw(" • "),
                    Span::styled(
                        format!("{} vers", package_stats.versions_seen.len()),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
            ])
            .style(item_style),
        );

        if i >= 9 {
            break;
        } // Limit to 10 packages for display
    }

    if items.is_empty() {
        items.push(ListItem::new(vec![
            Line::from(vec![Span::styled(
                "📦 No packages detected",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![Span::styled(
                "   Install packages to see analysis",
                Style::default().fg(Color::Gray),
            )]),
        ]));
    }

    let packages_list = List::new(items)
        .block(
            Block::default()
                .title("📦 Top Packages")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(packages_list, area);
}

fn draw_trends_and_conflicts(
    f: &mut Frame,
    _app: &App,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
    // Split into trends and conflicts sections
    let trend_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(area);

    // Top: Package trends
    draw_package_trends(f, analysis, trend_chunks[0]);

    // Bottom: Version conflicts
    draw_version_conflicts(f, analysis, trend_chunks[1]);
}

fn draw_package_trends(
    f: &mut Frame,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
    let mut items = Vec::new();

    for (i, trend) in analysis.package_trends.iter().enumerate() {
        let (trend_icon, trend_color) = match trend.trend_type {
            crate::analysis::package_tracker::TrendType::FrequentInstalls => ("📈", Color::Green),
            crate::analysis::package_tracker::TrendType::RepeatedInstalls => ("🔄", Color::Yellow),
            crate::analysis::package_tracker::TrendType::QuickRemoval => ("⚡", Color::Red),
            crate::analysis::package_tracker::TrendType::VersionChurn => ("🌀", Color::Magenta),
        };

        let trend_description = match trend.trend_type {
            crate::analysis::package_tracker::TrendType::FrequentInstalls => "Frequently installed",
            crate::analysis::package_tracker::TrendType::RepeatedInstalls => "Repeatedly installed",
            crate::analysis::package_tracker::TrendType::QuickRemoval => "Quickly removed",
            crate::analysis::package_tracker::TrendType::VersionChurn => "Version instability",
        };

        // Add time span information for better context
        let time_info = if trend.time_span_days > 0 {
            format!(" over {} days", trend.time_span_days)
        } else {
            " recently".to_string()
        };

        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled(trend_icon, Style::default().fg(trend_color)),
                Span::raw(" "),
                Span::styled(
                    trend.package.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", trend.manager),
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(vec![
                Span::raw("   "),
                Span::styled(trend_description, Style::default().fg(Color::Gray)),
                Span::raw(" • "),
                Span::styled(
                    format!("{}×", trend.frequency),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(time_info, Style::default().fg(Color::Yellow)),
            ]),
        ]));

        if i >= 4 {
            break;
        } // Limit to 5 trends for display
    }

    if items.is_empty() {
        items.push(ListItem::new(vec![Line::from(vec![Span::styled(
            "📊 No trends detected",
            Style::default().fg(Color::Yellow),
        )])]));
    }

    let trends_list = List::new(items)
        .block(
            Block::default()
                .title("📊 Package Trends")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(trends_list, area);
}

fn draw_version_conflicts(
    f: &mut Frame,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
    let mut items = Vec::new();

    for (i, conflict) in analysis.version_conflicts.iter().enumerate() {
        let (conflict_icon, conflict_color) = match conflict.conflict_type {
            crate::analysis::package_tracker::ConflictType::DowngradeDetected => ("⬇️", Color::Red),
            crate::analysis::package_tracker::ConflictType::MultipleVersions => {
                ("🔀", Color::Yellow)
            }
            crate::analysis::package_tracker::ConflictType::InconsistentVersioning => {
                ("⚠️", Color::Magenta)
            }
        };

        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled(conflict_icon, Style::default().fg(conflict_color)),
                Span::raw(" "),
                Span::styled(
                    &conflict.package,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("   "),
                Span::styled(&conflict.recommendation, Style::default().fg(Color::Gray)),
            ]),
        ]));

        if i >= 2 {
            break;
        } // Limit to 3 conflicts for display
    }

    if items.is_empty() {
        items.push(ListItem::new(vec![Line::from(vec![Span::styled(
            "✅ No conflicts detected",
            Style::default().fg(Color::Green),
        )])]));
    }

    let conflicts_list = List::new(items)
        .block(
            Block::default()
                .title("⚠️  Version Conflicts")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(conflicts_list, area);
}

fn draw_package_analytics(
    f: &mut Frame,
    _app: &App,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    tracker: &PackageTracker,
    area: Rect,
) {
    let analytics_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(35), // Manager distribution chart
                Constraint::Percentage(25), // Health gauge
                Constraint::Percentage(40), // Ecosystem health & recommendations
            ]
            .as_ref(),
        )
        .split(area);

    // Left: Manager distribution chart
    draw_manager_distribution_chart(f, analysis, analytics_chunks[0]);

    // Middle: Package health gauge
    draw_package_health_gauge(f, analysis, tracker, analytics_chunks[1]);

    // Right: Ecosystem health and recommendations
    draw_ecosystem_health_and_recommendations(f, analysis, analytics_chunks[2]);
}

fn draw_manager_distribution_chart(
    f: &mut Frame,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
    let mut chart_data = Vec::new();

    for manager_stats in &analysis.managers_used {
        if manager_stats.total_operations > 0 {
            let manager_name = if manager_stats.manager.len() > 8 {
                &manager_stats.manager[..8]
            } else {
                &manager_stats.manager
            };
            chart_data.push((manager_name, manager_stats.total_operations as u64));
        }
    }

    // Sort by operations and take top 6
    chart_data.sort_by(|a, b| b.1.cmp(&a.1));
    chart_data.truncate(6);

    if !chart_data.is_empty() {
        let bar_chart = BarChart::default()
            .block(
                Block::default()
                    .title("📊 Manager Usage")
                    .borders(Borders::ALL),
            )
            .data(&chart_data)
            .bar_width(6)
            .bar_style(Style::default().fg(Color::Green))
            .value_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(bar_chart, area);
    } else {
        let no_data = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "📊 Manager Usage",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "No package data",
                Style::default().fg(Color::Gray),
            )]),
        ])
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

        f.render_widget(no_data, area);
    }
}

fn draw_package_health_gauge(
    f: &mut Frame,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    tracker: &PackageTracker,
    area: Rect,
) {
    let health_score = tracker.calculate_package_health_score(analysis);
    let health_percentage = health_score as u16;

    let gauge_color = if health_score > 80.0 {
        Color::Green
    } else if health_score > 60.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let health_gauge = Gauge::default()
        .block(
            Block::default()
                .title("💊 Ecosystem Health")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(gauge_color))
        .percent(health_percentage)
        .label(format!("{}% Healthy", health_percentage));

    f.render_widget(health_gauge, area);
}

fn draw_ecosystem_health_and_recommendations(
    f: &mut Frame,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
    // Split into ecosystem health and recommendations
    let health_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(area);

    // Top: Ecosystem health breakdown
    draw_ecosystem_health_breakdown(f, analysis, health_chunks[0]);

    // Bottom: Recommendations
    draw_package_recommendations(f, analysis, health_chunks[1]);
}

fn draw_ecosystem_health_breakdown(
    f: &mut Frame,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
    let ecosystem_health = calculate_ecosystem_health(analysis);
    let mut items = Vec::new();

    items.push(ListItem::new(vec![Line::from(vec![Span::styled(
        "🌐 Ecosystem Health",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )])]));

    for (ecosystem, health) in ecosystem_health.iter() {
        let (icon, color) = match ecosystem.as_str() {
            "Linux" => ("🐧", Color::Blue),
            "Programming" => ("💻", Color::Green),
            "Container" => ("🐳", Color::Cyan),
            "System" => ("⚙️", Color::Yellow),
            "macOS" => ("🍎", Color::White),
            _ => ("📦", Color::Gray),
        };

        let health_color = if *health > 80.0 {
            Color::Green
        } else if *health > 60.0 {
            Color::Yellow
        } else {
            Color::Red
        };
        let health_bar = "█".repeat((*health / 10.0) as usize);

        items.push(ListItem::new(vec![Line::from(vec![
            Span::styled(icon, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(ecosystem, Style::default().fg(Color::White)),
            Span::raw(" "),
            Span::styled(
                format!("{:.0}%", health),
                Style::default()
                    .fg(health_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(health_bar, Style::default().fg(health_color)),
        ])]));
    }

    if ecosystem_health.is_empty() {
        items.push(ListItem::new(vec![Line::from(vec![Span::styled(
            "No ecosystem data available",
            Style::default().fg(Color::Gray),
        )])]));
    }

    let health_list = List::new(items)
        .block(
            Block::default()
                .title("🌐 Ecosystem Health")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(health_list, area);
}

fn draw_package_recommendations(
    f: &mut Frame,
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    area: Rect,
) {
    let mut recommendations = Vec::new();

    recommendations.push(Line::from(vec![Span::styled(
        "💡 Recommendations",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));
    recommendations.push(Line::from(""));

    // Show top 5 recommendations
    for (i, recommendation) in analysis.recommendations.iter().enumerate() {
        if i >= 5 {
            break;
        }

        let icon = match i {
            0 => "🔥",
            1 => "⚡",
            2 => "💡",
            3 => "📋",
            _ => "•",
        };

        recommendations.push(Line::from(vec![
            Span::styled(icon, Style::default().fg(Color::Yellow)),
            Span::raw(" "),
            Span::styled(recommendation, Style::default().fg(Color::White)),
        ]));
    }

    if analysis.recommendations.is_empty() {
        recommendations.push(Line::from(vec![Span::styled(
            "✅ No issues detected",
            Style::default().fg(Color::Green),
        )]));
        recommendations.push(Line::from(vec![Span::styled(
            "   Your package management looks healthy",
            Style::default().fg(Color::Gray),
        )]));
    }

    let recommendations_panel = Paragraph::new(recommendations)
        .block(
            Block::default()
                .title("💡 Recommendations")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(recommendations_panel, area);
}

// Helper functions

fn apply_package_filter(
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
    _filter: &PackageFilter,
) -> crate::analysis::package_tracker::PackageAnalysis {
    // For now, return the analysis as-is
    // In a full implementation, this would filter based on ecosystem
    analysis.clone()
}

fn count_security_issues(analysis: &crate::analysis::package_tracker::PackageAnalysis) -> usize {
    // Count security-related issues from trends and conflicts
    let mut security_issues = 0;

    for trend in &analysis.package_trends {
        match trend.trend_type {
            crate::analysis::package_tracker::TrendType::QuickRemoval => security_issues += 1,
            crate::analysis::package_tracker::TrendType::RepeatedInstalls => security_issues += 1,
            _ => {}
        }
    }

    // All version conflicts are considered security issues
    security_issues += analysis.version_conflicts.len();

    security_issues
}

fn calculate_ecosystem_health(
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
) -> HashMap<String, f32> {
    let mut ecosystem_health = HashMap::new();

    for manager_stats in &analysis.managers_used {
        let ecosystem = categorize_manager(&manager_stats.manager);
        let health = calculate_manager_health(manager_stats);

        // Average health scores for ecosystems with multiple managers
        let current_health = ecosystem_health.get(&ecosystem).unwrap_or(&0.0);
        let new_health = (current_health + health) / 2.0;
        ecosystem_health.insert(ecosystem, new_health);
    }

    ecosystem_health
}

fn categorize_manager(manager: &str) -> String {
    match manager {
        "apt" | "apt-get" | "yum" | "dnf" | "pacman" | "zypper" => "Linux".to_string(),
        "npm" | "yarn" | "pnpm" | "pip" | "pip3" | "cargo" | "gem" | "go" => {
            "Programming".to_string()
        }
        "docker" | "podman" | "kubectl" => "Container".to_string(),
        "brew" | "port" | "choco" | "scoop" => "System".to_string(),
        _ => "Other".to_string(),
    }
}

fn calculate_manager_health(manager_stats: &crate::analysis::package_tracker::ManagerStats) -> f32 {
    let mut health: f32 = 100.0;

    // Penalize high removal rate
    if manager_stats.total_operations > 0 {
        let removal_rate = manager_stats.removes as f32 / manager_stats.total_operations as f32;
        if removal_rate > 0.5 {
            health -= 30.0; // High removal rate is concerning
        } else if removal_rate > 0.3 {
            health -= 15.0;
        }
    }

    // Bonus for update activity
    if manager_stats.updates > 0 {
        health += 10.0; // Regular updates are good
    }

    // Penalize if no packages tracked
    if manager_stats.top_packages.is_empty() {
        health -= 20.0;
    }

    health.clamp(0.0, 100.0)
}

#[allow(dead_code)]
fn generate_security_insights(
    analysis: &crate::analysis::package_tracker::PackageAnalysis,
) -> Vec<String> {
    let mut insights = Vec::new();

    // Check for potentially risky package patterns
    for manager_stats in &analysis.managers_used {
        // High removal rate might indicate problematic packages
        if manager_stats.removes > manager_stats.installs / 3 && manager_stats.removes > 2 {
            insights.push(format!(
                "High removal rate in {} - review package quality",
                manager_stats.manager
            ));
        }

        // Many different packages might indicate dependency sprawl
        if manager_stats.top_packages.len() > 20 {
            insights.push(format!(
                "Large number of {} packages - consider dependency audit",
                manager_stats.manager
            ));
        }
    }

    // Check for version conflicts that might indicate security issues
    for conflict in &analysis.version_conflicts {
        if matches!(
            conflict.conflict_type,
            crate::analysis::package_tracker::ConflictType::DowngradeDetected
        ) {
            insights.push(format!(
                "Version downgrade detected for {} - security risk",
                conflict.package
            ));
        }
    }

    // Check for rapid install/remove cycles
    let quick_removals = analysis
        .package_trends
        .iter()
        .filter(|t| {
            matches!(
                t.trend_type,
                crate::analysis::package_tracker::TrendType::QuickRemoval
            )
        })
        .count();

    if quick_removals > 2 {
        insights.push("Multiple quick removals detected - review package vetting".to_string());
    }

    insights
}
