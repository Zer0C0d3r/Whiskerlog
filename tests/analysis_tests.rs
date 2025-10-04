use chrono::{DateTime, TimeZone, Utc};
use whiskerlog::analysis::heatmap::*;
use whiskerlog::analysis::package_tracker::*;
use whiskerlog::*;

fn create_test_command(cmd: &str, timestamp: DateTime<Utc>, packages: Vec<PackageRef>) -> Command {
    Command {
        id: None,
        command: cmd.to_string(),
        timestamp,
        exit_code: Some(0),
        duration: Some(100),
        working_directory: Some("/test".to_string()),
        host_id: "test-host".to_string(),
        session_id: "test-session".to_string(),
        shell: "bash".to_string(),
        packages_used: packages,
        network_endpoints: vec![],
        is_dangerous: false,
        danger_score: 0.0,
        danger_reasons: vec![],
        is_experiment: false,
        experiment_tags: vec![],
    }
}

fn create_test_package(
    manager: &str,
    name: &str,
    action: &str,
    version: Option<&str>,
) -> PackageRef {
    PackageRef {
        manager: manager.to_string(),
        name: name.to_string(),
        version: version.map(|v| v.to_string()),
        action: action.to_string(),
    }
}

#[test]
fn test_package_tracker_basic_analysis() {
    let tracker = PackageTracker::new();

    let commands = vec![
        create_test_command(
            "npm install react",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            vec![create_test_package(
                "npm",
                "react",
                "install",
                Some("18.2.0"),
            )],
        ),
        create_test_command(
            "npm install lodash",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 5, 0).unwrap(),
            vec![create_test_package(
                "npm",
                "lodash",
                "install",
                Some("4.17.21"),
            )],
        ),
        create_test_command(
            "pip install requests",
            Utc.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap(),
            vec![create_test_package(
                "pip",
                "requests",
                "install",
                Some("2.28.1"),
            )],
        ),
    ];

    let analysis = tracker.analyze_package_usage(&commands);

    assert_eq!(analysis.total_package_operations, 3);
    assert_eq!(analysis.managers_used.len(), 2);

    // Check npm manager stats
    let npm_manager = analysis
        .managers_used
        .iter()
        .find(|m| m.manager == "npm")
        .expect("npm manager should be found");
    assert_eq!(npm_manager.installs, 2);
    assert_eq!(npm_manager.removes, 0);
    assert_eq!(npm_manager.total_operations, 2);

    // Check pip manager stats
    let pip_manager = analysis
        .managers_used
        .iter()
        .find(|m| m.manager == "pip")
        .expect("pip manager should be found");
    assert_eq!(pip_manager.installs, 1);
    assert_eq!(pip_manager.removes, 0);
    assert_eq!(pip_manager.total_operations, 1);
}

#[test]
fn test_package_tracker_version_conflicts() {
    let tracker = PackageTracker::new();

    let commands = vec![
        create_test_command(
            "npm install react@18.2.0",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            vec![create_test_package(
                "npm",
                "react",
                "install",
                Some("18.2.0"),
            )],
        ),
        create_test_command(
            "npm install react@17.0.0",
            Utc.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap(),
            vec![create_test_package(
                "npm",
                "react",
                "install",
                Some("17.0.0"),
            )],
        ),
    ];

    let analysis = tracker.analyze_package_usage(&commands);

    assert_eq!(analysis.version_conflicts.len(), 1);
    let conflict = &analysis.version_conflicts[0];
    assert_eq!(conflict.package, "react");
    assert_eq!(conflict.manager, "npm");
}

#[test]
fn test_package_tracker_trends() {
    let tracker = PackageTracker::new();

    let commands = vec![
        create_test_command(
            "npm install react",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            vec![create_test_package("npm", "react", "install", None)],
        ),
        create_test_command(
            "npm install react",
            Utc.with_ymd_and_hms(2024, 1, 2, 10, 0, 0).unwrap(),
            vec![create_test_package("npm", "react", "install", None)],
        ),
        create_test_command(
            "npm install react",
            Utc.with_ymd_and_hms(2024, 1, 3, 10, 0, 0).unwrap(),
            vec![create_test_package("npm", "react", "install", None)],
        ),
    ];

    let analysis = tracker.analyze_package_usage(&commands);

    assert!(!analysis.package_trends.is_empty());
    let trend = analysis
        .package_trends
        .iter()
        .find(|t| t.package == "react")
        .expect("react trend should be found");
    assert_eq!(trend.frequency, 3);
    assert_eq!(trend.manager, "npm");
}

#[test]
fn test_package_tracker_health_score() {
    let tracker = PackageTracker::new();

    // Test with healthy package usage (more installs than removes)
    let healthy_commands = vec![
        create_test_command(
            "npm install react",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            vec![create_test_package("npm", "react", "install", None)],
        ),
        create_test_command(
            "npm install lodash",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 5, 0).unwrap(),
            vec![create_test_package("npm", "lodash", "install", None)],
        ),
    ];

    let healthy_analysis = tracker.analyze_package_usage(&healthy_commands);
    let healthy_score = tracker.calculate_package_health_score(&healthy_analysis);

    // Test with unhealthy package usage (many removes)
    let unhealthy_commands = vec![
        create_test_command(
            "npm install react",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            vec![create_test_package("npm", "react", "install", None)],
        ),
        create_test_command(
            "npm uninstall react",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 5, 0).unwrap(),
            vec![create_test_package("npm", "react", "remove", None)],
        ),
        create_test_command(
            "npm install lodash",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 10, 0).unwrap(),
            vec![create_test_package("npm", "lodash", "install", None)],
        ),
        create_test_command(
            "npm uninstall lodash",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 15, 0).unwrap(),
            vec![create_test_package("npm", "lodash", "remove", None)],
        ),
    ];

    let unhealthy_analysis = tracker.analyze_package_usage(&unhealthy_commands);
    let unhealthy_score = tracker.calculate_package_health_score(&unhealthy_analysis);

    // Both scores should be valid (between 0.0 and 1.0)
    assert!((0.0..=1.0).contains(&healthy_score));
    assert!((0.0..=1.0).contains(&unhealthy_score));

    // The healthy score should be at least as good as the unhealthy score
    // (or they might be equal if the algorithm doesn't penalize the pattern we tested)
    assert!(healthy_score >= unhealthy_score);
}

