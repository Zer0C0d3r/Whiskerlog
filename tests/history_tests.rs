use chrono::Utc;
use whiskerlog::history::*;
#[allow(unused_imports)]
use whiskerlog::*;

fn create_test_command(cmd: &str) -> Command {
    Command {
        id: None,
        command: cmd.to_string(),
        timestamp: Utc::now(),
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

#[test]
fn test_command_creation() {
    let command = create_test_command("ls -la");

    assert_eq!(command.command, "ls -la");
    assert_eq!(command.host_id, "test-host");
    assert_eq!(command.session_id, "test-session");
    assert_eq!(command.shell, "bash");
    assert!(!command.is_dangerous);
    assert!(!command.is_experiment);
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
fn test_command_with_packages() {
    let package = PackageRef {
        manager: "pip".to_string(),
        name: "requests".to_string(),
        version: None,
        action: "install".to_string(),
    };

    let mut command = create_test_command("pip install requests");
    command.packages_used.push(package);

    assert_eq!(command.packages_used.len(), 1);
    assert_eq!(command.packages_used[0].manager, "pip");
    assert_eq!(command.packages_used[0].name, "requests");
}

#[test]
fn test_command_with_network_endpoints() {
    let mut command = create_test_command("curl https://api.github.com");
    command
        .network_endpoints
        .push("https://api.github.com".to_string());

    assert_eq!(command.network_endpoints.len(), 1);
    assert_eq!(command.network_endpoints[0], "https://api.github.com");
}

#[test]
fn test_dangerous_command() {
    let mut command = create_test_command("sudo rm -rf /");
    command.is_dangerous = true;
    command.danger_score = 0.9;
    command
        .danger_reasons
        .push("destructive operation".to_string());

    assert!(command.is_dangerous);
    assert_eq!(command.danger_score, 0.9);
    assert_eq!(command.danger_reasons.len(), 1);
    assert_eq!(command.danger_reasons[0], "destructive operation");
}

#[test]
fn test_experiment_command() {
    let mut command = create_test_command("python -c 'import this'");
    command.is_experiment = true;
    command.experiment_tags.push("learning".to_string());
    command.experiment_tags.push("python".to_string());

    assert!(command.is_experiment);
    assert_eq!(command.experiment_tags.len(), 2);
    assert!(command.experiment_tags.contains(&"learning".to_string()));
    assert!(command.experiment_tags.contains(&"python".to_string()));
}

#[test]
fn test_command_enricher_creation() {
    let _enricher = CommandEnricher::new();

    // Test that enricher can be created
    // The actual enrichment methods are not exposed in the public API
    // so we just test creation
    // If we get here, creation succeeded
}

#[test]
fn test_history_parser_creation() {
    let _parser = HistoryParser::new();

    // Test that parser can be created
    // The actual parsing methods are not exposed in the public API
    // so we just test creation
    // If we get here, creation succeeded
}

#[test]
fn test_command_serialization() {
    let command = create_test_command("echo 'test'");

    // Test that command can be serialized to JSON
    let json = serde_json::to_string(&command);
    assert!(json.is_ok());

    // Test that it can be deserialized back
    let json_str = json.unwrap();
    let deserialized: Result<Command, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    let deserialized_command = deserialized.unwrap();
    assert_eq!(deserialized_command.command, "echo 'test'");
}

#[test]
fn test_package_ref_serialization() {
    let package = PackageRef {
        manager: "cargo".to_string(),
        name: "serde".to_string(),
        version: Some("1.0.0".to_string()),
        action: "add".to_string(),
    };

    // Test that package can be serialized to JSON
    let json = serde_json::to_string(&package);
    assert!(json.is_ok());

    // Test that it can be deserialized back
    let json_str = json.unwrap();
    let deserialized: Result<PackageRef, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    let deserialized_package = deserialized.unwrap();
    assert_eq!(deserialized_package.manager, "cargo");
    assert_eq!(deserialized_package.name, "serde");
}

#[test]
fn test_command_debug_trait() {
    let command = create_test_command("test command");

    // Test that Debug trait is implemented
    let debug_string = format!("{:?}", command);
    assert!(debug_string.contains("Command"));
    assert!(debug_string.contains("test command"));
}

#[test]
fn test_command_clone_trait() {
    let command = create_test_command("test command");

    // Test that Clone trait is implemented
    let cloned_command = command.clone();
    assert_eq!(cloned_command.command, command.command);
    assert_eq!(cloned_command.host_id, command.host_id);
}

#[test]
fn test_package_ref_debug_and_clone() {
    let package = PackageRef {
        manager: "npm".to_string(),
        name: "lodash".to_string(),
        version: Some("4.17.21".to_string()),
        action: "install".to_string(),
    };

    // Test Debug trait
    let debug_string = format!("{:?}", package);
    assert!(debug_string.contains("PackageRef"));
    assert!(debug_string.contains("lodash"));

    // Test Clone trait
    let cloned_package = package.clone();
    assert_eq!(cloned_package.name, package.name);
    assert_eq!(cloned_package.manager, package.manager);
}

#[test]
fn test_command_with_optional_fields() {
    let mut command = create_test_command("test");

    // Test with None values
    command.exit_code = None;
    command.duration = None;
    command.working_directory = None;

    assert_eq!(command.exit_code, None);
    assert_eq!(command.duration, None);
    assert_eq!(command.working_directory, None);

    // Required fields should still be present
    assert!(!command.command.is_empty());
    assert!(!command.host_id.is_empty());
    assert!(!command.session_id.is_empty());
    assert!(!command.shell.is_empty());
}

#[test]
fn test_package_ref_with_optional_version() {
    let package_with_version = PackageRef {
        manager: "pip".to_string(),
        name: "requests".to_string(),
        version: Some("2.28.0".to_string()),
        action: "install".to_string(),
    };

    let package_without_version = PackageRef {
        manager: "pip".to_string(),
        name: "flask".to_string(),
        version: None,
        action: "install".to_string(),
    };

    assert_eq!(package_with_version.version, Some("2.28.0".to_string()));
    assert_eq!(package_without_version.version, None);
}
