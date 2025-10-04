use chrono::{DateTime, Duration, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;

use crate::app::App;
use crate::history::HostType;
use crate::ui::theme::{get_host_icon, Icons, Theme};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with stats
            Constraint::Min(0),    // Main content
        ])
        .split(area);

    // Header with host statistics
    draw_host_header(f, app, chunks[0], &theme);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
        .split(chunks[1]);

    // Left panel: Host list
    draw_hosts_list(f, app, main_chunks[0], &theme);

    // Right panel: Host details
    draw_host_details(f, app, main_chunks[1], &theme);
}

fn draw_host_header(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let host_analysis = analyze_hosts(app);

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

    // Total hosts
    draw_metric_card(
        f,
        chunks[0],
        theme,
        "Total Hosts",
        host_analysis.total_hosts.to_string(),
        theme.style_primary(),
    );

    // Active hosts (used in last 7 days)
    draw_metric_card(
        f,
        chunks[1],
        theme,
        "Active",
        host_analysis.active_hosts.to_string(),
        theme.style_success(),
    );

    // Docker containers
    draw_metric_card(
        f,
        chunks[2],
        theme,
        "Docker",
        host_analysis.docker_hosts.to_string(),
        theme.style_info(),
    );

    // SSH connections
    draw_metric_card(
        f,
        chunks[3],
        theme,
        "SSH",
        host_analysis.ssh_hosts.to_string(),
        theme.style_secondary(),
    );

    // K8s pods
    draw_metric_card(
        f,
        chunks[4],
        theme,
        "K8s",
        host_analysis.k8s_hosts.to_string(),
        theme.style_accent(),
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

fn draw_hosts_list(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let host_analysis = analyze_hosts(app);
    let hosts = &host_analysis.hosts;

    let visible_hosts = hosts
        .iter()
        .skip(app.scroll_offset)
        .take(area.height as usize - 2); // Account for borders

    let host_items: Vec<ListItem> = visible_hosts
        .enumerate()
        .map(|(i, host_info)| {
            let global_index = app.scroll_offset + i;
            let is_selected = global_index == app.selected_index;

            let style = if is_selected {
                theme.style_selected()
            } else {
                theme.style_text()
            };

            let host_icon = get_host_icon(&host_info.host_id);
            let host_type = parse_host_type(&host_info.host_id);

            let status_indicator = if host_info.is_active {
                Span::styled("●", theme.style_success())
            } else {
                Span::styled("○", theme.style_text_dim())
            };

            let security_indicator = if host_info.danger_score > 0.7 {
                Span::styled(format!(" {}", Icons::ERROR), theme.style_danger())
            } else if host_info.danger_score > 0.3 {
                Span::styled(format!(" {}", Icons::WARNING), theme.style_warning())
            } else {
                Span::styled(format!(" {}", Icons::SUCCESS), theme.style_success())
            };

            let performance_bar = create_performance_indicator(host_info.avg_duration_ms, theme);

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{:2}. ", global_index + 1), theme.style_text_dim()),
                    status_indicator,
                    Span::raw(" "),
                    Span::styled(format!("{} ", host_icon), theme.style_accent()),
                    Span::styled(format_host_display(&host_info.host_id, &host_type), style),
                    security_indicator,
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        format!("{} cmds", host_info.total_commands),
                        theme.style_text_dim(),
                    ),
                    Span::raw(" | "),
                    Span::styled(
                        format!("{}ms avg", host_info.avg_duration_ms),
                        theme.style_text_dim(),
                    ),
                    Span::raw(" "),
                    performance_bar,
                ]),
            ])
        })
        .collect();

    let showing_start = app.scroll_offset + 1;
    let showing_end = (app.scroll_offset + host_items.len()).min(hosts.len());

    let hosts_list = List::new(host_items)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::HOSTS), theme.style_accent()),
                    Span::styled("Hosts & Environments", theme.style_title()),
                    Span::styled(
                        format!(" ({}-{} of {})", showing_start, showing_end, hosts.len()),
                        theme.style_text_dim(),
                    ),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(hosts_list, area);
}

fn draw_host_details(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let host_analysis = analyze_hosts(app);

    if host_analysis.hosts.is_empty() {
        draw_empty_state(f, area, theme);
        return;
    }

    let selected_host = if app.selected_index < host_analysis.hosts.len() {
        &host_analysis.hosts[app.selected_index]
    } else {
        &host_analysis.hosts[0]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Host info
            Constraint::Length(10), // Commands chart
            Constraint::Min(0),     // Recent commands
        ])
        .split(area);

    // Host information
    draw_host_info(f, selected_host, chunks[0], theme);

    // Command frequency chart
    draw_command_chart(f, app, selected_host, chunks[1], theme);

    // Recent commands on this host
    draw_recent_commands(f, app, selected_host, chunks[2], theme);
}

