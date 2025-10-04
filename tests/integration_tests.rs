use chrono::Utc;
use std::path::PathBuf;
use tempfile::TempDir;
use whiskerlog::*;

#[tokio::test]
async fn test_app_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let _config_path = temp_dir.path().join("config.toml");

    // Create a test config
    let config = Config {
        database_path: temp_dir.path().join("test.db"),
        history_paths: vec![temp_dir.path().join("test_history")],
        redaction_enabled: true,
        auto_import: false,
        danger_threshold: 0.7,
        experiment_detection: true,
    };

    config.save().unwrap();

    // Test app creation would work here if we had a way to inject config
    // For now, we'll test the components individually
}

#[test]
fn test_config_default() {
    let config = Config::default();
    assert!(config.redaction_enabled);
    assert!(config.auto_import);
    assert_eq!(config.danger_threshold, 0.7);
    assert!(config.experiment_detection);
}

#[test]
fn test_config_save_and_load() {
    // Test that config can be serialized and deserialized properly
    let original_config = Config {
        database_path: PathBuf::from("/tmp/test.db"),
        history_paths: vec![
            PathBuf::from("/tmp/history1"),
            PathBuf::from("/tmp/history2"),
        ],
        redaction_enabled: false,
        auto_import: true,
        danger_threshold: 0.5,
        experiment_detection: false,
    };

    // Test TOML serialization/deserialization directly
    let toml_string = toml::to_string(&original_config).unwrap();
    let deserialized_config: Config = toml::from_str(&toml_string).unwrap();

    // Verify the config was serialized and deserialized correctly
    assert_eq!(
        original_config.database_path,
        deserialized_config.database_path
    );
    assert_eq!(
        original_config.history_paths,
        deserialized_config.history_paths
    );
    assert_eq!(
        original_config.redaction_enabled,
        deserialized_config.redaction_enabled
    );
    assert_eq!(original_config.auto_import, deserialized_config.auto_import);
    assert_eq!(
        original_config.danger_threshold,
        deserialized_config.danger_threshold
    );
    assert_eq!(
        original_config.experiment_detection,
        deserialized_config.experiment_detection
    );

    // Test that default config is valid
    let default_config = Config::default();
    assert!(default_config.danger_threshold > 0.0);
    assert!(default_config.danger_threshold <= 1.0);
}

#[tokio::test]
async fn test_database_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut db = Database::new(&db_path).await.unwrap();

    // Test command insertion
    let test_command = Command {
        id: None,
        command: "ls -la".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        duration: Some(100),
        working_directory: Some("/home/user".to_string()),
        host_id: "test-host".to_string(),
        session_id: "test-session".to_string(),
        shell: "bash".to_string(),
        packages_used: vec![],
        network_endpoints: vec![],
        is_dangerous: false,
        danger_score: 0.0,
        danger_reasons: vec![],
        is_experiment: false,
        experiment_tags: vec![],
    };

    db.insert_command(&test_command).await.unwrap();

    // Test command retrieval
    let commands = db.get_commands(None).await.unwrap();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "ls -la");
    assert_eq!(commands[0].host_id, "test-host");
}

#[test]
fn test_command_creation() {
    let cmd = Command {
        id: None,
        command: "git status".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        duration: Some(50),
        working_directory: Some("/repo".to_string()),
        host_id: "dev-machine".to_string(),
        session_id: "session-123".to_string(),
        shell: "zsh".to_string(),
        packages_used: vec![],
        network_endpoints: vec![],
        is_dangerous: false,
        danger_score: 0.0,
        danger_reasons: vec![],
        is_experiment: false,
        experiment_tags: vec![],
    };

    assert_eq!(cmd.command, "git status");
    assert_eq!(cmd.host_id, "dev-machine");
    assert!(!cmd.is_dangerous);
}

#[test]
fn test_package_ref_creation() {
    let package = PackageRef {
        manager: "npm".to_string(),
        name: "react".to_string(),
        version: Some("18.2.0".to_string()),
        action: "install".to_string(),
    };

    assert_eq!(package.manager, "npm");
    assert_eq!(package.name, "react");
    assert_eq!(package.version, Some("18.2.0".to_string()));
    assert_eq!(package.action, "install");
}

