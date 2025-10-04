use anyhow::Result;

use crate::analysis::stats::{CommandStats, ProductivityStats, SessionStats, StatsAnalyzer};
use crate::config::Config;
use crate::db::Database;
use crate::history::{Command, HistoryParser};

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Summary,
    Commands,
    Sessions,
    Search,
    Hosts,
    Heatmap,
    Aliases,
    Dangerous,
    Network,
    Packages,
    Experiments,
}

impl Tab {
    pub fn all() -> Vec<Tab> {
        vec![
            Tab::Summary,
            Tab::Commands,
            Tab::Sessions,
            Tab::Search,
            Tab::Hosts,
            Tab::Heatmap,
            Tab::Aliases,
            Tab::Dangerous,
            Tab::Network,
            Tab::Packages,
            Tab::Experiments,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Summary => "Summary",
            Tab::Commands => "Commands",
            Tab::Sessions => "Sessions",
            Tab::Search => "Search",
            Tab::Hosts => "Hosts",
            Tab::Heatmap => "Heatmap",
            Tab::Aliases => "Aliases",
            Tab::Dangerous => "Dangerous",
            Tab::Network => "Network",
            Tab::Packages => "Packages",
            Tab::Experiments => "Experiments",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Time,
    Count,
    Host,
    Danger,
    Success,
    Length,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterBy {
    All,
    Failed,
    Experiments,
    Recent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchFilter {
    None,
    Failed,
    Dangerous,
    Recent,
    Experiments,
}

pub struct App {
    #[allow(dead_code)]
    pub config: Config,
    #[allow(dead_code)]
    pub db: Database,
    pub current_tab: Tab,
    pub tab_index: usize,
    pub commands: Vec<Command>,
    pub filtered_commands: Vec<Command>,
    pub search_mode: bool,
    pub search_query: String,
    pub search_filter: SearchFilter,
    pub help_visible: bool,
    pub scroll_offset: usize,
    pub selected_index: usize,
    pub stats: AppStats,
    pub sort_by: SortBy,
    pub filter_by: FilterBy,
    // Heatmap state
    pub heatmap_time_range: crate::analysis::heatmap::TimeRange,
    pub heatmap_view_mode: crate::analysis::heatmap::ViewMode,
    // Enhanced analytics
    pub command_stats: Option<CommandStats>,
    pub session_stats: Option<SessionStats>,
    pub productivity_stats: Option<ProductivityStats>,
    // Performance optimization
    pub last_analysis_update: std::time::Instant,
    pub analysis_cache_valid: bool,
}

#[derive(Debug, Default)]
pub struct AppStats {
    pub total_commands: usize,
    #[allow(dead_code)]
    pub unique_commands: usize,
    pub total_sessions: usize,
    pub hosts_count: usize,
    pub dangerous_commands: usize,
    pub network_endpoints: usize,
    pub packages_used: usize,
    pub experiment_sessions: usize,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = Config::load_or_create()?;

        let mut db = Database::new(&config.database_path).await?;

        // Parse and import history on first run
        let parser = HistoryParser::new();
        let commands = parser.parse_all_histories().await?;

        // Store commands in database
        for command in &commands {
            db.insert_command(command).await?;
        }

        let stats = Self::calculate_stats(&commands);

        let filtered_commands = commands.clone();

        // Initialize enhanced analytics
        let analyzer = StatsAnalyzer::new();
        let command_stats = Some(analyzer.analyze_commands(&commands));
        let session_stats = Some(analyzer.analyze_sessions(&commands));
        let productivity_stats = Some(analyzer.analyze_productivity(&commands));

        Ok(Self {
            config,
            db,
            current_tab: Tab::Summary,
            tab_index: 0,
            commands,
            filtered_commands,
            search_mode: false,
            search_query: String::new(),
            search_filter: SearchFilter::None,
            help_visible: false,
            scroll_offset: 0,
            selected_index: 0,
            stats,
            sort_by: SortBy::Time,
            filter_by: FilterBy::All,
            // Initialize heatmap state
            heatmap_time_range: crate::analysis::heatmap::TimeRange::Week,
            heatmap_view_mode: crate::analysis::heatmap::ViewMode::All,
            // Enhanced analytics
            command_stats,
            session_stats,
            productivity_stats,
            // Performance optimization
            last_analysis_update: std::time::Instant::now(),
            analysis_cache_valid: true,
        })
    }

    fn calculate_stats(commands: &[Command]) -> AppStats {
        let mut unique_commands = std::collections::HashSet::new();
        let mut hosts = std::collections::HashSet::new();
        let mut network_endpoints = std::collections::HashSet::new();
        let mut packages = std::collections::HashSet::new();
        let mut sessions = std::collections::HashSet::new();

        let mut dangerous_count = 0;
        let mut experiment_count = 0;

        for cmd in commands {
            unique_commands.insert(&cmd.command);
            hosts.insert(&cmd.host_id);
            sessions.insert(&cmd.session_id);

            for endpoint in &cmd.network_endpoints {
                network_endpoints.insert(endpoint);
            }

            for package in &cmd.packages_used {
                packages.insert((&package.manager, &package.name));
            }

            if cmd.is_dangerous {
                dangerous_count += 1;
            }

            if cmd.is_experiment {
                experiment_count += 1;
            }
        }

        AppStats {
            total_commands: commands.len(),
            unique_commands: unique_commands.len(),
            total_sessions: sessions.len(),
            hosts_count: hosts.len(),
            dangerous_commands: dangerous_count,
            network_endpoints: network_endpoints.len(),
            packages_used: packages.len(),
            experiment_sessions: experiment_count,
        }
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % Tab::all().len();
        self.current_tab = Tab::all()[self.tab_index].clone();
        self.reset_navigation();
    }

    pub fn previous_tab(&mut self) {
        if self.tab_index == 0 {
            self.tab_index = Tab::all().len() - 1;
        } else {
            self.tab_index -= 1;
        }
        self.current_tab = Tab::all()[self.tab_index].clone();
        self.reset_navigation();
    }

    pub fn go_to_search_tab(&mut self) {
        // Jump to Search tab instead of entering search mode
        self.tab_index = 3; // Search is the 4th tab (index 3)
        self.current_tab = Tab::Search;
        self.reset_navigation();
    }

    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }

    pub fn scroll_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        // Update scroll offset to keep selection visible
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    pub fn scroll_down(&mut self) {
        let max_items = self.get_current_items_count();
        if max_items > 0 && self.selected_index < max_items - 1 {
            self.selected_index += 1;
        }
        // Update scroll offset to keep selection visible (assuming 20 visible items)
        let visible_items = 20;
        if self.selected_index >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected_index - visible_items + 1;
        }
    }

    pub fn scroll_left(&mut self) {
        // Tab-specific left navigation - for now, just move selection up
        self.scroll_up();
    }

    pub fn scroll_right(&mut self) {
        // Tab-specific right navigation - for now, just move selection down
        self.scroll_down();
    }

    pub fn handle_enter(&mut self) {
        if self.search_mode {
            self.execute_search();
        } else {
            // Tab-specific enter action
            match self.current_tab {
                Tab::Commands => {
                    // Execute selected command or show details
                }
                Tab::Sessions => {
                    // Jump to selected session
                }
                Tab::Search => {
                    // Execute search or show command details
                }
                Tab::Hosts => {
                    // Connect to selected host or show host details
                }
                Tab::Dangerous => {
                    // Show command details or safety information
                }
                _ => {}
            }
        }
    }

    pub fn handle_escape(&mut self) {
        if self.help_visible {
            self.help_visible = false;
        } else if self.current_tab == Tab::Search
            && (!self.search_query.is_empty() || self.search_mode)
        {
            self.search_mode = false;
            self.search_query.clear();
            self.reset_navigation();
        }
    }

    pub fn handle_char(&mut self, c: char) {
        match self.current_tab {
            Tab::Search => {
                // In Search tab, always add characters to search query
                self.search_query.push(c);
                self.search_mode = true;
                self.reset_navigation(); // Reset selection when search changes
            }
            Tab::Commands => {
                // Handle sorting and filtering keys for Commands tab
                match c.to_ascii_uppercase() {
                    // Sorting keys
                    'T' => self.set_sort_by(SortBy::Time),
                    'C' => self.set_sort_by(SortBy::Count),
                    'H' => self.set_sort_by(SortBy::Host),
                    'D' => self.set_sort_by(SortBy::Danger),
                    'S' => self.set_sort_by(SortBy::Success),
                    'L' => self.set_sort_by(SortBy::Length),
                    // Filtering keys
                    'F' => self.set_filter_by(FilterBy::Failed),
                    'E' => self.set_filter_by(FilterBy::Experiments),
                    'R' => self.set_filter_by(FilterBy::Recent),
                    'A' => self.set_filter_by(FilterBy::All),
                    _ => {}
                }
            }
            Tab::Heatmap => {
                // Handle heatmap time range and view mode keys
                self.handle_heatmap_key(c);
            }
            Tab::Aliases => {
                // Handle alias-specific keys
                self.handle_alias_key(c);
            }
            Tab::Network => {
                // Handle network-specific keys
                self.handle_network_key(c);
            }
            _ => {
                // For other tabs, ignore character input
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.current_tab == Tab::Search {
            self.search_query.pop();
            if self.search_query.is_empty() {
                self.search_mode = false;
            }
            self.reset_navigation(); // Reset selection when search changes
        }
    }

    fn reset_navigation(&mut self) {
        self.scroll_offset = 0;
        self.selected_index = 0;
    }

    fn get_current_items_count(&self) -> usize {
        match self.current_tab {
            Tab::Commands => self.filtered_commands.len(),
            Tab::Sessions => self.stats.total_sessions,
            Tab::Hosts => self.get_hosts_count(),
            Tab::Dangerous => self.stats.dangerous_commands,
            Tab::Network => self.stats.network_endpoints,
            Tab::Packages => self.stats.packages_used,
            Tab::Experiments => self.stats.experiment_sessions,
            _ => 10, // Default for other tabs
        }
    }

    fn get_hosts_count(&self) -> usize {
        let mut hosts = std::collections::HashSet::new();
        for cmd in &self.commands {
            hosts.insert(&cmd.host_id);
        }
        hosts.len()
    }

    fn execute_search(&mut self) {
        // Implement fuzzy search logic
        self.search_mode = false;
    }

    pub fn scroll_to_top(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        let max_items = self.get_current_items_count();
        self.selected_index = max_items.saturating_sub(1);
    }

    pub fn page_up(&mut self) {
        let page_size = 10;
        if self.selected_index >= page_size {
            self.selected_index -= page_size;
        } else {
            self.selected_index = 0;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    pub fn page_down(&mut self) {
        let page_size = 10;
        let max_items = self.get_current_items_count();
        if self.selected_index + page_size < max_items {
            self.selected_index += page_size;
        } else {
            self.selected_index = max_items.saturating_sub(1);
        }
    }

    pub fn jump_to_tab(&mut self, index: usize) {
        let tabs = Tab::all();
        if index < tabs.len() {
            self.tab_index = index;
            self.current_tab = tabs[index].clone();
            self.reset_navigation();
        }
    }

    pub fn set_sort_by(&mut self, sort_by: SortBy) {
        self.sort_by = sort_by;
        self.apply_filters_and_sort();
        self.reset_navigation();
    }

    pub fn set_filter_by(&mut self, filter_by: FilterBy) {
        self.filter_by = filter_by;
        self.apply_filters_and_sort();
        self.reset_navigation();
    }

    fn apply_filters_and_sort(&mut self) {
        // Apply filters first
        self.filtered_commands = match self.filter_by {
            FilterBy::All => self.commands.clone(),
            FilterBy::Failed => self
                .commands
                .iter()
                .filter(|cmd| cmd.exit_code.is_some() && cmd.exit_code.unwrap() != 0)
                .cloned()
                .collect(),
            FilterBy::Experiments => self
                .commands
                .iter()
                .filter(|cmd| cmd.is_experiment)
                .cloned()
                .collect(),
            FilterBy::Recent => {
                let mut recent = self.commands.clone();
                recent.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                recent.into_iter().take(100).collect()
            }
        };

        // Apply sorting
        match self.sort_by {
            SortBy::Time => {
                self.filtered_commands
                    .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            }
            SortBy::Count => {
                // Sort by command frequency (would need to implement frequency counting)
                self.filtered_commands
                    .sort_by(|a, b| a.command.cmp(&b.command));
            }
            SortBy::Host => {
                self.filtered_commands
                    .sort_by(|a, b| a.host_id.cmp(&b.host_id));
            }
            SortBy::Danger => {
                self.filtered_commands
                    .sort_by(|a, b| match (a.is_dangerous, b.is_dangerous) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.timestamp.cmp(&b.timestamp),
                    });
            }
            SortBy::Success => {
                self.filtered_commands
                    .sort_by(|a, b| match (a.exit_code, b.exit_code) {
                        (Some(0), Some(x)) if x != 0 => std::cmp::Ordering::Less,
                        (Some(x), Some(0)) if x != 0 => std::cmp::Ordering::Greater,
                        _ => a.timestamp.cmp(&b.timestamp),
                    });
            }
            SortBy::Length => {
                self.filtered_commands
                    .sort_by(|a, b| b.command.len().cmp(&a.command.len()));
            }
        }
    }

    pub fn get_filtered_commands(&self) -> &[Command] {
        &self.filtered_commands
    }

    pub fn set_search_filter(&mut self, filter: SearchFilter) {
        self.search_filter = filter;
        self.reset_navigation();
    }

    pub fn get_search_filter(&self) -> &SearchFilter {
        &self.search_filter
    }

    pub fn handle_function_key(&mut self, key: u8) {
        // Only handle function keys in Search tab
        if self.current_tab == Tab::Search {
            match key {
                1 => self.set_search_filter(SearchFilter::Failed),
                2 => self.set_search_filter(SearchFilter::Dangerous),
                3 => self.set_search_filter(SearchFilter::Recent),
                4 => self.set_search_filter(SearchFilter::Experiments),
                _ => {}
            }
        }
    }

    pub fn handle_heatmap_key(&mut self, key: char) {
        if self.current_tab == Tab::Heatmap {
            match key.to_ascii_uppercase() {
                // Time range controls
                'D' => self.set_heatmap_time_range(crate::analysis::heatmap::TimeRange::Day),
                'W' => self.set_heatmap_time_range(crate::analysis::heatmap::TimeRange::Week),
                'M' => self.set_heatmap_time_range(crate::analysis::heatmap::TimeRange::Month),
                'Y' => self.set_heatmap_time_range(crate::analysis::heatmap::TimeRange::Year),
                // View mode controls
                'A' => self.set_heatmap_view_mode(crate::analysis::heatmap::ViewMode::All),
                'R' => self.set_heatmap_view_mode(crate::analysis::heatmap::ViewMode::Dangerous),
                'E' => self.set_heatmap_view_mode(crate::analysis::heatmap::ViewMode::Experiments),
                'F' => self.set_heatmap_view_mode(crate::analysis::heatmap::ViewMode::Failed),
                _ => {}
            }
        }
    }

    pub fn set_heatmap_time_range(&mut self, time_range: crate::analysis::heatmap::TimeRange) {
        self.heatmap_time_range = time_range;
        self.reset_navigation();
    }

    pub fn set_heatmap_view_mode(&mut self, view_mode: crate::analysis::heatmap::ViewMode) {
        self.heatmap_view_mode = view_mode;
        self.reset_navigation();
    }

    pub fn handle_alias_key(&mut self, key: char) {
        if self.current_tab == Tab::Aliases {
            match key.to_ascii_uppercase() {
                // Sorting keys
                'S' => {
                    // Sort by savings - this would be implemented with alias state
                    self.reset_navigation();
                }
                'F' => {
                    // Sort by frequency
                    self.reset_navigation();
                }
                'L' => {
                    // Sort by length
                    self.reset_navigation();
                }
                // Filter keys
                'G' => {
                    // Filter Git commands
                    self.reset_navigation();
                }
                'D' => {
                    // Filter Docker commands
                    self.reset_navigation();
                }
                'A' => {
                    // Show all commands
                    self.reset_navigation();
                }
                // Action keys
                'E' => {
                    // Export aliases
                    // This would trigger export functionality
                }
                'C' => {
                    // Copy selected alias
                    // This would copy to clipboard
                }
                'R' => {
                    // Refresh analysis
                    self.reset_navigation();
                }
                // Shell export keys
                'B' => {
                    // Export for Bash
                    // This would generate bash aliases
                }
                'Z' => {
                    // Export for Zsh
                    // This would generate zsh aliases
                }
                _ => {}
            }
        }
    }

    pub fn handle_network_key(&mut self, key: char) {
        if self.current_tab == Tab::Network {
            match key.to_ascii_uppercase() {
                // Filter keys
                'S' => {
                    // Filter secure endpoints only
                    self.reset_navigation();
                }
                'I' => {
                    // Filter insecure endpoints only
                    self.reset_navigation();
                }
                'A' => {
                    // Show all endpoints
                    self.reset_navigation();
                }
                // Sort keys
                'U' => {
                    // Sort by usage count
                    self.reset_navigation();
                }
                'T' => {
                    // Sort by time (most recent first)
                    self.reset_navigation();
                }
                'R' => {
                    // Sort by risk level
                    self.reset_navigation();
                }
                _ => {}
            }
        }
    }

    // Enhanced analytics methods
    pub fn refresh_analytics(&mut self) {
        let now = std::time::Instant::now();

        // Only refresh if cache is invalid or it's been more than 30 seconds
        if !self.analysis_cache_valid
            || now.duration_since(self.last_analysis_update).as_secs() > 30
        {
            let analyzer = StatsAnalyzer::new();
            self.command_stats = Some(analyzer.analyze_commands(&self.commands));
            self.session_stats = Some(analyzer.analyze_sessions(&self.commands));
            self.productivity_stats = Some(analyzer.analyze_productivity(&self.commands));

            self.last_analysis_update = now;
            self.analysis_cache_valid = true;
        }
    }

    pub fn invalidate_analytics_cache(&mut self) {
        self.analysis_cache_valid = false;
    }

    #[allow(dead_code)]
    pub fn get_command_stats(&mut self) -> &CommandStats {
        self.refresh_analytics();
        self.command_stats.as_ref().unwrap()
    }

    #[allow(dead_code)]
    pub fn get_session_stats(&mut self) -> &SessionStats {
        self.refresh_analytics();
        self.session_stats.as_ref().unwrap()
    }

    #[allow(dead_code)]
    pub fn get_productivity_stats(&mut self) -> &ProductivityStats {
        self.refresh_analytics();
        self.productivity_stats.as_ref().unwrap()
    }

    // Optimized command loading for large datasets
    #[allow(dead_code)]
    pub async fn load_commands_paginated(
        &mut self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Command>> {
        self.db.get_commands_paginated(offset, limit).await
    }

    // Background analytics update
    pub fn update_analytics_background(&mut self) {
        // This would be called periodically to update analytics without blocking UI
        if !self.analysis_cache_valid {
            self.refresh_analytics();
        }
    }
}
