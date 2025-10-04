use anyhow::Result;
use chrono::{TimeZone, Utc};
use regex::Regex;
use std::fs;

use super::{Command, CommandEnricher};

pub struct HistoryParser {
    enricher: CommandEnricher,
    #[allow(dead_code)]
    bash_regex: Regex,
    zsh_regex: Regex,
}

impl Default for HistoryParser {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryParser {
    pub fn new() -> Self {
        Self {
            enricher: CommandEnricher::new(),
            // Bash history format: command (no timestamp by default)
            bash_regex: Regex::new(r"^(.+)$").unwrap(),
            // Zsh history format: : timestamp:duration;command
            zsh_regex: Regex::new(r"^: (\d+):(\d+);(.+)$").unwrap(),
        }
    }

    pub async fn parse_all_histories(&self) -> Result<Vec<Command>> {
        let mut all_commands = Vec::new();

        // Parse bash history
        if let Ok(commands) = self.parse_bash_history().await {
            all_commands.extend(commands);
        }

        // Parse zsh history
        if let Ok(commands) = self.parse_zsh_history().await {
            all_commands.extend(commands);
        }

        // Parse fish history
        if let Ok(commands) = self.parse_fish_history().await {
            all_commands.extend(commands);
        }

        // Sort by timestamp
        all_commands.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(all_commands)
    }

    async fn parse_bash_history(&self) -> Result<Vec<Command>> {
        let home = dirs::home_dir().unwrap_or_default();
        let history_path = home.join(".bash_history");

        if !history_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&history_path)?;
        let mut commands = Vec::new();
        let session_id = format!("bash-{}", chrono::Utc::now().timestamp());

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let mut command = Command {
                command: line.to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(line_num as i64),
                session_id: session_id.clone(),
                shell: "bash".to_string(),
                ..Default::default()
            };

            // Enrich the command with additional metadata
            command = self.enricher.enrich(command).await;
            commands.push(command);
        }

        Ok(commands)
    }

    async fn parse_zsh_history(&self) -> Result<Vec<Command>> {
        let home = dirs::home_dir().unwrap_or_default();
        let history_path = home.join(".zsh_history");

        if !history_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&history_path)?;
        let mut commands = Vec::new();
        let session_id = format!("zsh-{}", chrono::Utc::now().timestamp());

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let mut command = if let Some(captures) = self.zsh_regex.captures(line) {
                let timestamp = captures.get(1).unwrap().as_str().parse::<i64>()?;
                let duration = captures.get(2).unwrap().as_str().parse::<u64>().ok();
                let cmd_text = captures.get(3).unwrap().as_str();

                Command {
                    command: cmd_text.to_string(),
                    timestamp: Utc
                        .timestamp_opt(timestamp, 0)
                        .single()
                        .unwrap_or_else(Utc::now),
                    duration: duration.map(|d| d * 1000), // convert to milliseconds
                    session_id: session_id.clone(),
                    shell: "zsh".to_string(),
                    ..Default::default()
                }
            } else {
                // Fallback for malformed lines
                Command {
                    command: line.to_string(),
                    timestamp: Utc::now(),
                    session_id: session_id.clone(),
                    shell: "zsh".to_string(),
                    ..Default::default()
                }
            };

            // Enrich the command with additional metadata
            command = self.enricher.enrich(command).await;
            commands.push(command);
        }

        Ok(commands)
    }

    async fn parse_fish_history(&self) -> Result<Vec<Command>> {
        let home = dirs::home_dir().unwrap_or_default();
        let history_path = home.join(".local/share/fish/fish_history");

        if !history_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&history_path)?;
        let mut commands = Vec::new();
        let session_id = format!("fish-{}", chrono::Utc::now().timestamp());

        let mut current_command = None;
        let mut current_timestamp = None;

        for line in content.lines() {
            if let Some(stripped) = line.strip_prefix("- cmd: ") {
                current_command = Some(stripped.to_string());
            } else if let Some(stripped) = line.strip_prefix("  when: ") {
                if let Ok(timestamp) = stripped.parse::<i64>() {
                    current_timestamp = Some(
                        Utc.timestamp_opt(timestamp, 0)
                            .single()
                            .unwrap_or_else(Utc::now),
                    );
                }
            } else if line.trim().is_empty() && current_command.is_some() {
                // End of entry
                if let Some(cmd_text) = current_command.take() {
                    let mut command = Command {
                        command: cmd_text,
                        timestamp: current_timestamp.unwrap_or_else(Utc::now),
                        session_id: session_id.clone(),
                        shell: "fish".to_string(),
                        ..Default::default()
                    };

                    // Enrich the command with additional metadata
                    command = self.enricher.enrich(command).await;
                    commands.push(command);
                }
                current_timestamp = None;
            }
        }

        // Handle last entry if file doesn't end with blank line
        if let Some(cmd_text) = current_command {
            let mut command = Command {
                command: cmd_text,
                timestamp: current_timestamp.unwrap_or_else(Utc::now),
                session_id: session_id.clone(),
                shell: "fish".to_string(),
                ..Default::default()
            };

            command = self.enricher.enrich(command).await;
            commands.push(command);
        }

        Ok(commands)
    }
}