#[test]
fn test_package_tracker_recommendations() {
    let tracker = PackageTracker::new();

    let commands = vec![
        // High removal rate scenario
        create_test_command(
            "npm install package1",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            vec![create_test_package("npm", "package1", "install", None)],
        ),
        create_test_command(
            "npm uninstall package1",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 5, 0).unwrap(),
            vec![create_test_package("npm", "package1", "remove", None)],
        ),
        create_test_command(
            "npm install package2",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 10, 0).unwrap(),
            vec![create_test_package("npm", "package2", "install", None)],
        ),
        create_test_command(
            "npm uninstall package2",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 15, 0).unwrap(),
            vec![create_test_package("npm", "package2", "remove", None)],
        ),
    ];

    let analysis = tracker.analyze_package_usage(&commands);

    assert!(!analysis.recommendations.is_empty());

    // Should have recommendation about high removal rate
    let has_removal_recommendation = analysis
        .recommendations
        .iter()
        .any(|r| r.contains("removal rate") || r.contains("testing before installing"));
    assert!(has_removal_recommendation);
}

#[test]
fn test_heatmap_analyzer_basic() {
    let analyzer = HeatmapAnalyzer::new();

    let commands = vec![
        create_test_command(
            "git status",
            Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap(),
            vec![],
        ),
        create_test_command(
            "git commit",
            Utc.with_ymd_and_hms(2024, 1, 1, 14, 30, 0).unwrap(),
            vec![],
        ),
        create_test_command(
            "git push",
            Utc.with_ymd_and_hms(2024, 1, 1, 17, 45, 0).unwrap(),
            vec![],
        ),
    ];

    let analysis = analyzer.analyze_work_patterns(&commands);

    // Check that we have reasonable ratios
    assert!(analysis.weekday_ratio >= 0.0 && analysis.weekday_ratio <= 1.0);
    assert!(analysis.weekend_ratio >= 0.0 && analysis.weekend_ratio <= 1.0);
    assert!(analysis.work_hours_ratio >= 0.0 && analysis.work_hours_ratio <= 1.0);
}

#[test]
fn test_empty_command_analysis() {
    let tracker = PackageTracker::new();
    let analyzer = HeatmapAnalyzer::new();

    let empty_commands: Vec<Command> = vec![];

    let package_analysis = tracker.analyze_package_usage(&empty_commands);
    assert_eq!(package_analysis.total_package_operations, 0);
    assert!(package_analysis.managers_used.is_empty());
    assert!(package_analysis.package_trends.is_empty());
    assert!(package_analysis.version_conflicts.is_empty());

    let heatmap_analysis = analyzer.analyze_work_patterns(&empty_commands);
    assert_eq!(heatmap_analysis.weekday_ratio, 0.0);
    assert_eq!(heatmap_analysis.weekend_ratio, 0.0);
}

#[test]
fn test_package_tracker_creation() {
    let _tracker = PackageTracker::new();

    // Test that tracker can be created
    // If we get here, creation succeeded
}

#[test]
fn test_heatmap_analyzer_creation() {
    let _analyzer = HeatmapAnalyzer::new();

    // Test that analyzer can be created
    // If we get here, creation succeeded
}

#[test]
fn test_package_analysis_with_no_packages() {
    let tracker = PackageTracker::new();

    let commands = vec![
        create_test_command(
            "ls -la",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            vec![], // No packages
        ),
        create_test_command(
            "git status",
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 5, 0).unwrap(),
            vec![], // No packages
        ),
    ];

    let analysis = tracker.analyze_package_usage(&commands);

    assert_eq!(analysis.total_package_operations, 0);
    assert!(analysis.managers_used.is_empty());
    assert!(analysis.package_trends.is_empty());
    assert!(analysis.version_conflicts.is_empty());
}

#[test]
fn test_heatmap_analysis_with_multiple_days() {
    let analyzer = HeatmapAnalyzer::new();

    let commands = vec![
        create_test_command(
            "git status",
            Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap(),
            vec![],
        ),
        create_test_command(
            "git commit",
            Utc.with_ymd_and_hms(2024, 1, 2, 14, 30, 0).unwrap(),
            vec![],
        ),
        create_test_command(
            "git push",
            Utc.with_ymd_and_hms(2024, 1, 3, 17, 45, 0).unwrap(),
            vec![],
        ),
    ];

    let analysis = analyzer.analyze_work_patterns(&commands);

    // Check that we have reasonable ratios for multiple days
    assert!(analysis.weekday_ratio >= 0.0 && analysis.weekday_ratio <= 1.0);
    assert!(analysis.weekend_ratio >= 0.0 && analysis.weekend_ratio <= 1.0);
}
