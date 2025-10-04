use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use std::collections::HashMap;

use crate::history::Command;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommandStats {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub success_rate: f32,
    pub average_duration: Option<f64>,
    pub commands_per_day: f32,
    pub most_active_hour: u32,
    pub most_active_day: chrono::Weekday,
    pub top_commands: Vec<CommandFrequency>,
    pub shell_distribution: HashMap<String, usize>,
    pub host_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommandFrequency {
    pub command: String,
    pub count: usize,
    pub percentage: f32,
    pub last_used: DateTime<Utc>,
    pub average_duration: Option<f64>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub average_session_length: f64,
    pub average_commands_per_session: f32,
    pub longest_session: Duration,
    pub most_productive_session: String,
    pub session_distribution: HashMap<String, usize>, // by shell
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProductivityStats {
    pub productivity_score: f32,
    pub efficiency_indicators: Vec<String>,
    pub improvement_suggestions: Vec<String>,
    pub peak_hours: Vec<u32>,
    pub workflow_patterns: Vec<WorkflowPattern>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorkflowPattern {
    pub pattern: String,
    pub frequency: usize,
    pub efficiency_score: f32,
    pub description: String,
}

pub struct StatsAnalyzer;

impl Default for StatsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_commands(&self, commands: &[Command]) -> CommandStats {
        if commands.is_empty() {
            return CommandStats {
                total_commands: 0,
                unique_commands: 0,
                success_rate: 0.0,
                average_duration: None,
                commands_per_day: 0.0,
                most_active_hour: 12,
                most_active_day: chrono::Weekday::Mon,
                top_commands: Vec::new(),
                shell_distribution: HashMap::new(),
                host_distribution: HashMap::new(),
            };
        }

        let total_commands = commands.len();
        let unique_commands = self.count_unique_commands(commands);
        let success_rate = self.calculate_success_rate(commands);
        let average_duration = self.calculate_average_duration(commands);
        let commands_per_day = self.calculate_commands_per_day(commands);
        let most_active_hour = self.find_most_active_hour(commands);
        let most_active_day = self.find_most_active_day(commands);
        let top_commands = self.get_top_commands(commands, 10);
        let shell_distribution = self.get_shell_distribution(commands);
        let host_distribution = self.get_host_distribution(commands);

        CommandStats {
            total_commands,
            unique_commands,
            success_rate,
            average_duration,
            commands_per_day,
            most_active_hour,
            most_active_day,
            top_commands,
            shell_distribution,
            host_distribution,
        }
    }

    pub fn analyze_sessions(&self, commands: &[Command]) -> SessionStats {
        let mut sessions: HashMap<String, Vec<&Command>> = HashMap::new();

        // Group commands by session
        for cmd in commands {
            sessions
                .entry(cmd.session_id.clone())
                .or_default()
                .push(cmd);
        }

        let total_sessions = sessions.len();

        if total_sessions == 0 {
            return SessionStats {
                total_sessions: 0,
                average_session_length: 0.0,
                average_commands_per_session: 0.0,
                longest_session: Duration::zero(),
                most_productive_session: String::new(),
                session_distribution: HashMap::new(),
            };
        }

        let mut session_lengths = Vec::new();
        let mut session_command_counts = Vec::new();
        let mut longest_session = Duration::zero();
        let mut most_productive_session = String::new();
        let mut max_commands = 0;
        let mut shell_distribution = HashMap::new();

        for (session_id, session_commands) in &sessions {
            let command_count = session_commands.len();
            session_command_counts.push(command_count);

            if command_count > max_commands {
                max_commands = command_count;
                most_productive_session = session_id.clone();
            }

            // Calculate session duration
            if let (Some(first), Some(last)) = (
                session_commands.iter().map(|c| c.timestamp).min(),
                session_commands.iter().map(|c| c.timestamp).max(),
            ) {
                let duration = last - first;
                session_lengths.push(duration.num_minutes() as f64);

                if duration > longest_session {
                    longest_session = duration;
                }
            }

            // Track shell distribution
            for cmd in session_commands {
                *shell_distribution.entry(cmd.shell.clone()).or_insert(0) += 1;
            }
        }

        let average_session_length = if !session_lengths.is_empty() {
            session_lengths.iter().sum::<f64>() / session_lengths.len() as f64
        } else {
            0.0
        };

        let average_commands_per_session = if !session_command_counts.is_empty() {
            session_command_counts.iter().sum::<usize>() as f32
                / session_command_counts.len() as f32
        } else {
            0.0
        };

        SessionStats {
            total_sessions,
            average_session_length,
            average_commands_per_session,
            longest_session,
            most_productive_session,
            session_distribution: shell_distribution,
        }
    }

    pub fn analyze_productivity(&self, commands: &[Command]) -> ProductivityStats {
        let productivity_score = self.calculate_productivity_score(commands);
        let efficiency_indicators = self.identify_efficiency_indicators(commands);
        let improvement_suggestions = self.generate_improvement_suggestions(commands);
        let peak_hours = self.identify_peak_hours(commands);
        let workflow_patterns = self.detect_workflow_patterns(commands);

        ProductivityStats {
            productivity_score,
            efficiency_indicators,
            improvement_suggestions,
            peak_hours,
            workflow_patterns,
        }
    }

    fn count_unique_commands(&self, commands: &[Command]) -> usize {
        let mut unique = std::collections::HashSet::new();
        for cmd in commands {
            unique.insert(&cmd.command);
        }
        unique.len()
    }

    fn calculate_success_rate(&self, commands: &[Command]) -> f32 {
        let total_with_exit_code = commands.iter().filter(|c| c.exit_code.is_some()).count();
        if total_with_exit_code == 0 {
            return 1.0; // Assume success if no exit codes
        }

        let successful = commands.iter().filter(|c| c.exit_code == Some(0)).count();
        successful as f32 / total_with_exit_code as f32
    }

    fn calculate_average_duration(&self, commands: &[Command]) -> Option<f64> {
        let durations: Vec<u64> = commands.iter().filter_map(|c| c.duration).collect();
        if durations.is_empty() {
            None
        } else {
            Some(durations.iter().sum::<u64>() as f64 / durations.len() as f64)
        }
    }

    fn calculate_commands_per_day(&self, commands: &[Command]) -> f32 {
        if commands.is_empty() {
            return 0.0;
        }

        let first = commands.iter().map(|c| c.timestamp).min().unwrap();
        let last = commands.iter().map(|c| c.timestamp).max().unwrap();
        let days = (last - first).num_days().max(1) as f32;

        commands.len() as f32 / days
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

    fn find_most_active_day(&self, commands: &[Command]) -> chrono::Weekday {
        let mut day_counts = HashMap::new();

        for cmd in commands {
            *day_counts.entry(cmd.timestamp.weekday()).or_insert(0) += 1;
        }

        day_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(day, _)| day)
            .unwrap_or(chrono::Weekday::Mon)
    }

    fn get_top_commands(&self, commands: &[Command], limit: usize) -> Vec<CommandFrequency> {
        let mut command_stats: HashMap<String, (usize, DateTime<Utc>, Vec<u64>)> = HashMap::new();

        for cmd in commands {
            let entry =
                command_stats
                    .entry(cmd.command.clone())
                    .or_insert((0, cmd.timestamp, Vec::new()));
            entry.0 += 1; // count
            entry.1 = entry.1.max(cmd.timestamp); // last used
            if let Some(duration) = cmd.duration {
                entry.2.push(duration); // durations
            }
        }

        let total_commands = commands.len() as f32;
        let mut frequencies: Vec<_> = command_stats
            .into_iter()
            .map(|(command, (count, last_used, durations))| {
                let average_duration = if durations.is_empty() {
                    None
                } else {
                    Some(durations.iter().sum::<u64>() as f64 / durations.len() as f64)
                };

                CommandFrequency {
                    command,
                    count,
                    percentage: (count as f32 / total_commands) * 100.0,
                    last_used,
                    average_duration,
                }
            })
            .collect();

        frequencies.sort_by(|a, b| b.count.cmp(&a.count));
        frequencies.truncate(limit);
        frequencies
    }

    fn get_shell_distribution(&self, commands: &[Command]) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for cmd in commands {
            *distribution.entry(cmd.shell.clone()).or_insert(0) += 1;
        }
        distribution
    }

    fn get_host_distribution(&self, commands: &[Command]) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for cmd in commands {
            *distribution.entry(cmd.host_id.clone()).or_insert(0) += 1;
        }
        distribution
    }

    fn calculate_productivity_score(&self, commands: &[Command]) -> f32 {
        if commands.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;

        // Success rate component (0-30 points)
        let success_rate = self.calculate_success_rate(commands);
        score += success_rate * 30.0;

        // Command diversity component (0-25 points)
        let unique_ratio = self.count_unique_commands(commands) as f32 / commands.len() as f32;
        score += unique_ratio * 25.0;

        // Efficiency component (0-25 points) - based on average command length and complexity
        let avg_complexity = self.calculate_average_complexity(commands);
        score += (avg_complexity / 10.0) * 25.0;

        // Learning component (0-20 points) - based on experiment ratio
        let experiment_ratio =
            commands.iter().filter(|c| c.is_experiment).count() as f32 / commands.len() as f32;
        score += experiment_ratio * 20.0;

        score.min(100.0)
    }

    fn calculate_average_complexity(&self, commands: &[Command]) -> f32 {
        if commands.is_empty() {
            return 0.0;
        }

        let total_complexity: f32 = commands
            .iter()
            .map(|cmd| self.estimate_command_complexity(&cmd.command))
            .sum();

        total_complexity / commands.len() as f32
    }

    fn estimate_command_complexity(&self, command: &str) -> f32 {
        let mut complexity = 1.0;

        // Word count
        let word_count = command.split_whitespace().count();
        complexity += (word_count as f32 - 1.0) * 0.5;

        // Special characters and operators
        if command.contains('|') {
            complexity += 2.0;
        }
        if command.contains('>') || command.contains('<') {
            complexity += 1.0;
        }
        if command.contains("&&") || command.contains("||") {
            complexity += 1.5;
        }
        if command.contains("$(") || command.contains("`") {
            complexity += 2.0;
        }
        if command.contains("--") {
            complexity += 0.5;
        }

        complexity.min(10.0)
    }

    fn identify_efficiency_indicators(&self, commands: &[Command]) -> Vec<String> {
        let mut indicators = Vec::new();

        let success_rate = self.calculate_success_rate(commands);
        if success_rate > 0.9 {
            indicators.push("High command success rate".to_string());
        }

        let unique_ratio = self.count_unique_commands(commands) as f32 / commands.len() as f32;
        if unique_ratio > 0.7 {
            indicators.push("Good command diversity".to_string());
        }

        let experiment_ratio =
            commands.iter().filter(|c| c.is_experiment).count() as f32 / commands.len() as f32;
        if experiment_ratio > 0.1 {
            indicators.push("Active learning and experimentation".to_string());
        }

        if let Some(avg_duration) = self.calculate_average_duration(commands) {
            if avg_duration < 1000.0 {
                // Less than 1 second average
                indicators.push("Fast command execution".to_string());
            }
        }

        indicators
    }

    fn generate_improvement_suggestions(&self, commands: &[Command]) -> Vec<String> {
        let mut suggestions = Vec::new();

        let success_rate = self.calculate_success_rate(commands);
        if success_rate < 0.8 {
            suggestions
                .push("Consider using --help or man pages to reduce command failures".to_string());
        }

        // Check for repetitive commands that could be aliased
        let top_commands = self.get_top_commands(commands, 5);
        for cmd_freq in &top_commands {
            if cmd_freq.command.len() > 20 && cmd_freq.count > 5 {
                suggestions.push(format!(
                    "Consider creating an alias for '{}'",
                    if cmd_freq.command.len() > 30 {
                        format!("{}...", &cmd_freq.command[..27])
                    } else {
                        cmd_freq.command.clone()
                    }
                ));
                break; // Only suggest one alias at a time
            }
        }

        let experiment_ratio =
            commands.iter().filter(|c| c.is_experiment).count() as f32 / commands.len() as f32;
        if experiment_ratio < 0.05 {
            suggestions
                .push("Try exploring new tools and commands to expand your skills".to_string());
        }

        // Check for dangerous command usage
        let dangerous_ratio =
            commands.iter().filter(|c| c.is_dangerous).count() as f32 / commands.len() as f32;
        if dangerous_ratio > 0.1 {
            suggestions
                .push("Review dangerous commands and consider safer alternatives".to_string());
        }

        suggestions.truncate(5);
        suggestions
    }

    fn identify_peak_hours(&self, commands: &[Command]) -> Vec<u32> {
        let mut hour_counts = HashMap::new();

        for cmd in commands {
            *hour_counts.entry(cmd.timestamp.hour()).or_insert(0) += 1;
        }

        let max_count = hour_counts.values().max().unwrap_or(&0);
        let threshold = (*max_count as f32 * 0.7) as usize; // 70% of peak activity

        let mut peak_hours: Vec<_> = hour_counts
            .into_iter()
            .filter(|(_, count)| *count >= threshold)
            .map(|(hour, _)| hour)
            .collect();

        peak_hours.sort();
        peak_hours
    }

    fn detect_workflow_patterns(&self, commands: &[Command]) -> Vec<WorkflowPattern> {
        let mut patterns = Vec::new();

        // Detect common command sequences
        let sequences = self.find_common_sequences(commands, 3);
        for (sequence, frequency) in sequences {
            if frequency >= 3 {
                let efficiency_score = self.calculate_sequence_efficiency(&sequence, commands);
                patterns.push(WorkflowPattern {
                    pattern: sequence.join(" → "),
                    frequency,
                    efficiency_score,
                    description: format!("Common workflow sequence (used {} times)", frequency),
                });
            }
        }

        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        patterns.truncate(5);
        patterns
    }

    fn find_common_sequences(
        &self,
        commands: &[Command],
        length: usize,
    ) -> HashMap<Vec<String>, usize> {
        let mut sequences = HashMap::new();

        // Group commands by session first
        let mut sessions: HashMap<String, Vec<&Command>> = HashMap::new();
        for cmd in commands {
            sessions
                .entry(cmd.session_id.clone())
                .or_default()
                .push(cmd);
        }

        for session_commands in sessions.values() {
            if session_commands.len() < length {
                continue;
            }

            // Sort by timestamp within session
            let mut sorted_commands = session_commands.clone();
            sorted_commands.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            // Extract sequences
            for window in sorted_commands.windows(length) {
                let sequence: Vec<String> = window
                    .iter()
                    .map(|cmd| {
                        cmd.command
                            .split_whitespace()
                            .next()
                            .unwrap_or(&cmd.command)
                            .to_string()
                    })
                    .collect();

                *sequences.entry(sequence).or_insert(0) += 1;
            }
        }

        sequences
    }

    fn calculate_sequence_efficiency(&self, sequence: &[String], commands: &[Command]) -> f32 {
        // Simple efficiency calculation based on success rate of commands in sequence
        let mut total_success = 0;
        let mut total_commands = 0;

        for cmd_name in sequence {
            for cmd in commands {
                if cmd.command.starts_with(cmd_name) {
                    total_commands += 1;
                    if cmd.exit_code == Some(0) {
                        total_success += 1;
                    }
                }
            }
        }

        if total_commands == 0 {
            0.5 // Default efficiency
        } else {
            total_success as f32 / total_commands as f32
        }
    }
}
