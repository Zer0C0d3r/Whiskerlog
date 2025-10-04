pub mod detector;
pub mod enricher;
pub mod parser;

pub use enricher::CommandEnricher;
pub use parser::HistoryParser;
// pub use detector::*; // Unused for now

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: Option<i64>,
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub duration: Option<u64>, // milliseconds
    pub working_directory: Option<String>,
    pub session_id: String,
    pub host_id: String,
    pub network_endpoints: Vec<String>,
    pub packages_used: Vec<PackageRef>,
    pub is_experiment: bool,
    pub experiment_tags: Vec<String>,
    pub is_dangerous: bool,
    pub danger_score: f32,
    pub danger_reasons: Vec<String>,
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRef {
    pub manager: String, // "apt", "npm", "pip", "cargo", "brew"
    pub name: String,    // "docker", "pandas", "ripgrep"
    pub version: Option<String>,
    pub action: String, // "install", "remove", "update"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostType {
    Local,
    Ssh {
        user: String,
        host: String,
    },
    Docker {
        container: String,
        image: Option<String>,
    },
    Kubernetes {
        pod: String,
        namespace: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NetworkEndpoint {
    pub protocol: String, // "http", "ssh", "postgres"
    pub host: String,     // "github.com", "prod-db.internal"
    pub port: Option<u16>,
    pub is_secure: bool, // HTTPS vs HTTP, SSH vs telnet
}

impl Default for Command {
    fn default() -> Self {
        Self {
            id: None,
            command: String::new(),
            timestamp: Utc::now(),
            exit_code: None,
            duration: None,
            working_directory: None,
            session_id: String::new(),
            host_id: "local".to_string(),
            network_endpoints: Vec::new(),
            packages_used: Vec::new(),
            is_experiment: false,
            experiment_tags: Vec::new(),
            is_dangerous: false,
            danger_score: 0.0,
            danger_reasons: Vec::new(),
            shell: "unknown".to_string(),
        }
    }
}