fn draw_empty_state(f: &mut Frame, area: Rect, theme: &Theme) {
    let empty_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", Icons::HOSTS), theme.style_text_dim()),
            Span::styled("No hosts detected", theme.style_text_dim()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Hosts will appear here when you:",
            theme.style_text(),
        )]),
        Line::from(vec![Span::styled(
            "• Run commands on remote servers",
            theme.style_text(),
        )]),
        Line::from(vec![Span::styled(
            "• Use Docker containers",
            theme.style_text(),
        )]),
        Line::from(vec![Span::styled(
            "• Connect to Kubernetes pods",
            theme.style_text(),
        )]),
        Line::from(vec![Span::styled(
            "• SSH to remote machines",
            theme.style_text(),
        )]),
    ];

    let paragraph = Paragraph::new(empty_text)
        .block(
            Block::default()
                .title("Host Details")
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .alignment(Alignment::Center)
        .style(theme.style_text());

    f.render_widget(paragraph, area);
}

fn draw_host_info(f: &mut Frame, host_info: &HostInfo, area: Rect, theme: &Theme) {
    let host_type = parse_host_type(&host_info.host_id);
    let (type_name, type_details) = match host_type {
        HostType::Local => (
            "Local Machine",
            "Your local development environment".to_string(),
        ),
        HostType::Ssh { user, host } => ("SSH Connection", format!("{}@{}", user, host)),
        HostType::Docker { container, image } => {
            let img_info = image
                .as_ref()
                .map(|i| format!(" ({})", i))
                .unwrap_or_default();
            ("Docker Container", format!("{}{}", container, img_info))
        }
        HostType::Kubernetes { pod, namespace } => {
            ("Kubernetes Pod", format!("{} in {}", pod, namespace))
        }
    };

    let status = if host_info.is_active {
        "Active"
    } else {
        "Inactive"
    };
    let status_style = if host_info.is_active {
        theme.style_success()
    } else {
        theme.style_text_dim()
    };

    let security_level = if host_info.danger_score > 0.7 {
        ("High Risk", theme.style_danger())
    } else if host_info.danger_score > 0.3 {
        ("Medium Risk", theme.style_warning())
    } else {
        ("Low Risk", theme.style_success())
    };

    let info_text = vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", get_host_icon(&host_info.host_id)),
                theme.style_accent(),
            ),
            Span::styled(type_name, theme.style_title()),
            Span::styled(format!(" - {}", status), status_style),
        ]),
        Line::from(vec![
            Span::styled("Details: ", theme.style_text_dim()),
            Span::styled(type_details, theme.style_text()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Commands: ", theme.style_text_dim()),
            Span::styled(host_info.total_commands.to_string(), theme.style_primary()),
            Span::styled("  Dangerous: ", theme.style_text_dim()),
            Span::styled(
                host_info.dangerous_commands.to_string(),
                theme.style_danger(),
            ),
            Span::styled("  Experiments: ", theme.style_text_dim()),
            Span::styled(
                host_info.experiment_commands.to_string(),
                theme.style_warning(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Avg Duration: ", theme.style_text_dim()),
            Span::styled(
                format!("{}ms", host_info.avg_duration_ms),
                theme.style_info(),
            ),
            Span::styled("  Security: ", theme.style_text_dim()),
            Span::styled(security_level.0, security_level.1),
        ]),
        Line::from(vec![
            Span::styled("Last Seen: ", theme.style_text_dim()),
            Span::styled(format_last_seen(&host_info.last_seen), theme.style_text()),
        ]),
    ];

    let info_paragraph = Paragraph::new(info_text)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::INFO), theme.style_info()),
                    Span::styled("Host Information", theme.style_title()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(info_paragraph, area);
}

