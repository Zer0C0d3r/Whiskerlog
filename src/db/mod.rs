use anyhow::Result;
use chrono::TimeZone;
use rusqlite::{params, Connection};
use std::path::Path;

use crate::history::Command;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let connection = Connection::open(path)?;
        let mut db = Self { connection };
        db.initialize().await?;
        Ok(db)
    }

    async fn initialize(&mut self) -> Result<()> {
        let sql = include_str!("schema.sql");
        self.connection.execute_batch(sql)?;
        Ok(())
    }

    pub async fn insert_command(&mut self, command: &Command) -> Result<i64> {
        let _id = self.connection.execute(
            "INSERT INTO commands (
                command, timestamp, exit_code, duration, working_directory,
                session_id, host_id, network_endpoints, packages_used,
                is_experiment, experiment_tags, is_dangerous, danger_score,
                danger_reasons, shell
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                command.command,
                command.timestamp.timestamp(),
                command.exit_code,
                command.duration.map(|d| d as i64),
                command.working_directory,
                command.session_id,
                command.host_id,
                serde_json::to_string(&command.network_endpoints).unwrap_or_default(),
                serde_json::to_string(&command.packages_used).unwrap_or_default(),
                command.is_experiment,
                serde_json::to_string(&command.experiment_tags).unwrap_or_default(),
                command.is_dangerous,
                command.danger_score,
                serde_json::to_string(&command.danger_reasons).unwrap_or_default(),
                command.shell,
            ],
        )?;

        Ok(self.connection.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub async fn get_commands_paginated(
        &mut self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Command>> {
        let sql = format!(
            "SELECT * FROM commands ORDER BY timestamp DESC LIMIT {} OFFSET {}",
            limit, offset
        );

        let mut stmt = self.connection.prepare(&sql)?;
        let command_iter = stmt.query_map([], |row| {
            Ok(Command {
                id: Some(row.get(0)?),
                command: row.get(1)?,
                timestamp: chrono::Utc
                    .timestamp_opt(row.get(2)?, 0)
                    .single()
                    .unwrap_or_else(chrono::Utc::now),
                exit_code: row.get(3)?,
                duration: row.get::<_, Option<i64>>(4)?.map(|d| d as u64),
                working_directory: row.get(5)?,
                session_id: row.get(6)?,
                host_id: row.get(7)?,
                network_endpoints: serde_json::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or_default(),
                packages_used: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
                is_experiment: row.get(10)?,
                experiment_tags: serde_json::from_str(&row.get::<_, String>(11)?)
                    .unwrap_or_default(),
                is_dangerous: row.get(12)?,
                danger_score: row.get(13)?,
                danger_reasons: serde_json::from_str(&row.get::<_, String>(14)?)
                    .unwrap_or_default(),
                shell: row.get(15)?,
            })
        })?;

        let mut commands = Vec::new();
        for command in command_iter {
            commands.push(command?);
        }

        Ok(commands)
    }

    #[allow(dead_code)]
    pub async fn get_commands(&mut self, limit: Option<usize>) -> Result<Vec<Command>> {
        let sql = match limit {
            Some(l) => format!("SELECT * FROM commands ORDER BY timestamp DESC LIMIT {}", l),
            None => "SELECT * FROM commands ORDER BY timestamp DESC".to_string(),
        };

        let mut stmt = self.connection.prepare(&sql)?;
        let command_iter = stmt.query_map([], |row| {
            Ok(Command {
                id: Some(row.get(0)?),
                command: row.get(1)?,
                timestamp: chrono::Utc
                    .timestamp_opt(row.get(2)?, 0)
                    .single()
                    .unwrap_or_else(chrono::Utc::now),
                exit_code: row.get(3)?,
                duration: row.get::<_, Option<i64>>(4)?.map(|d| d as u64),
                working_directory: row.get(5)?,
                session_id: row.get(6)?,
                host_id: row.get(7)?,
                network_endpoints: serde_json::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or_default(),
                packages_used: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
                is_experiment: row.get(10)?,
                experiment_tags: serde_json::from_str(&row.get::<_, String>(11)?)
                    .unwrap_or_default(),
                is_dangerous: row.get(12)?,
                danger_score: row.get(13)?,
                danger_reasons: serde_json::from_str(&row.get::<_, String>(14)?)
                    .unwrap_or_default(),
                shell: row.get(15)?,
            })
        })?;

        let mut commands = Vec::new();
        for command in command_iter {
            commands.push(command?);
        }

        Ok(commands)
    }
}
