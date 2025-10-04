use super::PackageRef;
use regex::Regex;

pub struct HostDetector {
    ssh_regex: Regex,
    docker_regex: Regex,
    kubectl_regex: Regex,
}

impl Default for HostDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl HostDetector {
    pub fn new() -> Self {
        Self {
            ssh_regex: Regex::new(r"ssh\s+(?:(\w+)@)?([^\s]+)").unwrap(),
            docker_regex: Regex::new(r"docker\s+(?:exec|run).*?(?:-it\s+)?([^\s]+)").unwrap(),
            kubectl_regex: Regex::new(r"kubectl\s+exec.*?([^\s]+)").unwrap(),
        }
    }

    pub fn detect(&self, command: &str) -> String {
        // Check for SSH
        if let Some(captures) = self.ssh_regex.captures(command) {
            let user = captures.get(1).map(|m| m.as_str()).unwrap_or("unknown");
            let host = captures.get(2).unwrap().as_str();
            return format!("ssh:{}@{}", user, host);
        }

        // Check for Docker
        if let Some(captures) = self.docker_regex.captures(command) {
            let container = captures.get(1).unwrap().as_str();
            return format!("docker:{}", container);
        }

        // Check for Kubernetes
        if let Some(captures) = self.kubectl_regex.captures(command) {
            let pod = captures.get(1).unwrap().as_str();
            return format!("k8s:{}", pod);
        }

        "local".to_string()
    }
}

pub struct NetworkDetector {
    curl_regex: Regex,
    wget_regex: Regex,
    ssh_regex: Regex,
    db_regex: Regex,
}

impl Default for NetworkDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkDetector {
    pub fn new() -> Self {
        Self {
            curl_regex: Regex::new(r"curl\s+.*?(https?://[^\s]+)").unwrap(),
            wget_regex: Regex::new(r"wget\s+.*?(https?://[^\s]+)").unwrap(),
            ssh_regex: Regex::new(r"ssh\s+(?:\w+@)?([^\s]+)").unwrap(),
            db_regex: Regex::new(r"(?:psql|mysql|redis-cli).*?(?:-h\s+([^\s]+)|@([^\s]+))")
                .unwrap(),
        }
    }

    pub fn detect(&self, command: &str) -> Vec<String> {
        let mut endpoints = Vec::new();

        // HTTP/HTTPS endpoints
        for regex in [&self.curl_regex, &self.wget_regex] {
            if let Some(captures) = regex.captures(command) {
                if let Some(url) = captures.get(1) {
                    endpoints.push(url.as_str().to_string());
                }
            }
        }

        // SSH endpoints
        if let Some(captures) = self.ssh_regex.captures(command) {
            if let Some(host) = captures.get(1) {
                endpoints.push(format!("ssh://{}", host.as_str()));
            }
        }

        // Database endpoints
        if let Some(captures) = self.db_regex.captures(command) {
            let host = captures.get(1).or(captures.get(2));
            if let Some(host) = host {
                endpoints.push(format!("db://{}", host.as_str()));
            }
        }

        endpoints
    }
}

pub struct PackageDetector {
    npm_regex: Regex,
    apt_regex: Regex,
    pip_regex: Regex,
    cargo_regex: Regex,
    brew_regex: Regex,
}

impl Default for PackageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageDetector {
    pub fn new() -> Self {
        Self {
            npm_regex: Regex::new(r"npm\s+(install|remove|update)\s+([^\s]+)").unwrap(),
            apt_regex: Regex::new(r"(?:apt|apt-get)\s+(install|remove|update)\s+([^\s]+)").unwrap(),
            pip_regex: Regex::new(r"pip\s+(install|uninstall)\s+([^\s]+)").unwrap(),
            cargo_regex: Regex::new(r"cargo\s+(install|uninstall)\s+([^\s]+)").unwrap(),
            brew_regex: Regex::new(r"brew\s+(install|uninstall|update)\s+([^\s]+)").unwrap(),
        }
    }

    pub fn detect(&self, command: &str) -> Vec<PackageRef> {
        let mut packages = Vec::new();

        let detectors = [
            ("npm", &self.npm_regex),
            ("apt", &self.apt_regex),
            ("pip", &self.pip_regex),
            ("cargo", &self.cargo_regex),
            ("brew", &self.brew_regex),
        ];

        for (manager, regex) in detectors {
            if let Some(captures) = regex.captures(command) {
                let action = captures.get(1).unwrap().as_str();
                let name = captures.get(2).unwrap().as_str();

                packages.push(PackageRef {
                    manager: manager.to_string(),
                    name: name.to_string(),
                    version: None, // Could be enhanced to extract version
                    action: action.to_string(),
                });
            }
        }

        packages
    }
}

pub struct DangerResult {
    pub is_dangerous: bool,
    pub score: f32,
    pub reasons: Vec<String>,
}