fn draw_command_chart(f: &mut Frame, app: &App, host_info: &HostInfo, area: Rect, theme: &Theme) {
    let host_commands: Vec<_> = app
        .commands
        .iter()
        .filter(|cmd| cmd.host_id == host_info.host_id)
        .collect();

    // Get top 5 commands for this host with stable sorting
    let mut command_counts: HashMap<String, usize> = HashMap::new();
    for cmd in &host_commands {
        let short_cmd = cmd
            .command
            .split_whitespace()
            .next()
            .unwrap_or(&cmd.command)
            .to_string();
        *command_counts.entry(short_cmd).or_insert(0) += 1;
    }

    // Create stable sorted vector to prevent flickering
    let mut top_commands: Vec<(String, usize)> = command_counts.into_iter().collect();
    
    // Sort by count (descending) then by name for stability
    top_commands.sort_by(|a, b| {
        match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => a.0.cmp(&b.0), // Secondary sort by name for stability
            other => other,
        }
    });
    top_commands.truncate(5);

    if top_commands.is_empty() {
        let empty_chart = Paragraph::new("No commands found for this host")
            .block(
                Block::default()
                    .title("Command Frequency")
                    .borders(Borders::ALL)
                    .border_style(theme.style_border()),
            )
            .alignment(Alignment::Center)
            .style(theme.style_text_dim());

        f.render_widget(empty_chart, area);
        return;
    }

    let chart_data: Vec<(&str, u64)> = top_commands
        .iter()
        .take(5) // Ensure consistent number of items
        .map(|(cmd, count)| (cmd.as_str(), *count as u64))
        .collect();

    let chart = BarChart::default()
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::CHART), theme.style_accent()),
                    Span::styled("Top Commands", theme.style_title()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .data(&chart_data)
        .bar_width(8)
        .bar_style(theme.style_primary())
        .value_style(theme.style_text());

    f.render_widget(chart, area);
}

fn draw_recent_commands(f: &mut Frame, app: &App, host_info: &HostInfo, area: Rect, theme: &Theme) {
    let mut host_commands: Vec<_> = app
        .commands
        .iter()
        .filter(|cmd| cmd.host_id == host_info.host_id)
        .collect();

    host_commands.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    host_commands.truncate(10);

    let command_items: Vec<ListItem> = host_commands
        .into_iter()
        .enumerate()
        .map(|(i, cmd)| {
            let time_str = cmd.timestamp.format("%m-%d %H:%M:%S").to_string();

            let (status_icon, status_style) = match cmd.exit_code {
                Some(0) => (Icons::SUCCESS, theme.style_success()),
                Some(_) => (Icons::ERROR, theme.style_danger()),
                None => ("", theme.style_text_dim()),
            };

            let danger_indicator = if cmd.is_dangerous {
                Span::styled(format!(" {}", Icons::WARNING), theme.style_danger())
            } else {
                Span::raw("")
            };

            let experiment_indicator = if cmd.is_experiment {
                Span::styled(format!(" {}", Icons::EXPERIMENTS), theme.style_warning())
            } else {
                Span::raw("")
            };

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

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:2}. ", i + 1), theme.style_text_dim()),
                Span::styled(format!("{} ", status_icon), status_style),
                Span::styled(time_str, theme.style_text_dim()),
                Span::raw(" "),
                Span::styled(cmd.command.clone(), theme.style_text()),
                danger_indicator,
                experiment_indicator,
                Span::styled(format!(" [{}]", duration_str), theme.style_text_dim()),
            ]))
        })
        .collect();

    let commands_list = List::new(command_items)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(format!("{} ", Icons::COMMANDS), theme.style_accent()),
                    Span::styled("Recent Commands", theme.style_title()),
                ]))
                .borders(Borders::ALL)
                .border_style(theme.style_border()),
        )
        .style(theme.style_text());

    f.render_widget(commands_list, area);
}

// Helper functions and data structures

#[derive(Debug, Clone)]
struct HostAnalysis {
    total_hosts: usize,
    active_hosts: usize,
    docker_hosts: usize,
    ssh_hosts: usize,
    k8s_hosts: usize,
    hosts: Vec<HostInfo>,
}

#[derive(Debug, Clone)]
struct HostInfo {
    host_id: String,
    total_commands: usize,
    dangerous_commands: usize,
    experiment_commands: usize,
    avg_duration_ms: u64,
    danger_score: f32,
    is_active: bool,
    last_seen: DateTime<Utc>,
}

