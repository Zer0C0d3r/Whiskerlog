use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

use crate::history::Command;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExperimentAnalysis {
    pub total_experiment_commands: usize,
    pub experiment_sessions: Vec<ExperimentSession>,
    pub learning_patterns: Vec<LearningPattern>,
    pub tool_exploration: Vec<ToolExploration>,
    pub knowledge_gaps: Vec<KnowledgeGap>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExperimentSession {
    pub session_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_minutes: i64,
    pub command_count: usize,
    pub experiment_ratio: f32,
    pub primary_focus: String,
    pub tools_explored: Vec<String>,
    pub learning_indicators: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LearningPattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub frequency: usize,
    pub tools_involved: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PatternType {
    HelpSeeking,     // Frequent use of --help, man pages
    ToolExploration, // Running commands without args to see usage
    TrialAndError,   // Repeated similar commands with variations
    Documentation,   // Using man, info, tldr frequently
    Experimentation, // Commands with test, try, play keywords
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolExploration {
    pub tool: String,
    pub exploration_commands: Vec<String>,
    pub help_commands: usize,
    pub test_commands: usize,
    pub success_rate: f32,
    pub learning_progression: Vec<ProgressionStep>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProgressionStep {
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub complexity_level: u8, // 1-10 scale
    pub success: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KnowledgeGap {
    pub area: String,
    pub indicators: Vec<String>,
    pub suggested_resources: Vec<String>,
    pub priority: Priority,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[allow(dead_code)]
pub struct ExperimentDetector;

#[allow(dead_code)]
impl Default for ExperimentDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperimentDetector {
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn analyze_experiments(&self, commands: &[Command]) -> ExperimentAnalysis {
        let experiment_commands: Vec<_> = commands.iter().filter(|cmd| cmd.is_experiment).collect();

        let experiment_sessions = self.identify_experiment_sessions(commands);
        let learning_patterns = self.detect_learning_patterns(&experiment_commands);
        let tool_exploration = self.analyze_tool_exploration(&experiment_commands);
        let knowledge_gaps = self.identify_knowledge_gaps(&experiment_commands);
        let recommendations = self.generate_learning_recommendations(
            &learning_patterns,
            &tool_exploration,
            &knowledge_gaps,
        );

        ExperimentAnalysis {
            total_experiment_commands: experiment_commands.len(),
            experiment_sessions,
            learning_patterns,
            tool_exploration,
            knowledge_gaps,
            recommendations,
        }
    }

    #[allow(dead_code)]
    fn identify_experiment_sessions(&self, commands: &[Command]) -> Vec<ExperimentSession> {
        let mut sessions = HashMap::new();

        // Group commands by session
        for cmd in commands {
            sessions
                .entry(cmd.session_id.clone())
                .or_insert(Vec::new())
                .push(cmd);
        }

        let mut experiment_sessions = Vec::new();

        for (session_id, session_commands) in sessions {
            let experiment_count = session_commands.iter().filter(|c| c.is_experiment).count();
            let total_count = session_commands.len();
            let experiment_ratio = experiment_count as f32 / total_count as f32;

            // Consider it an experiment session if >30% of commands are experimental
            if experiment_ratio > 0.3 && experiment_count > 2 {
                let start_time = session_commands.iter().map(|c| c.timestamp).min().unwrap();
                let end_time = session_commands.iter().map(|c| c.timestamp).max().unwrap();
                let duration_minutes = (end_time - start_time).num_minutes();

                let tools_explored = self.extract_tools_from_session(&session_commands);
                let primary_focus = self.determine_session_focus(&session_commands);
                let learning_indicators = self.extract_learning_indicators(&session_commands);

                experiment_sessions.push(ExperimentSession {
                    session_id,
                    start_time,
                    end_time,
                    duration_minutes,
                    command_count: total_count,
                    experiment_ratio,
                    primary_focus,
                    tools_explored,
                    learning_indicators,
                });
            }
        }

        experiment_sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        experiment_sessions
    }

    #[allow(dead_code)]
    fn extract_tools_from_session(&self, commands: &[&Command]) -> Vec<String> {
        let mut tools = std::collections::HashSet::new();

        for cmd in commands {
            if let Some(tool) = cmd.command.split_whitespace().next() {
                tools.insert(tool.to_string());
            }
        }

        tools.into_iter().collect()
    }

    #[allow(dead_code)]
    fn determine_session_focus(&self, commands: &[&Command]) -> String {
        let mut tool_counts = HashMap::new();

        for cmd in commands {
            if let Some(tool) = cmd.command.split_whitespace().next() {
                *tool_counts.entry(tool.to_string()).or_insert(0) += 1;
            }
        }

        tool_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(tool, _)| tool)
            .unwrap_or_else(|| "General".to_string())
    }

    #[allow(dead_code)]
    fn extract_learning_indicators(&self, commands: &[&Command]) -> Vec<String> {
        let mut indicators = Vec::new();

        let help_count = commands
            .iter()
            .filter(|c| {
                c.command.contains("--help")
                    || c.command.starts_with("man ")
                    || c.command.starts_with("tldr ")
            })
            .count();

        if help_count > 0 {
            indicators.push(format!("Help-seeking: {} commands", help_count));
        }

        let test_count = commands
            .iter()
            .filter(|c| {
                c.command.contains("test")
                    || c.command.contains("try")
                    || c.command.contains("example")
            })
            .count();

        if test_count > 0 {
            indicators.push(format!("Testing: {} commands", test_count));
        }

        indicators
    }

    #[allow(dead_code)]
    fn detect_learning_patterns(&self, commands: &[&Command]) -> Vec<LearningPattern> {
        let mut patterns = Vec::new();

        // Help-seeking pattern
        let help_commands = commands
            .iter()
            .filter(|c| {
                c.command.contains("--help")
                    || c.command.contains("-h ")
                    || c.command.starts_with("man ")
                    || c.command.starts_with("tldr ")
                    || c.command.starts_with("info ")
            })
            .count();

        if help_commands > 5 {
            patterns.push(LearningPattern {
                pattern_type: PatternType::HelpSeeking,
                description: format!("Frequent help command usage ({} instances)", help_commands),
                frequency: help_commands,
                tools_involved: self.extract_help_tools(commands),
                confidence: 0.9,
            });
        }

        // Tool exploration pattern
        let exploration_commands = commands
            .iter()
            .filter(|c| {
                let parts: Vec<&str> = c.command.split_whitespace().collect();
                parts.len() == 1 && !["ls", "cd", "pwd", "clear"].contains(&parts[0])
            })
            .count();

        if exploration_commands > 3 {
            patterns.push(LearningPattern {
                pattern_type: PatternType::ToolExploration,
                description: format!(
                    "Tool exploration detected ({} bare commands)",
                    exploration_commands
                ),
                frequency: exploration_commands,
                tools_involved: self.extract_exploration_tools(commands),
                confidence: 0.8,
            });
        }

        // Trial and error pattern
        let trial_error_groups = self.detect_trial_and_error(commands);
        if !trial_error_groups.is_empty() {
            patterns.push(LearningPattern {
                pattern_type: PatternType::TrialAndError,
                description: format!(
                    "Trial and error learning ({} command groups)",
                    trial_error_groups.len()
                ),
                frequency: trial_error_groups.len(),
                tools_involved: trial_error_groups,
                confidence: 0.7,
            });
        }

        patterns
    }

    #[allow(dead_code)]
    fn extract_help_tools(&self, commands: &[&Command]) -> Vec<String> {
        let mut tools = std::collections::HashSet::new();

        for cmd in commands {
            if cmd.command.starts_with("man ") {
                if let Some(tool) = cmd.command.split_whitespace().nth(1) {
                    tools.insert(tool.to_string());
                }
            } else if cmd.command.contains("--help") {
                if let Some(tool) = cmd.command.split_whitespace().next() {
                    tools.insert(tool.to_string());
                }
            }
        }

        tools.into_iter().collect()
    }

    #[allow(dead_code)]
    fn extract_exploration_tools(&self, commands: &[&Command]) -> Vec<String> {
        let mut tools = std::collections::HashSet::new();

        for cmd in commands {
            let parts: Vec<&str> = cmd.command.split_whitespace().collect();
            if parts.len() == 1 {
                tools.insert(parts[0].to_string());
            }
        }

        tools.into_iter().collect()
    }

    #[allow(dead_code)]
    fn detect_trial_and_error(&self, commands: &[&Command]) -> Vec<String> {
        let mut groups = Vec::new();
        let mut tool_sequences: HashMap<String, Vec<&Command>> = HashMap::new();

        // Group commands by base tool
        for cmd in commands {
            if let Some(tool) = cmd.command.split_whitespace().next() {
                tool_sequences
                    .entry(tool.to_string())
                    .or_default()
                    .push(cmd);
            }
        }

        // Look for rapid sequences of similar commands
        for (tool, sequence) in tool_sequences {
            if sequence.len() >= 3 {
                // Check if commands are close in time and have variations
                let mut is_trial_error = false;
                for window in sequence.windows(3) {
                    let time_span = window[2].timestamp - window[0].timestamp;
                    if time_span <= Duration::minutes(10) {
                        // Check for command variations
                        let commands: Vec<_> = window.iter().map(|c| &c.command).collect();
                        if self.are_command_variations(&commands) {
                            is_trial_error = true;
                            break;
                        }
                    }
                }

                if is_trial_error {
                    groups.push(tool);
                }
            }
        }

        groups
    }

    #[allow(dead_code)]
    fn are_command_variations(&self, commands: &[&String]) -> bool {
        // Simple heuristic: commands start with same tool but have different arguments
        if commands.len() < 2 {
            return false;
        }

        let first_tool = commands[0].split_whitespace().next();
        for cmd in commands.iter().skip(1) {
            let tool = cmd.split_whitespace().next();
            if tool != first_tool {
                return false;
            }
            // Check if arguments are different
            if *cmd != commands[0] {
                return true;
            }
        }

        false
    }

    #[allow(dead_code)]
    fn analyze_tool_exploration(&self, commands: &[&Command]) -> Vec<ToolExploration> {
        let mut tool_data: HashMap<String, Vec<&Command>> = HashMap::new();

        // Group by tool
        for cmd in commands {
            if let Some(tool) = cmd.command.split_whitespace().next() {
                tool_data.entry(tool.to_string()).or_default().push(cmd);
            }
        }

        let mut explorations = Vec::new();

        for (tool, tool_commands) in tool_data {
            if tool_commands.len() >= 3 {
                // Minimum threshold for exploration
                let help_commands = tool_commands
                    .iter()
                    .filter(|c| c.command.contains("--help") || c.command.contains("-h"))
                    .count();

                let test_commands = tool_commands
                    .iter()
                    .filter(|c| c.command.contains("test") || c.command.contains("example"))
                    .count();

                let successful_commands = tool_commands
                    .iter()
                    .filter(|c| c.exit_code == Some(0))
                    .count();

                let success_rate = successful_commands as f32 / tool_commands.len() as f32;

                let progression = self.analyze_learning_progression(&tool_commands);

                explorations.push(ToolExploration {
                    tool,
                    exploration_commands: tool_commands.iter().map(|c| c.command.clone()).collect(),
                    help_commands,
                    test_commands,
                    success_rate,
                    learning_progression: progression,
                });
            }
        }

        explorations.sort_by(|a, b| {
            b.exploration_commands
                .len()
                .cmp(&a.exploration_commands.len())
        });
        explorations.truncate(10);
        explorations
    }

    #[allow(dead_code)]
    fn analyze_learning_progression(&self, commands: &[&Command]) -> Vec<ProgressionStep> {
        let mut progression = Vec::new();

        for cmd in commands {
            let complexity = self.estimate_command_complexity(&cmd.command);
            let success = cmd.exit_code == Some(0);

            progression.push(ProgressionStep {
                timestamp: cmd.timestamp,
                command: cmd.command.clone(),
                complexity_level: complexity,
                success,
            });
        }

        progression.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        progression
    }

    #[allow(dead_code)]
    fn estimate_command_complexity(&self, command: &str) -> u8 {
        let parts = command.split_whitespace().count();
        let has_pipes = command.contains('|');
        let has_redirects = command.contains('>') || command.contains('<');
        let has_complex_args = command.contains("--") || command.contains("$(");

        let mut complexity = 1;

        if parts > 3 {
            complexity += 1;
        }
        if parts > 6 {
            complexity += 1;
        }
        if has_pipes {
            complexity += 2;
        }
        if has_redirects {
            complexity += 1;
        }
        if has_complex_args {
            complexity += 1;
        }

        complexity.min(10)
    }

    #[allow(dead_code)]
    fn identify_knowledge_gaps(&self, commands: &[&Command]) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();

        // Analyze failed commands for knowledge gaps
        let failed_commands: Vec<_> = commands
            .iter()
            .filter(|c| c.exit_code.is_some() && c.exit_code != Some(0))
            .collect();

        if !failed_commands.is_empty() {
            let mut tool_failures: HashMap<String, usize> = HashMap::new();

            for cmd in &failed_commands {
                if let Some(tool) = cmd.command.split_whitespace().next() {
                    *tool_failures.entry(tool.to_string()).or_insert(0) += 1;
                }
            }

            for (tool, failure_count) in tool_failures {
                if failure_count >= 3 {
                    gaps.push(KnowledgeGap {
                        area: format!("{} usage", tool),
                        indicators: vec![format!("{} failed commands", failure_count)],
                        suggested_resources: vec![
                            format!("man {}", tool),
                            format!("{} --help", tool),
                            format!("Online tutorials for {}", tool),
                        ],
                        priority: if failure_count > 5 {
                            Priority::High
                        } else {
                            Priority::Medium
                        },
                    });
                }
            }
        }

        gaps
    }

    #[allow(dead_code)]
    fn generate_learning_recommendations(
        &self,
        patterns: &[LearningPattern],
        explorations: &[ToolExploration],
        gaps: &[KnowledgeGap],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Pattern-based recommendations
        for pattern in patterns {
            match pattern.pattern_type {
                PatternType::HelpSeeking => {
                    recommendations.push(
                        "📚 Great job using help resources! Consider bookmarking useful man pages"
                            .to_string(),
                    );
                }
                PatternType::ToolExploration => {
                    recommendations.push(
                        "🔍 Your tool exploration shows curiosity! Try 'tldr' for quick examples"
                            .to_string(),
                    );
                }
                PatternType::TrialAndError => {
                    recommendations.push("🧪 Trial and error is valuable! Consider testing in safe environments first".to_string());
                }
                _ => {}
            }
        }

        // Exploration-based recommendations
        for exploration in explorations.iter().take(3) {
            if exploration.success_rate < 0.5 {
                recommendations.push(format!(
                    "💡 Struggling with {}? Try starting with basic examples and building up",
                    exploration.tool
                ));
            } else if exploration.success_rate > 0.8 {
                recommendations.push(format!(
                    "🎉 You're mastering {}! Consider exploring advanced features",
                    exploration.tool
                ));
            }
        }

        // Gap-based recommendations
        for gap in gaps.iter().take(2) {
            if gap.priority == Priority::High {
                recommendations.push(format!(
                    "🎯 Focus on improving {} skills - high impact area",
                    gap.area
                ));
            }
        }

        // General learning recommendations
        if patterns
            .iter()
            .any(|p| matches!(p.pattern_type, PatternType::HelpSeeking))
        {
            recommendations.push(
                "📖 Consider creating a personal cheat sheet for frequently used commands"
                    .to_string(),
            );
        }

        recommendations.truncate(6);
        recommendations
    }

    #[allow(dead_code)]
    pub fn calculate_learning_score(&self, analysis: &ExperimentAnalysis) -> f32 {
        if analysis.total_experiment_commands == 0 {
            return 0.0;
        }

        let mut score = 0.0;

        // Base score from experiment ratio
        let experiment_ratio = analysis.total_experiment_commands as f32 / 100.0; // Normalize
        score += experiment_ratio.min(0.4); // Up to 40% for experimentation

        // Bonus for diverse learning patterns
        let pattern_diversity = analysis.learning_patterns.len() as f32 * 0.1;
        score += pattern_diversity.min(0.3);

        // Bonus for tool exploration success
        let avg_success_rate = if !analysis.tool_exploration.is_empty() {
            analysis
                .tool_exploration
                .iter()
                .map(|e| e.success_rate)
                .sum::<f32>()
                / analysis.tool_exploration.len() as f32
        } else {
            0.0
        };
        score += avg_success_rate * 0.3;

        score.min(1.0)
    }
}
