use whiskerlog::*;

#[test]
fn test_config_default_values() {
    let config = Config::default();

    assert!(config.redaction_enabled);
    assert!(config.auto_import);
    assert_eq!(config.danger_threshold, 0.7);
    assert!(config.experiment_detection);
    assert!(!config.history_paths.is_empty());
}

#[test]
fn test_config_serialization() {
    let config = Config {
        database_path: "/tmp/test.db".into(),
        history_paths: vec![
            "/home/user/.bash_history".into(),
            "/home/user/.zsh_history".into(),
        ],
        redaction_enabled: false,
        auto_import: true,
        danger_threshold: 0.5,
        experiment_detection: false,
    };

    let toml_string = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&toml_string).unwrap();

    assert_eq!(config.database_path, deserialized.database_path);
    assert_eq!(config.history_paths, deserialized.history_paths);
    assert_eq!(config.redaction_enabled, deserialized.redaction_enabled);
    assert_eq!(config.auto_import, deserialized.auto_import);
    assert_eq!(config.danger_threshold, deserialized.danger_threshold);
    assert_eq!(
        config.experiment_detection,
        deserialized.experiment_detection
    );
}

#[test]
fn test_config_toml_format() {
    let config = Config {
        database_path: "/tmp/whiskerlog.db".into(),
        history_paths: vec!["/home/user/.bash_history".into()],
        redaction_enabled: true,
        auto_import: false,
        danger_threshold: 0.8,
        experiment_detection: true,
    };

    let toml_string = toml::to_string_pretty(&config).unwrap();

    assert!(toml_string.contains("database_path"));
    assert!(toml_string.contains("history_paths"));
    assert!(toml_string.contains("redaction_enabled"));
    assert!(toml_string.contains("auto_import"));
    assert!(toml_string.contains("danger_threshold"));
    assert!(toml_string.contains("experiment_detection"));

    // Check specific values
    assert!(toml_string.contains("redaction_enabled = true"));
    assert!(toml_string.contains("auto_import = false"));
    assert!(toml_string.contains("danger_threshold = 0.8"));
    assert!(toml_string.contains("experiment_detection = true"));
}

#[test]
fn test_config_with_multiple_history_paths() {
    let config = Config {
        database_path: "/var/lib/whiskerlog/history.db".into(),
        history_paths: vec![
            "/home/user/.bash_history".into(),
            "/home/user/.zsh_history".into(),
            "/home/user/.local/share/fish/fish_history".into(),
            "/root/.bash_history".into(),
        ],
        redaction_enabled: true,
        auto_import: true,
        danger_threshold: 0.6,
        experiment_detection: true,
    };

    let toml_string = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&toml_string).unwrap();

    assert_eq!(deserialized.history_paths.len(), 4);
    assert!(deserialized
        .history_paths
        .contains(&"/home/user/.bash_history".into()));
    assert!(deserialized
        .history_paths
        .contains(&"/home/user/.zsh_history".into()));
    assert!(deserialized
        .history_paths
        .contains(&"/home/user/.local/share/fish/fish_history".into()));
    assert!(deserialized
        .history_paths
        .contains(&"/root/.bash_history".into()));
}

#[test]
fn test_config_danger_threshold_bounds() {
    // Test minimum threshold
    let config_min = Config {
        database_path: "/tmp/test.db".into(),
        history_paths: vec![],
        redaction_enabled: true,
        auto_import: true,
        danger_threshold: 0.0,
        experiment_detection: true,
    };

    let toml_string = toml::to_string(&config_min).unwrap();
    let deserialized: Config = toml::from_str(&toml_string).unwrap();
    assert_eq!(deserialized.danger_threshold, 0.0);

    // Test maximum threshold
    let config_max = Config {
        database_path: "/tmp/test.db".into(),
        history_paths: vec![],
        redaction_enabled: true,
        auto_import: true,
        danger_threshold: 1.0,
        experiment_detection: true,
    };

    let toml_string = toml::to_string(&config_max).unwrap();
    let deserialized: Config = toml::from_str(&toml_string).unwrap();
    assert_eq!(deserialized.danger_threshold, 1.0);
}

