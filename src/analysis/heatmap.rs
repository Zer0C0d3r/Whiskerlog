use chrono::{Datelike, Duration, Timelike, Utc, Weekday};
use std::collections::HashMap;

use crate::history::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeRange {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    All,
    Dangerous,
    Experiments,
    Failed,
}

#[derive(Debug, Clone)]
pub struct HeatmapData {
    pub grid: [[f32; 7]; 24], // [hour][day_of_week] = activity_level (0.0 to 1.0)
    pub max_activity: f32,
    pub total_commands: usize,
}

#[derive(Debug, Clone)]
pub struct ActivityPeriod {
    pub hour: u32,
    pub day_of_week: Weekday,
    pub activity_level: f32,
    pub command_count: usize,
}

pub struct HeatmapAnalyzer;

impl Default for HeatmapAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl HeatmapAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_heatmap(
        &self,
        commands: &[Command],
        time_range: TimeRange,
        view_mode: ViewMode,
    ) -> HeatmapData {
        // Filter commands based on time range and view mode
        let filtered_commands = self.filter_commands(commands, time_range, view_mode);

        let mut activity_grid = [[0usize; 7]; 24]; // [hour][day_of_week] = count
        let mut max_count = 0usize;

        // Count commands by hour and day of week
        for cmd in &filtered_commands {
            let hour = cmd.timestamp.hour() as usize;
            let day_of_week = self.weekday_to_index(cmd.timestamp.weekday());

            activity_grid[hour][day_of_week] += 1;
            max_count = max_count.max(activity_grid[hour][day_of_week]);
        }

        // Normalize to 0.0-1.0 range
        let mut normalized_grid = [[0.0f32; 7]; 24];
        for hour in 0..24 {
            for day in 0..7 {
                if max_count > 0 {
                    normalized_grid[hour][day] = activity_grid[hour][day] as f32 / max_count as f32;
                }
            }
        }

        HeatmapData {
            grid: normalized_grid,
            max_activity: max_count as f32,
            total_commands: filtered_commands.len(),
        }
    }

    fn filter_commands(
        &self,
        commands: &[Command],
        time_range: TimeRange,
        view_mode: ViewMode,
    ) -> Vec<Command> {
        let now = Utc::now();
        let cutoff_time = match time_range {
            TimeRange::Day => now - Duration::days(1),
            TimeRange::Week => now - Duration::days(7),
            TimeRange::Month => now - Duration::days(30),
            TimeRange::Year => now - Duration::days(365),
        };

        // First, apply view mode filter to all commands
        let view_filtered: Vec<Command> = commands
            .iter()
            .filter(|cmd| match view_mode {
                ViewMode::All => true,
                ViewMode::Dangerous => cmd.is_dangerous,
                ViewMode::Experiments => cmd.is_experiment,
                ViewMode::Failed => cmd.exit_code.unwrap_or(0) != 0,
            })
            .cloned()
            .collect();

        // Then apply time range filter
        let time_filtered: Vec<Command> = view_filtered
            .iter()
            .filter(|cmd| cmd.timestamp >= cutoff_time)
            .cloned()
            .collect();

        // If no commands in the selected time range, show the most recent commands
        // up to the time range limit, or all available commands if fewer
        if time_filtered.is_empty() && !view_filtered.is_empty() {
            let limit = match time_range {
                TimeRange::Day => 50,    // Show up to 50 recent commands for day view
                TimeRange::Week => 200,  // Show up to 200 recent commands for week view
                TimeRange::Month => 500, // Show up to 500 recent commands for month view
                TimeRange::Year => view_filtered.len(), // Show all for year view
            };

            // Sort by timestamp (most recent first) and take the limit
            let mut sorted_commands = view_filtered;
            sorted_commands.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            sorted_commands.into_iter().take(limit).collect()
        } else {
            time_filtered
        }
    }

    pub fn get_peak_activity_periods(
        &self,
        heatmap: &HeatmapData,
        threshold: f32,
    ) -> Vec<ActivityPeriod> {
        let mut periods = Vec::new();

        for hour in 0..24 {
            for day in 0..7 {
                let activity_level = heatmap.grid[hour][day];
                if activity_level >= threshold {
                    periods.push(ActivityPeriod {
                        hour: hour as u32,
                        day_of_week: self.index_to_weekday(day),
                        activity_level,
                        command_count: (activity_level * heatmap.max_activity) as usize,
                    });
                }
            }
        }

        // Sort by activity level (highest first)
        periods.sort_by(|a, b| b.activity_level.partial_cmp(&a.activity_level).unwrap());
        periods
    }

    pub fn analyze_work_patterns(&self, commands: &[Command]) -> WorkPatternAnalysis {
        let mut weekday_commands = 0;
        let mut weekend_commands = 0;
        let mut work_hours_commands = 0; // 9 AM to 5 PM
        let mut late_night_commands = 0; // 10 PM to 6 AM

        for cmd in commands {
            let hour = cmd.timestamp.hour();
            let weekday = cmd.timestamp.weekday();

            // Weekday vs weekend
            match weekday {
                Weekday::Sat | Weekday::Sun => weekend_commands += 1,
                _ => weekday_commands += 1,
            }

            // Work hours vs other times
            if (9..17).contains(&hour) {
                work_hours_commands += 1;
            }

            // Late night activity
            if !(6..22).contains(&hour) {
                late_night_commands += 1;
            }
        }

        let total = commands.len() as f32;

        WorkPatternAnalysis {
            weekday_ratio: if total > 0.0 {
                weekday_commands as f32 / total
            } else {
                0.0
            },
            weekend_ratio: if total > 0.0 {
                weekend_commands as f32 / total
            } else {
                0.0
            },
            work_hours_ratio: if total > 0.0 {
                work_hours_commands as f32 / total
            } else {
                0.0
            },
            late_night_ratio: if total > 0.0 {
                late_night_commands as f32 / total
            } else {
                0.0
            },
            most_active_day: self.find_most_active_day(commands),
            most_active_hour: self.find_most_active_hour(commands),
        }
    }

    fn weekday_to_index(&self, weekday: Weekday) -> usize {
        match weekday {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        }
    }

    fn index_to_weekday(&self, index: usize) -> Weekday {
        match index {
            0 => Weekday::Mon,
            1 => Weekday::Tue,
            2 => Weekday::Wed,
            3 => Weekday::Thu,
            4 => Weekday::Fri,
            5 => Weekday::Sat,
            6 => Weekday::Sun,
            _ => Weekday::Mon, // Default fallback
        }
    }

    fn find_most_active_day(&self, commands: &[Command]) -> Weekday {
        let mut day_counts = HashMap::new();

        for cmd in commands {
            *day_counts.entry(cmd.timestamp.weekday()).or_insert(0) += 1;
        }

        day_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(day, _)| day)
            .unwrap_or(Weekday::Mon)
    }

    fn find_most_active_hour(&self, commands: &[Command]) -> u32 {
        let mut hour_counts = HashMap::new();

        for cmd in commands {
            *hour_counts.entry(cmd.timestamp.hour()).or_insert(0) += 1;
        }

        hour_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| hour)
            .unwrap_or(12)
    }
}

#[derive(Debug, Clone)]
pub struct WorkPatternAnalysis {
    pub weekday_ratio: f32,
    pub weekend_ratio: f32,
    pub work_hours_ratio: f32,
    pub late_night_ratio: f32,
    pub most_active_day: Weekday,
    pub most_active_hour: u32,
}
