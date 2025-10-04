use chrono::{DateTime, TimeZone, Utc};
use tempfile::TempDir;
use whiskerlog::*;

async fn create_test_database() -> (Database, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();
    (db, temp_dir)
}

fn create_test_command_with_id(id: i64, cmd: &str, timestamp: DateTime<Utc>) -> Command {
    Command {
        id: Some(id),
        command: cmd.to_string(),
        timestamp,
        exit_code: Some(0),
        duration: Some(100),
        working_directory: Some("/test".to_string()),
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
    }
}

#[tokio::test]
async fn test_database_creation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let db = Database::new(&db_path).await;
    assert!(db.is_ok());

    // Check that database file was created
    assert!(db_path.exists());
}

#[tokio::test]
async fn test_insert_and_retrieve_command() {
    let (mut db, _temp_dir) = create_test_database().await;

    let test_command = Command {
        id: None,
        command: "echo 'hello world'".to_string(),
        timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
        exit_code: Some(0),
        duration: Some(50),
        working_directory: Some("/home/user".to_string()),
        host_id: "laptop".to_string(),
        session_id: "session-123".to_string(),
        shell: "bash".to_string(),
        packages_used: vec![],
        network_endpoints: vec![],
        is_dangerous: false,
        danger_score: 0.1,
        danger_reasons: vec![],
        is_experiment: false,
        experiment_tags: vec![],
    };

    // Insert command
    db.insert_command(&test_command).await.unwrap();

    // Retrieve commands
    let commands = db.get_commands(None).await.unwrap();

    assert_eq!(commands.len(), 1);
    let retrieved = &commands[0];

    assert_eq!(retrieved.command, "echo 'hello world'");
    assert_eq!(retrieved.exit_code, Some(0));
    assert_eq!(retrieved.duration, Some(50));
    assert_eq!(retrieved.working_directory, Some("/home/user".to_string()));
    assert_eq!(retrieved.host_id, "laptop");
    assert_eq!(retrieved.session_id, "session-123");
    assert_eq!(retrieved.shell, "bash");
    assert!(!retrieved.is_dangerous);
    assert_eq!(retrieved.danger_score, 0.1);
    assert!(!retrieved.is_experiment);
}

#[tokio::test]
async fn test_insert_command_with_packages() {
    let (mut db, _temp_dir) = create_test_database().await;

    let package = PackageRef {
        manager: "npm".to_string(),
        name: "react".to_string(),
        version: Some("18.2.0".to_string()),
        action: "install".to_string(),
    };

    let test_command = Command {
        id: None,
        command: "npm install react@18.2.0".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        duration: Some(5000),
        working_directory: Some("/project".to_string()),
        host_id: "dev-machine".to_string(),
        session_id: "session-456".to_string(),
        shell: "zsh".to_string(),
        packages_used: vec![package],
        network_endpoints: vec![],
        is_dangerous: false,
        danger_score: 0.0,
        danger_reasons: vec![],
        is_experiment: false,
        experiment_tags: vec![],
    };

    db.insert_command(&test_command).await.unwrap();

    let commands = db.get_commands(None).await.unwrap();
    assert_eq!(commands.len(), 1);

    let retrieved = &commands[0];
    assert_eq!(retrieved.packages_used.len(), 1);

    let retrieved_package = &retrieved.packages_used[0];
    assert_eq!(retrieved_package.manager, "npm");
    assert_eq!(retrieved_package.name, "react");
    assert_eq!(retrieved_package.version, Some("18.2.0".to_string()));
    assert_eq!(retrieved_package.action, "install");
}

#[tokio::test]
async fn test_insert_command_with_network_endpoints() {
    let (mut db, _temp_dir) = create_test_database().await;

    let test_command = Command {
        id: None,
        command: "curl https://api.github.com/user/repos".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        duration: Some(1200),
        working_directory: Some("/home/user".to_string()),
        host_id: "workstation".to_string(),
        session_id: "session-789".to_string(),
        shell: "bash".to_string(),
        packages_used: vec![],
        network_endpoints: vec!["https://api.github.com".to_string()],
        is_dangerous: false,
        danger_score: 0.2,
        danger_reasons: vec![],
        is_experiment: false,
        experiment_tags: vec![],
    };

    db.insert_command(&test_command).await.unwrap();

    let commands = db.get_commands(None).await.unwrap();
    assert_eq!(commands.len(), 1);

    let retrieved = &commands[0];
    assert_eq!(retrieved.network_endpoints.len(), 1);
    assert_eq!(retrieved.network_endpoints[0], "https://api.github.com");
}