fn analyze_hosts(app: &App) -> HostAnalysis {
    let mut host_stats: HashMap<String, HostInfo> = HashMap::new();
    let now = Utc::now();
    let week_ago = now - Duration::days(7);

    for cmd in &app.commands {
        let entry = host_stats
            .entry(cmd.host_id.clone())
            .or_insert_with(|| HostInfo {
                host_id: cmd.host_id.clone(),
                total_commands: 0,
                dangerous_commands: 0,
                experiment_commands: 0,
                avg_duration_ms: 0,
                danger_score: 0.0,
                is_active: false,
                last_seen: cmd.timestamp,
            });

        entry.total_commands += 1;
        if cmd.is_dangerous {
            entry.dangerous_commands += 1;
        }
        if cmd.is_experiment {
            entry.experiment_commands += 1;
        }
        if cmd.timestamp > entry.last_seen {
            entry.last_seen = cmd.timestamp;
        }
        if cmd.timestamp > week_ago {
            entry.is_active = true;
        }

        // Update average duration
        if let Some(duration) = cmd.duration {
            entry.avg_duration_ms = (entry.avg_duration_ms + duration) / 2;
        }

        // Update danger score
        entry.danger_score = (entry.danger_score + cmd.danger_score) / 2.0;
    }

    let mut hosts: Vec<_> = host_stats.into_values().collect();
    hosts.sort_by(|a, b| b.total_commands.cmp(&a.total_commands));

    let total_hosts = hosts.len();
    let active_hosts = hosts.iter().filter(|h| h.is_active).count();
    let docker_hosts = hosts
        .iter()
        .filter(|h| h.host_id.starts_with("docker:"))
        .count();
    let ssh_hosts = hosts
        .iter()
        .filter(|h| h.host_id.starts_with("ssh:"))
        .count();
    let k8s_hosts = hosts
        .iter()
        .filter(|h| h.host_id.starts_with("k8s:"))
        .count();

    HostAnalysis {
        total_hosts,
        active_hosts,
        docker_hosts,
        ssh_hosts,
        k8s_hosts,
        hosts,
    }
}

fn parse_host_type(host_id: &str) -> HostType {
    if host_id == "local" {
        HostType::Local
    } else if let Some(ssh_part) = host_id.strip_prefix("ssh:") {
        let parts: Vec<&str> = ssh_part.split('@').collect();
        if parts.len() == 2 {
            HostType::Ssh {
                user: parts[0].to_string(),
                host: parts[1].to_string(),
            }
        } else {
            HostType::Ssh {
                user: "unknown".to_string(),
                host: ssh_part.to_string(),
            }
        }
    } else if let Some(docker_part) = host_id.strip_prefix("docker:") {
        let parts: Vec<&str> = docker_part.split(':').collect();
        HostType::Docker {
            container: parts[0].to_string(),
            image: if parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                None
            },
        }
    } else if let Some(k8s_part) = host_id.strip_prefix("k8s:") {
        let parts: Vec<&str> = k8s_part.split('/').collect();
        if parts.len() == 2 {
            HostType::Kubernetes {
                namespace: parts[0].to_string(),
                pod: parts[1].to_string(),
            }
        } else {
            HostType::Kubernetes {
                namespace: "default".to_string(),
                pod: k8s_part.to_string(),
            }
        }
    } else {
        HostType::Local
    }
}

fn format_host_display(_host_id: &str, host_type: &HostType) -> String {
    match host_type {
        HostType::Local => "Local Machine".to_string(),
        HostType::Ssh { user, host } => format!("{}@{}", user, host),
        HostType::Docker {
            container,
            image: _,
        } => container.clone(),
        HostType::Kubernetes { pod, namespace } => format!("{}/{}", namespace, pod),
    }
}

fn format_last_seen(timestamp: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(*timestamp);

    if diff.num_days() > 0 {
        format!("{} days ago", diff.num_days())
    } else if diff.num_hours() > 0 {
        format!("{} hours ago", diff.num_hours())
    } else if diff.num_minutes() > 0 {
        format!("{} minutes ago", diff.num_minutes())
    } else {
        "Just now".to_string()
    }
}

fn create_performance_indicator(avg_duration_ms: u64, theme: &Theme) -> Span<'_> {
    let indicator = if avg_duration_ms < 100 {
        "▰▰▰▱▱" // Fast
    } else if avg_duration_ms < 500 {
        "▰▰▱▱▱" // Medium
    } else if avg_duration_ms < 1000 {
        "▰▱▱▱▱" // Slow
    } else {
        "▱▱▱▱▱" // Very slow
    };

    let style = if avg_duration_ms < 100 {
        theme.style_success()
    } else if avg_duration_ms < 500 {
        theme.style_warning()
    } else {
        theme.style_danger()
    };

    Span::styled(indicator, style)
}
