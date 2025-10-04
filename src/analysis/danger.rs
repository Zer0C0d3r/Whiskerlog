use std::collections::HashMap;

use crate::history::Command;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DangerAnalysis {
    pub total_dangerous: usize,
    pub danger_by_category: HashMap<String, usize>,
    pub danger_trends: Vec<DangerTrend>,
    pub top_risky_commands: Vec<RiskyCommand>,
    pub safety_recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DangerTrend {
    pub date: chrono::NaiveDate,
    pub danger_count: usize,
    pub total_count: usize,
    pub danger_ratio: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RiskyCommand {
    pub command: String,
    pub count: usize,
    pub max_danger_score: f32,
    pub reasons: Vec<String>,
    pub safer_alternatives: Vec<String>,
}

#[allow(dead_code)]
pub struct DangerAnalyzer;

#[allow(dead_code)]
impl Default for DangerAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl DangerAnalyzer {
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn analyze_danger_patterns(&self, commands: &[Command]) -> DangerAnalysis {
        let dangerous_commands: Vec<_> = commands.iter().filter(|c| c.is_dangerous).collect();

        // Categorize dangers
        let mut danger_by_category = HashMap::new();
        for cmd in &dangerous_commands {
            for reason in &cmd.danger_reasons {
                *danger_by_category.entry(reason.clone()).or_insert(0) += 1;
            }
        }

        // Analyze trends over time
        let danger_trends = self.calculate_danger_trends(commands);

        // Find top risky commands
        let top_risky_commands = self.find_top_risky_commands(commands);

        // Generate safety recommendations
        let safety_recommendations =
            self.generate_safety_recommendations(&danger_by_category, &top_risky_commands);

        DangerAnalysis {
            total_dangerous: dangerous_commands.len(),
            danger_by_category,
            danger_trends,
            top_risky_commands,
            safety_recommendations,
        }
    }

    #[allow(dead_code)]
    fn calculate_danger_trends(&self, commands: &[Command]) -> Vec<DangerTrend> {
        let mut daily_stats: HashMap<chrono::NaiveDate, (usize, usize)> = HashMap::new(); // (dangerous, total)

        for cmd in commands {
            let date = cmd.timestamp.date_naive();
            let entry = daily_stats.entry(date).or_insert((0, 0));
            entry.1 += 1; // total
            if cmd.is_dangerous {
                entry.0 += 1; // dangerous
            }
        }

        let mut trends: Vec<_> = daily_stats
            .into_iter()
            .map(|(date, (danger_count, total_count))| DangerTrend {
                date,
                danger_count,
                total_count,
                danger_ratio: if total_count > 0 {
                    danger_count as f32 / total_count as f32
                } else {
                    0.0
                },
            })
            .collect();

        trends.sort_by(|a, b| a.date.cmp(&b.date));
        trends
    }

    #[allow(dead_code)]
    fn find_top_risky_commands(&self, commands: &[Command]) -> Vec<RiskyCommand> {
        let mut command_risks: HashMap<String, (usize, f32, Vec<String>)> = HashMap::new(); // (count, max_score, reasons)

        for cmd in commands {
            if cmd.is_dangerous {
                let entry =
                    command_risks
                        .entry(cmd.command.clone())
                        .or_insert((0, 0.0, Vec::new()));
                entry.0 += 1; // count
                entry.1 = entry.1.max(cmd.danger_score); // max score

                // Collect unique reasons
                for reason in &cmd.danger_reasons {
                    if !entry.2.contains(reason) {
                        entry.2.push(reason.clone());
                    }
                }
            }
        }

        let mut risky_commands: Vec<_> = command_risks
            .into_iter()
            .map(|(command, (count, max_danger_score, reasons))| {
                let safer_alternatives = self.suggest_safer_alternatives(&command);
                RiskyCommand {
                    command,
                    count,
                    max_danger_score,
                    reasons,
                    safer_alternatives,
                }
            })
            .collect();

        // Sort by danger score * count (impact)
        risky_commands.sort_by(|a, b| {
            let impact_a = a.max_danger_score * a.count as f32;
            let impact_b = b.max_danger_score * b.count as f32;
            impact_b.partial_cmp(&impact_a).unwrap()
        });

        risky_commands.truncate(10); // Top 10
        risky_commands
    }

    #[allow(dead_code)]
    fn suggest_safer_alternatives(&self, command: &str) -> Vec<String> {
        let mut alternatives = Vec::new();

        if command.contains("rm -rf") {
            alternatives.push("Use 'rm -i' for interactive deletion".to_string());
            alternatives.push("Move to trash instead of permanent deletion".to_string());
            alternatives.push("Use 'find' with '-delete' for more control".to_string());
        }

        if command.contains("chmod 777") {
            alternatives.push("Use more restrictive permissions like 755 or 644".to_string());
            alternatives.push("Set specific user/group permissions instead".to_string());
        }

        if command.contains("sudo rm") {
            alternatives.push("Double-check the path before running".to_string());
            alternatives.push("Use 'sudo -l' to verify permissions first".to_string());
        }

        if command.contains("curl") && command.contains("| bash") {
            alternatives.push("Download script first, then review before executing".to_string());
            alternatives.push("Use package manager instead of direct script execution".to_string());
        }

        if command.contains("dd") {
            alternatives.push("Double-check input and output devices".to_string());
            alternatives.push("Use 'lsblk' to verify device names first".to_string());
            alternatives.push("Consider using 'cp' for file copying instead".to_string());
        }

        if alternatives.is_empty() {
            alternatives.push("Review command carefully before execution".to_string());
            alternatives.push("Test in a safe environment first".to_string());
        }

        alternatives
    }

    #[allow(dead_code)]
    fn generate_safety_recommendations(
        &self,
        danger_categories: &HashMap<String, usize>,
        risky_commands: &[RiskyCommand],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // General recommendations
        recommendations.push(
            "🛡️ Always backup important data before running destructive commands".to_string(),
        );
        recommendations
            .push("🔍 Use 'man' or '--help' to understand command options before use".to_string());
        recommendations.push("🧪 Test dangerous commands in a safe environment first".to_string());

        // Category-specific recommendations
        if danger_categories.get("File deletion").unwrap_or(&0) > &5 {
            recommendations.push(
                "📁 Consider using a trash utility instead of 'rm' for safer file deletion"
                    .to_string(),
            );
        }

        if danger_categories.get("Permission change").unwrap_or(&0) > &3 {
            recommendations
                .push("🔐 Use principle of least privilege - avoid 777 permissions".to_string());
        }

        if danger_categories.get("Privileged execution").unwrap_or(&0) > &10 {
            recommendations.push(
                "👑 Minimize sudo usage - use regular user permissions when possible".to_string(),
            );
        }

        // Command-specific recommendations
        for risky_cmd in risky_commands.iter().take(3) {
            if risky_cmd.count > 5 {
                recommendations.push(format!(
                    "⚠️ You frequently use '{}' - consider safer alternatives",
                    risky_cmd
                        .command
                        .split_whitespace()
                        .next()
                        .unwrap_or(&risky_cmd.command)
                ));
            }
        }

        recommendations.truncate(8); // Keep it manageable
        recommendations
    }

    #[allow(dead_code)]
    pub fn calculate_safety_score(&self, commands: &[Command]) -> f32 {
        if commands.is_empty() {
            return 1.0; // Perfect safety for no commands
        }

        let total_commands = commands.len() as f32;
        let dangerous_commands = commands.iter().filter(|c| c.is_dangerous).count() as f32;
        let total_danger_score: f32 = commands.iter().map(|c| c.danger_score).sum();

        // Base safety from ratio of safe commands
        let safe_ratio = (total_commands - dangerous_commands) / total_commands;

        // Penalty based on average danger score
        let avg_danger_score = total_danger_score / total_commands;
        let danger_penalty = avg_danger_score * 0.5;

        (safe_ratio - danger_penalty).clamp(0.0, 1.0)
    }
}