#[tokio::test]
async fn test_multiple_commands_insertion() {
    let (mut db, _temp_dir) = create_test_database().await;

    let commands = vec![
        create_test_command_with_id(
            1,
            "ls -la",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
        ),
        create_test_command_with_id(
            2,
            "git status",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 5, 0).unwrap(),
        ),
        create_test_command_with_id(
            3,
            "npm test",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 10, 0).unwrap(),
        ),
    ];

    for command in &commands {
        db.insert_command(command).await.unwrap();
    }

    let retrieved_commands = db.get_commands(None).await.unwrap();
    assert_eq!(retrieved_commands.len(), 3);

    // Commands should be ordered by timestamp (most recent first by default)
    assert_eq!(retrieved_commands[0].command, "npm test");
    assert_eq!(retrieved_commands[1].command, "git status");
    assert_eq!(retrieved_commands[2].command, "ls -la");
}

#[tokio::test]
async fn test_dangerous_command_storage() {
    let (mut db, _temp_dir) = create_test_database().await;

    let dangerous_command = Command {
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

    db.insert_command(&dangerous_command).await.unwrap();

    let commands = db.get_commands(None).await.unwrap();
    assert_eq!(commands.len(), 1);

    let retrieved = &commands[0];
    assert!(retrieved.is_dangerous);
    assert_eq!(retrieved.danger_score, 0.8);
    assert_eq!(retrieved.danger_reasons.len(), 1);
    assert_eq!(retrieved.danger_reasons[0], "destructive operation");
}

#[tokio::test]
async fn test_experiment_command_storage() {
    let (mut db, _temp_dir) = create_test_database().await;

    let experiment_command = Command {
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

    db.insert_command(&experiment_command).await.unwrap();

    let commands = db.get_commands(None).await.unwrap();
    assert_eq!(commands.len(), 1);

    let retrieved = &commands[0];
    assert!(retrieved.is_experiment);
    assert!(!retrieved.is_dangerous);
    assert_eq!(retrieved.experiment_tags.len(), 2);
    assert!(retrieved.experiment_tags.contains(&"learning".to_string()));
    assert!(retrieved.experiment_tags.contains(&"python".to_string()));
}

#[tokio::test]
async fn test_database_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("persistent.db");

    // Create database and insert command
    {
        let mut db = Database::new(&db_path).await.unwrap();
        let test_command = create_test_command_with_id(1, "echo 'persistence test'", Utc::now());
        db.insert_command(&test_command).await.unwrap();
    }

    // Reopen database and check command is still there
    {
        let mut db = Database::new(&db_path).await.unwrap();
        let commands = db.get_commands(None).await.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].command, "echo 'persistence test'");
    }
}

#[tokio::test]
async fn test_empty_database() {
    let (mut db, _temp_dir) = create_test_database().await;

    let commands = db.get_commands(None).await.unwrap();
    assert!(commands.is_empty());
}

#[tokio::test]
async fn test_command_with_null_fields() {
    let (mut db, _temp_dir) = create_test_database().await;

    let minimal_command = Command {
        id: None,
        command: "echo".to_string(),
        timestamp: Utc::now(),
        exit_code: None,
        duration: None,
        working_directory: None,
        host_id: "unknown".to_string(),
        session_id: "unknown".to_string(),
        shell: "bash".to_string(),
        packages_used: vec![],
        network_endpoints: vec![],
        is_dangerous: false,
        danger_score: 0.0,
        danger_reasons: vec![],
        is_experiment: false,
        experiment_tags: vec![],
    };

    db.insert_command(&minimal_command).await.unwrap();

    let commands = db.get_commands(None).await.unwrap();
    assert_eq!(commands.len(), 1);

    let retrieved = &commands[0];
    assert_eq!(retrieved.command, "echo");
    assert_eq!(retrieved.exit_code, None);
    assert_eq!(retrieved.duration, None);
    assert_eq!(retrieved.working_directory, None);
}

#[tokio::test]
async fn test_large_command_storage() {
    let (mut db, _temp_dir) = create_test_database().await;

    // Create a command with a very long command string
    let long_command = "a".repeat(10000);
    let test_command = Command {
        id: None,
        command: long_command.clone(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        duration: Some(1000),
        working_directory: Some("/test".to_string()),
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

    let commands = db.get_commands(None).await.unwrap();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, long_command);
}

#[tokio::test]
async fn test_limited_command_retrieval() {
    let (mut db, _temp_dir) = create_test_database().await;

    // Insert 5 commands
    for i in 1..=5 {
        let command = create_test_command_with_id(i, &format!("command {}", i), Utc::now());
        db.insert_command(&command).await.unwrap();
    }

    // Retrieve only 3 commands
    let commands = db.get_commands(Some(3)).await.unwrap();
    assert_eq!(commands.len(), 3);

    // Retrieve all commands
    let all_commands = db.get_commands(None).await.unwrap();
    assert_eq!(all_commands.len(), 5);
}