#[test]
fn test_app_stats_default() {
    let stats = AppStats::default();
    assert_eq!(stats.total_commands, 0);
    assert_eq!(stats.total_sessions, 0);
    assert_eq!(stats.hosts_count, 0);
    assert_eq!(stats.dangerous_commands, 0);
    assert_eq!(stats.network_endpoints, 0);
    assert_eq!(stats.packages_used, 0);
    assert_eq!(stats.experiment_sessions, 0);
}

#[test]
fn test_tab_enum_functionality() {
    let tab = Tab::Summary;
    assert_eq!(tab.title(), "Summary");

    let all_tabs = Tab::all();
    assert!(!all_tabs.is_empty());
    assert!(all_tabs.contains(&Tab::Summary));
    assert!(all_tabs.contains(&Tab::Commands));
    assert!(all_tabs.contains(&Tab::Packages));
}

#[test]
fn test_command_with_packages() {
    let package = PackageRef {
        manager: "pip".to_string(),
        name: "requests".to_string(),
        version: Some("2.28.0".to_string()),
        action: "install".to_string(),
    };

    let cmd = Command {
        id: None,
        command: "pip install requests==2.28.0".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        duration: Some(1000),
        working_directory: Some("/project".to_string()),
        host_id: "dev-machine".to_string(),
        session_id: "session-456".to_string(),
        shell: "bash".to_string(),
        packages_used: vec![package],
        network_endpoints: vec![],
        is_dangerous: false,
        danger_score: 0.0,
        danger_reasons: vec![],
        is_experiment: false,
        experiment_tags: vec![],
    };

    assert_eq!(cmd.packages_used.len(), 1);
    assert_eq!(cmd.packages_used[0].manager, "pip");
    assert_eq!(cmd.packages_used[0].name, "requests");
}

#[test]
fn test_command_with_network_endpoints() {
    let cmd = Command {
        id: None,
        command: "curl https://api.github.com/user".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        duration: Some(500),
        working_directory: Some("/home/user".to_string()),
        host_id: "laptop".to_string(),
        session_id: "session-789".to_string(),
        shell: "bash".to_string(),
        packages_used: vec![],
        network_endpoints: vec!["https://api.github.com".to_string()],
        is_dangerous: false,
        danger_score: 0.0,
        danger_reasons: vec![],
        is_experiment: false,
        experiment_tags: vec![],
    };

    assert_eq!(cmd.network_endpoints.len(), 1);
    assert_eq!(cmd.network_endpoints[0], "https://api.github.com");
}

#[test]
fn test_dangerous_command() {
    let cmd = Command {
        id: None,
        command: "sudo rm -rf /tmp/*".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        duration: Some(2000),
        working_directory: Some("/tmp".to_string()),
        host_id: "server".to_string(),
        session_id: "session-danger".to_string(),
        shell: "bash".to_string(),
        packages_used: vec![],
        network_endpoints: vec![],
        is_dangerous: true,
        danger_score: 0.8,
        danger_reasons: vec!["destructive operation".to_string()],
        is_experiment: false,
        experiment_tags: vec![],
    };

    assert!(cmd.is_dangerous);
    assert_eq!(cmd.danger_score, 0.8);
    assert_eq!(cmd.danger_reasons.len(), 1);
}

#[test]
fn test_experiment_command() {
    let cmd = Command {
        id: None,
        command: "python -c 'import this'".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        duration: Some(100),
        working_directory: Some("/learning".to_string()),
        host_id: "laptop".to_string(),
        session_id: "session-learn".to_string(),
        shell: "python".to_string(),
        packages_used: vec![],
        network_endpoints: vec![],
        is_dangerous: false,
        danger_score: 0.0,
        danger_reasons: vec![],
        is_experiment: true,
        experiment_tags: vec!["learning".to_string(), "python".to_string()],
    };

    assert!(cmd.is_experiment);
    assert_eq!(cmd.experiment_tags.len(), 2);
    assert!(cmd.experiment_tags.contains(&"learning".to_string()));
}