pub struct DangerDetector {
    dangerous_commands: Vec<(&'static str, f32, &'static str)>,
    dangerous_patterns: Vec<(Regex, f32, &'static str)>,
}

impl Default for DangerDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DangerDetector {
    pub fn new() -> Self {
        let dangerous_patterns = vec![
            (
                Regex::new(r"rm\s+-rf\s+/").unwrap(),
                1.0,
                "Recursive delete from root",
            ),
            (
                Regex::new(r"chmod\s+777").unwrap(),
                0.8,
                "Overly permissive permissions",
            ),
            (
                Regex::new(r"sudo\s+rm").unwrap(),
                0.7,
                "Privileged file deletion",
            ),
            (
                Regex::new(r"dd\s+.*of=/dev/").unwrap(),
                0.9,
                "Direct disk write",
            ),
            (Regex::new(r"mkfs").unwrap(), 0.9, "Filesystem creation"),
            (
                Regex::new(r"curl.*\|\s*(?:bash|sh)").unwrap(),
                0.8,
                "Pipe to shell execution",
            ),
            (
                Regex::new(r"wget.*-O-.*\|\s*(?:bash|sh)").unwrap(),
                0.8,
                "Pipe to shell execution",
            ),
        ];

        Self {
            dangerous_commands: vec![
                ("rm", 0.6, "File deletion"),
                ("rmdir", 0.5, "Directory deletion"),
                ("mv", 0.3, "File movement"),
                ("cp", 0.2, "File copying"),
                ("chmod", 0.4, "Permission change"),
                ("chown", 0.4, "Ownership change"),
                ("sudo", 0.5, "Privileged execution"),
            ],
            dangerous_patterns,
        }
    }

    pub fn assess(&self, command: &str) -> DangerResult {
        let mut score: f32 = 0.0;
        let mut reasons = Vec::new();

        // Check dangerous patterns first (higher priority)
        for (pattern, pattern_score, reason) in &self.dangerous_patterns {
            if pattern.is_match(command) {
                score = score.max(*pattern_score);
                reasons.push(reason.to_string());
            }
        }

        // Check dangerous commands
        let first_word = command.split_whitespace().next().unwrap_or("");
        for (cmd, cmd_score, reason) in &self.dangerous_commands {
            if first_word == *cmd {
                score = score.max(*cmd_score);
                if !reasons.iter().any(|r| r.contains(reason)) {
                    reasons.push(reason.to_string());
                }
            }
        }

        DangerResult {
            is_dangerous: score > 0.5,
            score,
            reasons,
        }
    }
}

pub struct ExperimentResult {
    pub is_experiment: bool,
    pub tags: Vec<String>,
}

pub struct ExperimentDetector {
    learning_commands: Vec<&'static str>,
    help_patterns: Vec<Regex>,
    test_patterns: Vec<Regex>,
}

impl Default for ExperimentDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperimentDetector {
    pub fn new() -> Self {
        Self {
            learning_commands: vec![
                "man", "help", "tldr", "info", "which", "type", "whatis", "apropos",
            ],
            help_patterns: vec![
                Regex::new(r"--help").unwrap(),
                Regex::new(r"-h\b").unwrap(),
                Regex::new(r"--usage").unwrap(),
            ],
            test_patterns: vec![
                Regex::new(r"\btest\b").unwrap(),
                Regex::new(r"\btry\b").unwrap(),
                Regex::new(r"\bplay\b").unwrap(),
                Regex::new(r"\bsandbox\b").unwrap(),
                Regex::new(r"\bexperiment\b").unwrap(),
                Regex::new(r"\bdemo\b").unwrap(),
            ],
        }
    }

    pub fn detect(&self, command: &str) -> ExperimentResult {
        let mut is_experiment = false;
        let mut tags = Vec::new();

        let first_word = command.split_whitespace().next().unwrap_or("");

        // Check for learning commands
        if self.learning_commands.contains(&first_word) {
            is_experiment = true;
            tags.push("learning".to_string());
        }

        // Check for help patterns
        for pattern in &self.help_patterns {
            if pattern.is_match(command) {
                is_experiment = true;
                tags.push("help-seeking".to_string());
                break;
            }
        }

        // Check for test patterns
        for pattern in &self.test_patterns {
            if pattern.is_match(command) {
                is_experiment = true;
                tags.push("testing".to_string());
                break;
            }
        }

        // Detect tool exploration (running command without args)
        if command.split_whitespace().count() == 1 && !self.learning_commands.contains(&first_word)
        {
            // Common tools that people run without args to see usage
            let exploration_tools = ["jq", "ffmpeg", "docker", "kubectl", "git", "curl", "grep"];
            if exploration_tools.contains(&first_word) {
                is_experiment = true;
                tags.push("tool-exploration".to_string());
            }
        }

        ExperimentResult {
            is_experiment,
            tags,
        }
    }
}