#[test]
fn test_config_boolean_combinations() {
    let test_cases = vec![
        (true, true, true),
        (true, true, false),
        (true, false, true),
        (true, false, false),
        (false, true, true),
        (false, true, false),
        (false, false, true),
        (false, false, false),
    ];

    for (redaction, auto_import, experiment) in test_cases {
        let config = Config {
            database_path: "/tmp/test.db".into(),
            history_paths: vec![],
            redaction_enabled: redaction,
            auto_import,
            danger_threshold: 0.5,
            experiment_detection: experiment,
        };

        let toml_string = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_string).unwrap();

        assert_eq!(deserialized.redaction_enabled, redaction);
        assert_eq!(deserialized.auto_import, auto_import);
        assert_eq!(deserialized.experiment_detection, experiment);
    }
}

#[test]
fn test_config_empty_history_paths() {
    let config = Config {
        database_path: "/tmp/test.db".into(),
        history_paths: vec![],
        redaction_enabled: true,
        auto_import: true,
        danger_threshold: 0.7,
        experiment_detection: true,
    };

    let toml_string = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&toml_string).unwrap();

    assert!(deserialized.history_paths.is_empty());
}

#[test]
fn test_config_path_with_spaces_and_special_chars() {
    let config = Config {
        database_path: "/tmp/whisker log & data/test.db".into(),
        history_paths: vec![
            "/home/user name/.bash_history".into(),
            "/home/user-name/.zsh_history".into(),
            "/home/user.name/.fish_history".into(),
        ],
        redaction_enabled: true,
        auto_import: true,
        danger_threshold: 0.7,
        experiment_detection: true,
    };

    let toml_string = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&toml_string).unwrap();

    assert_eq!(
        deserialized.database_path.to_string_lossy(),
        "/tmp/whisker log & data/test.db"
    );
    assert!(deserialized
        .history_paths
        .iter()
        .any(|p| p.to_string_lossy().contains("user name")));
    assert!(deserialized
        .history_paths
        .iter()
        .any(|p| p.to_string_lossy().contains("user-name")));
    assert!(deserialized
        .history_paths
        .iter()
        .any(|p| p.to_string_lossy().contains("user.name")));
}

#[test]
fn test_config_from_invalid_toml() {
    let invalid_toml = r#"
        database_path = "/tmp/test.db"
        history_paths = ["/home/user/.bash_history"]
        redaction_enabled = "not_a_boolean"
        auto_import = true
        danger_threshold = 0.7
        experiment_detection = true
    "#;

    let result: Result<Config, _> = toml::from_str(invalid_toml);
    assert!(result.is_err());
}

#[test]
fn test_config_missing_required_fields() {
    let incomplete_toml = r#"
        database_path = "/tmp/test.db"
        redaction_enabled = true
        # Missing other required fields
    "#;

    let result: Result<Config, _> = toml::from_str(incomplete_toml);
    assert!(result.is_err());
}

#[test]
fn test_config_extra_fields_ignored() {
    let toml_with_extra = r#"
        database_path = "/tmp/test.db"
        history_paths = ["/home/user/.bash_history"]
        redaction_enabled = true
        auto_import = true
        danger_threshold = 0.7
        experiment_detection = true
        extra_field = "this should be ignored"
        another_extra = 42
    "#;

    let result: Result<Config, _> = toml::from_str(toml_with_extra);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.database_path.to_string_lossy(), "/tmp/test.db");
    assert_eq!(config.danger_threshold, 0.7);
}

#[test]
fn test_config_round_trip_precision() {
    let original_config = Config {
        database_path: "/precise/path/test.db".into(),
        history_paths: vec![
            "/home/user/.bash_history".into(),
            "/home/user/.zsh_history".into(),
        ],
        redaction_enabled: true,
        auto_import: false,
        danger_threshold: 0.123_456_79,
        experiment_detection: true,
    };

    let toml_string = toml::to_string(&original_config).unwrap();
    let deserialized: Config = toml::from_str(&toml_string).unwrap();

    // Check that floating point precision is maintained reasonably
    assert!((deserialized.danger_threshold - 0.123_456_79).abs() < 0.000001);
}

#[test]
fn test_config_display_debug() {
    let config = Config::default();

    // Test that Debug trait is implemented
    let debug_string = format!("{:?}", config);
    assert!(debug_string.contains("Config"));
    assert!(debug_string.contains("database_path"));
    assert!(debug_string.contains("redaction_enabled"));
}

// Note: The actual file I/O tests for Config::load_or_create() and Config::save()
// would require mocking the directories or using integration tests with proper
// file system setup. These tests focus on the serialization/deserialization logic.
