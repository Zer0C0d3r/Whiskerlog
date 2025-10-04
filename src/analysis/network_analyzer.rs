use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::history::Command;

#[derive(Debug, Clone)]
pub struct NetworkAnalysis {
    pub total_network_commands: usize,
    pub unique_endpoints: usize,
    pub protocol_breakdown: HashMap<String, usize>,
    pub security_issues: Vec<SecurityIssue>,
    pub top_endpoints: Vec<EndpointStats>,
    pub connection_patterns: Vec<ConnectionPattern>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SecurityIssue {
    pub issue_type: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub affected_commands: Vec<String>,
    #[allow(dead_code)]
    pub recommendation: String,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct EndpointStats {
    pub endpoint: String,
    pub protocol: String,
    pub usage_count: usize,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_secure: bool,
    pub success_rate: f32,
}

#[derive(Debug, Clone)]
pub struct ConnectionPattern {
    pub pattern_type: String,
    pub description: String,
    pub frequency: usize,
    pub risk_level: SecuritySeverity,
}

pub struct NetworkAnalyzer;

impl Default for NetworkAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_network_activity(&self, commands: &[Command]) -> NetworkAnalysis {
        let network_commands: Vec<_> = commands
            .iter()
            .filter(|cmd| !cmd.network_endpoints.is_empty())
            .collect();

        let mut endpoint_stats = HashMap::new();
        let mut protocol_counts = HashMap::new();

        // Collect endpoint statistics
        for cmd in &network_commands {
            for endpoint in &cmd.network_endpoints {
                let protocol = self.extract_protocol(endpoint);
                *protocol_counts.entry(protocol.clone()).or_insert(0) += 1;

                let stats = endpoint_stats
                    .entry(endpoint.clone())
                    .or_insert(EndpointStats {
                        endpoint: endpoint.clone(),
                        protocol: protocol.clone(),
                        usage_count: 0,
                        first_seen: cmd.timestamp,
                        last_seen: cmd.timestamp,
                        is_secure: self.is_secure_endpoint(endpoint),
                        success_rate: 0.0,
                    });

                stats.usage_count += 1;
                stats.first_seen = stats.first_seen.min(cmd.timestamp);
                stats.last_seen = stats.last_seen.max(cmd.timestamp);
            }
        }

        // Calculate success rates (placeholder - would need exit code analysis)
        for stats in endpoint_stats.values_mut() {
            stats.success_rate = 0.95; // Placeholder
        }

        let mut top_endpoints: Vec<_> = endpoint_stats.into_values().collect();
        top_endpoints.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        top_endpoints.truncate(20);

        let security_issues = self.identify_security_issues(&network_commands);
        let connection_patterns = self.analyze_connection_patterns(&network_commands);

        NetworkAnalysis {
            total_network_commands: network_commands.len(),
            unique_endpoints: top_endpoints.len(),
            protocol_breakdown: protocol_counts,
            security_issues,
            top_endpoints,
            connection_patterns,
        }
    }

    fn extract_protocol(&self, endpoint: &str) -> String {
        if endpoint.starts_with("https://") {
            "HTTPS".to_string()
        } else if endpoint.starts_with("http://") {
            "HTTP".to_string()
        } else if endpoint.starts_with("ssh://") {
            "SSH".to_string()
        } else if endpoint.starts_with("db://") {
            "Database".to_string()
        } else if endpoint.contains(":22") {
            "SSH".to_string()
        } else if endpoint.contains(":80") {
            "HTTP".to_string()
        } else if endpoint.contains(":443") {
            "HTTPS".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    fn is_secure_endpoint(&self, endpoint: &str) -> bool {
        endpoint.starts_with("https://")
            || endpoint.starts_with("ssh://")
            || endpoint.contains(":22")
            || endpoint.contains(":443")
    }

    fn identify_security_issues(&self, commands: &[&Command]) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        // Check for insecure HTTP usage
        let mut insecure_http_commands = Vec::new();
        for cmd in commands {
            for endpoint in &cmd.network_endpoints {
                if endpoint.starts_with("http://") {
                    insecure_http_commands.push(cmd.command.clone());
                    break;
                }
            }
        }

        if !insecure_http_commands.is_empty() {
            issues.push(SecurityIssue {
                issue_type: "Insecure HTTP".to_string(),
                description: format!(
                    "{} commands using insecure HTTP protocol",
                    insecure_http_commands.len()
                ),
                severity: SecuritySeverity::Medium,
                affected_commands: insecure_http_commands,
                recommendation: "Use HTTPS instead of HTTP for secure communication".to_string(),
            });
        }

        // Check for potential credential exposure
        let mut credential_exposure_commands = Vec::new();
        for cmd in commands {
            if self.contains_potential_credentials(&cmd.command) {
                credential_exposure_commands.push(cmd.command.clone());
            }
        }

        if !credential_exposure_commands.is_empty() {
            issues.push(SecurityIssue {
                issue_type: "Credential Exposure".to_string(),
                description: "Commands may contain exposed credentials".to_string(),
                severity: SecuritySeverity::High,
                affected_commands: credential_exposure_commands,
                recommendation:
                    "Use environment variables or credential files instead of inline credentials"
                        .to_string(),
            });
        }

        // Check for suspicious endpoints
        let mut suspicious_endpoints = Vec::new();
        for cmd in commands {
            for endpoint in &cmd.network_endpoints {
                if self.is_suspicious_endpoint(endpoint) {
                    suspicious_endpoints.push(cmd.command.clone());
                    break;
                }
            }
        }

        if !suspicious_endpoints.is_empty() {
            issues.push(SecurityIssue {
                issue_type: "Suspicious Endpoints".to_string(),
                description: "Connections to potentially suspicious endpoints detected".to_string(),
                severity: SecuritySeverity::Medium,
                affected_commands: suspicious_endpoints,
                recommendation: "Verify the legitimacy of these endpoints before connecting"
                    .to_string(),
            });
        }

        issues
    }

    fn contains_potential_credentials(&self, command: &str) -> bool {
        let credential_patterns = [
            "password=",
            "pwd=",
            "pass=",
            "token=",
            "key=",
            "secret=",
            "user:",
            "username:",
            "login:",
            "auth:",
            "api_key=",
        ];

        credential_patterns
            .iter()
            .any(|pattern| command.to_lowercase().contains(pattern))
    }

    fn is_suspicious_endpoint(&self, endpoint: &str) -> bool {
        let suspicious_patterns = [
            "bit.ly",
            "tinyurl.com",
            "t.co",
            "goo.gl", // URL shorteners
            "pastebin.com",
            "hastebin.com",              // Paste sites
            "raw.githubusercontent.com", // Raw GitHub content (could be suspicious)
        ];

        suspicious_patterns
            .iter()
            .any(|pattern| endpoint.contains(pattern))
    }

    fn analyze_connection_patterns(&self, commands: &[&Command]) -> Vec<ConnectionPattern> {
        let mut patterns = Vec::new();

        // Pattern: Frequent API calls
        let api_commands = commands
            .iter()
            .filter(|cmd| cmd.network_endpoints.iter().any(|e| e.contains("api.")))
            .count();

        if api_commands > 10 {
            patterns.push(ConnectionPattern {
                pattern_type: "API Usage".to_string(),
                description: format!("High API usage detected ({} commands)", api_commands),
                frequency: api_commands,
                risk_level: SecuritySeverity::Low,
            });
        }

        // Pattern: Database connections
        let db_commands = commands
            .iter()
            .filter(|cmd| cmd.network_endpoints.iter().any(|e| e.starts_with("db://")))
            .count();

        if db_commands > 5 {
            patterns.push(ConnectionPattern {
                pattern_type: "Database Access".to_string(),
                description: format!("Database connections detected ({} commands)", db_commands),
                frequency: db_commands,
                risk_level: SecuritySeverity::Medium,
            });
        }

        // Pattern: SSH connections
        let ssh_commands = commands
            .iter()
            .filter(|cmd| {
                cmd.network_endpoints
                    .iter()
                    .any(|e| e.starts_with("ssh://"))
            })
            .count();

        if ssh_commands > 3 {
            patterns.push(ConnectionPattern {
                pattern_type: "Remote Access".to_string(),
                description: format!("SSH connections detected ({} commands)", ssh_commands),
                frequency: ssh_commands,
                risk_level: SecuritySeverity::Low,
            });
        }

        patterns
    }

    pub fn calculate_network_security_score(&self, analysis: &NetworkAnalysis) -> f32 {
        if analysis.total_network_commands == 0 {
            return 1.0; // Perfect score for no network activity
        }

        let mut score = 1.0;

        // Penalty for security issues
        for issue in &analysis.security_issues {
            let penalty = match issue.severity {
                SecuritySeverity::Critical => 0.4,
                SecuritySeverity::High => 0.3,
                SecuritySeverity::Medium => 0.2,
                SecuritySeverity::Low => 0.1,
            };
            score -= penalty
                * (issue.affected_commands.len() as f32 / analysis.total_network_commands as f32);
        }

        // Bonus for secure protocols
        let secure_endpoints = analysis
            .top_endpoints
            .iter()
            .filter(|e| e.is_secure)
            .map(|e| e.usage_count)
            .sum::<usize>() as f32;

        let total_usage = analysis
            .top_endpoints
            .iter()
            .map(|e| e.usage_count)
            .sum::<usize>() as f32;

        if total_usage > 0.0 {
            let secure_ratio = secure_endpoints / total_usage;
            score += secure_ratio * 0.2; // Up to 20% bonus for using secure protocols
        }

        score.clamp(0.0, 1.0)
    }
}
