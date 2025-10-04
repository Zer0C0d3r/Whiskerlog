use std::collections::HashMap;

use crate::history::Command;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AliasSuggestion {
    pub command: String,
    pub suggested_alias: String,
    pub frequency: usize,
    pub time_saved_per_use: usize, // characters saved
    pub total_time_saved: usize,   // total characters that would be saved
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AliasAnalysis {
    pub suggestions: Vec<AliasSuggestion>,
    pub existing_aliases_usage: HashMap<String, usize>,
    pub potential_savings: usize, // total characters that could be saved
}

#[allow(dead_code)]
pub struct AliasSuggester;

#[allow(dead_code)]
impl Default for AliasSuggester {
    fn default() -> Self {
        Self::new()
    }
}

impl AliasSuggester {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_alias_opportunities(&self, commands: &[Command]) -> AliasAnalysis {
        // Early return for empty commands
        if commands.is_empty() {
            return AliasAnalysis {
                suggestions: Vec::new(),
                existing_aliases_usage: HashMap::new(),
                potential_savings: 0,
            };
        }

        // Safety check: limit analysis to prevent crashes with too many commands
        let commands_to_analyze = if commands.len() > 1000 {
            &commands[commands.len() - 1000..] // Take the most recent 1000 commands
        } else {
            commands
        };

        // Count command frequencies with better analysis
        let mut command_counts: HashMap<String, usize> = HashMap::new();
        let mut command_contexts: HashMap<String, Vec<String>> = HashMap::new();

        for cmd in commands_to_analyze {
            // Normalize command (remove variable parts like file paths, IDs)
            let normalized_cmd = self.normalize_command(&cmd.command);
            *command_counts.entry(normalized_cmd.clone()).or_insert(0) += 1;

            // Track command contexts for better alias suggestions
            command_contexts
                .entry(normalized_cmd)
                .or_default()
                .push(cmd.working_directory.clone().unwrap_or_default());
        }

        // Generate suggestions with enhanced logic
        let mut suggestions = Vec::new();
        let mut total_potential_savings = 0;

        for (command, count) in &command_counts {
            // Enhanced thresholds based on command complexity
            let min_frequency = if command.contains("git") || command.contains("docker") {
                2
            } else {
                3
            };
            let min_length = if command.split_whitespace().count() > 3 {
                8
            } else {
                12
            };

            if *count >= min_frequency && command.len() > min_length {
                if let Some(suggestion) = self.generate_alias_suggestion(command, *count) {
                    total_potential_savings += suggestion.total_time_saved;
                    suggestions.push(suggestion);
                }
            }
        }

        // Advanced sorting: prioritize by impact score (frequency × savings × complexity)
        suggestions.sort_by(|a, b| {
            let score_a =
                a.frequency * a.time_saved_per_use * self.calculate_complexity_score(&a.command);
            let score_b =
                b.frequency * b.time_saved_per_use * self.calculate_complexity_score(&b.command);
            score_b.cmp(&score_a)
        });
        suggestions.truncate(25); // Top 25 suggestions

        // Enhanced existing alias detection
        let existing_aliases_usage = self.detect_existing_aliases(commands);

        AliasAnalysis {
            suggestions,
            existing_aliases_usage,
            potential_savings: total_potential_savings,
        }
    }

    fn normalize_command(&self, command: &str) -> String {
        let normalized = command.to_string();

        // Simple pattern replacements without regex for now
        // Replace numbers with placeholder
        let words: Vec<&str> = normalized.split_whitespace().collect();
        let mut new_words = Vec::new();

        for word in words {
            if word.chars().all(|c| c.is_ascii_digit()) {
                new_words.push("N");
            } else if word.contains('/')
                && (word.ends_with(".txt")
                    || word.ends_with(".log")
                    || word.ends_with(".json")
                    || word.ends_with(".yaml")
                    || word.ends_with(".yml"))
            {
                new_words.push("/FILE");
            } else {
                new_words.push(word);
            }
        }

        new_words.join(" ")
    }

    fn calculate_complexity_score(&self, command: &str) -> usize {
        let mut score = 1;

        // More complex commands get higher scores
        score += command.split_whitespace().count().saturating_sub(1);
        score += command.matches("--").count();
        score += command.matches(" -").count();

        // Bonus for common complex tools
        if command.contains("docker") {
            score += 2;
        }
        if command.contains("kubectl") {
            score += 3;
        }
        if command.contains("git") {
            score += 1;
        }
        if command.contains("npm") || command.contains("yarn") {
            score += 1;
        }

        score
    }

    fn generate_alias_suggestion(
        &self,
        command: &str,
        frequency: usize,
    ) -> Option<AliasSuggestion> {
        let suggested_alias = self.create_alias_name(command)?;
        let time_saved_per_use = command.len().saturating_sub(suggested_alias.len());

        if time_saved_per_use < 3 {
            return None; // Not worth aliasing
        }

        Some(AliasSuggestion {
            command: command.to_string(),
            suggested_alias,
            frequency,
            time_saved_per_use,
            total_time_saved: time_saved_per_use * frequency,
        })
    }

    fn create_alias_name(&self, command: &str) -> Option<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let base_cmd = parts[0];

        // Enhanced patterns for alias generation
        match base_cmd {
            "git" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "status" => Some("gs".to_string()),
                        "add" => {
                            if parts.len() > 2 && parts[2] == "." {
                                Some("gaa".to_string()) // git add all
                            } else {
                                Some("ga".to_string())
                            }
                        }
                        "commit" => {
                            if parts.contains(&"-m") {
                                Some("gcm".to_string()) // git commit with message
                            } else if parts.contains(&"--amend") {
                                Some("gca".to_string()) // git commit amend
                            } else {
                                Some("gc".to_string())
                            }
                        }
                        "push" => {
                            if parts.contains(&"origin") {
                                Some("gpo".to_string()) // git push origin
                            } else {
                                Some("gp".to_string())
                            }
                        }
                        "pull" => {
                            if parts.contains(&"origin") {
                                Some("glo".to_string()) // git pull origin
                            } else {
                                Some("gl".to_string())
                            }
                        }
                        "checkout" => Some("gco".to_string()),
                        "branch" => Some("gb".to_string()),
                        "log" => {
                            if parts.contains(&"--oneline") {
                                Some("glog1".to_string())
                            } else {
                                Some("glog".to_string())
                            }
                        }
                        "diff" => Some("gd".to_string()),
                        "merge" => Some("gm".to_string()),
                        "rebase" => Some("gr".to_string()),
                        "stash" => Some("gst".to_string()),
                        "remote" => Some("grem".to_string()),
                        _ => Some(format!("g{}", parts[1].chars().next()?)),
                    }
                } else {
                    Some("g".to_string())
                }
            }
            "docker" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "ps" => Some("dps".to_string()),
                        "images" => Some("di".to_string()),
                        "run" => Some("dr".to_string()),
                        "exec" => Some("de".to_string()),
                        "build" => Some("db".to_string()),
                        "compose" => Some("dc".to_string()),
                        _ => Some(format!("d{}", parts[1].chars().next()?)),
                    }
                } else {
                    Some("d".to_string())
                }
            }
            "kubectl" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "get" => Some("kg".to_string()),
                        "describe" => Some("kd".to_string()),
                        "apply" => Some("ka".to_string()),
                        "delete" => Some("kdel".to_string()),
                        "logs" => Some("kl".to_string()),
                        "exec" => Some("ke".to_string()),
                        "port-forward" => Some("kpf".to_string()),
                        _ => Some(format!("k{}", parts[1].chars().next()?)),
                    }
                } else {
                    Some("k".to_string())
                }
            }
            "npm" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "install" => Some("ni".to_string()),
                        "start" => Some("ns".to_string()),
                        "test" => Some("nt".to_string()),
                        "run" => Some("nr".to_string()),
                        "build" => Some("nb".to_string()),
                        _ => Some(format!("n{}", parts[1].chars().next()?)),
                    }
                } else {
                    Some("n".to_string())
                }
            }
            "yarn" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "install" => Some("yi".to_string()),
                        "start" => Some("ys".to_string()),
                        "test" => Some("yt".to_string()),
                        "build" => Some("yb".to_string()),
                        "add" => Some("ya".to_string()),
                        _ => Some(format!("y{}", parts[1].chars().next()?)),
                    }
                } else {
                    Some("y".to_string())
                }
            }
            "cargo" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "build" => Some("cb".to_string()),
                        "run" => Some("cr".to_string()),
                        "test" => Some("ct".to_string()),
                        "check" => Some("cc".to_string()),
                        "clippy" => Some("ccl".to_string()),
                        _ => Some(format!("c{}", parts[1].chars().next()?)),
                    }
                } else {
                    Some("c".to_string())
                }
            }
            "systemctl" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "status" => Some("scs".to_string()),
                        "start" => Some("scst".to_string()),
                        "stop" => Some("scsp".to_string()),
                        "restart" => Some("scr".to_string()),
                        "enable" => Some("sce".to_string()),
                        "disable" => Some("scd".to_string()),
                        _ => Some(format!("sc{}", parts[1].chars().next()?)),
                    }
                } else {
                    Some("sc".to_string())
                }
            }
            "ls" => {
                if command.contains("-la") || command.contains("-al") {
                    Some("ll".to_string())
                } else if command.contains("-l") {
                    Some("l".to_string())
                } else {
                    None
                }
            }
            _ => {
                // Generic alias generation
                if command.len() > 15 {
                    // Take first letter of each word
                    let alias: String = parts
                        .iter()
                        .take(3) // Max 3 words
                        .filter_map(|word| word.chars().next())
                        .collect();

                    if alias.len() >= 2 && alias.len() <= 5 {
                        Some(alias)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn detect_existing_aliases(&self, commands: &[Command]) -> HashMap<String, usize> {
        let mut alias_usage = HashMap::new();

        // Common aliases to look for
        let common_aliases = [
            "ll", "la", "l", "gs", "ga", "gc", "gp", "gl", "gco", "gb", "dps", "di", "dr", "de",
            "db", "dc", "kg", "kd", "ka", "kl", "vim", "vi", "nano", "cat", "less", "more", "grep",
            "find",
        ];

        for cmd in commands {
            let first_word = cmd.command.split_whitespace().next().unwrap_or("");
            if common_aliases.contains(&first_word) {
                *alias_usage.entry(first_word.to_string()).or_insert(0) += 1;
            }
        }

        alias_usage
    }

    #[allow(dead_code)]
    pub fn generate_shell_aliases(&self, suggestions: &[AliasSuggestion], shell: &str) -> String {
        let mut output = String::new();

        match shell {
            "bash" | "zsh" => {
                output.push_str("# Generated aliases by Whiskerlog\n");
                for suggestion in suggestions.iter().take(10) {
                    output.push_str(&format!(
                        "alias {}='{}'\n",
                        suggestion.suggested_alias, suggestion.command
                    ));
                }
            }
            "fish" => {
                output.push_str("# Generated aliases by Whiskerlog\n");
                for suggestion in suggestions.iter().take(10) {
                    output.push_str(&format!(
                        "alias {} '{}'\n",
                        suggestion.suggested_alias, suggestion.command
                    ));
                }
            }
            _ => {
                output.push_str("# Shell not supported for alias generation\n");
            }
        }

        output
    }

    pub fn calculate_efficiency_gain(&self, analysis: &AliasAnalysis) -> f32 {
        if analysis.potential_savings == 0 {
            return 0.0;
        }

        // Estimate typing speed benefit (characters saved / average typing speed)
        // Assume average typing speed of 40 WPM = ~200 characters per minute
        let time_saved_minutes = analysis.potential_savings as f32 / 200.0;

        // Convert to efficiency percentage (arbitrary scale)
        (time_saved_minutes * 10.0).min(100.0)
    }
}
