use super::*;
use crate::config::{validate_config, Config, ConfigError, IgnoreConfig};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_should_ignore_url() {
    let base_url = "https://example.com";
    let config = Config {
        url: Some(base_url.to_string()),
        ignore: Some(IgnoreConfig {
            domains: Some(vec!["ignored.com".to_string()]),
            regex: Some(vec![".*\\.pdf$".to_string()]),
        }),
        forbidden_domains: Some(vec!["forbidden.com".to_string()]),
        ignored_childs: Some(vec!["ignore-me".to_string()]),
        timeout: Some(30),
        default_output: None,
    };

    // Test ignoring based on domain
    assert!(should_ignore_url(
        "https://ignored.com/page",
        &config,
        base_url
    ));

    // Test ignoring based on regex
    assert!(should_ignore_url(
        "https://example.com/document.pdf",
        &config,
        base_url
    ));

    // Test forbidden domain
    assert!(should_ignore_url(
        "https://forbidden.com/page",
        &config,
        base_url
    ));

    // Test ignored child path
    assert!(should_ignore_url(
        "https://example.com/ignore-me/page",
        &config,
        base_url
    ));

    // Test valid URL (should not be ignored)
    assert!(!should_ignore_url(
        "https://example.com/valid-page",
        &config,
        base_url
    ));

    // Test strict mode (different domain)
    assert!(should_ignore_url(
        "https://different.com/page",
        &config,
        base_url
    ));
}

#[test]
fn test_load_config() {
    // Create a temporary config file
    let config_content = r#"
    url: https://example.com
    ignore:
      domains:
        - ignored.com
      regex:
        - ".*\\.pdf$"
    forbidden_domains:
      - forbidden.com
    ignored_childs:
      - ignore-me
    timeout: 30
    default_output: json
    "#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    // Test loading the config
    let config = load_config(Some(temp_file.path().to_str().unwrap()))
        .unwrap()
        .unwrap();

    assert_eq!(config.url, Some("https://example.com".to_string()));
    assert_eq!(
        config.ignore.as_ref().unwrap().domains.as_ref().unwrap(),
        &vec!["ignored.com"]
    );
    assert_eq!(
        config.ignore.as_ref().unwrap().regex.as_ref().unwrap(),
        &vec![".*\\.pdf$"]
    );
    assert_eq!(
        config.forbidden_domains.as_ref().unwrap(),
        &vec!["forbidden.com"]
    );
    assert_eq!(config.ignored_childs.as_ref().unwrap(), &vec!["ignore-me"]);
    assert_eq!(config.timeout.unwrap(), 30);
    assert_eq!(config.default_output.as_ref().unwrap(), "json");

    // Test loading non-existent config
    assert!(load_config(Some("non_existent_config.yaml")).is_err());
}

#[test]
fn test_validate_config() {
    // Valid config
    let valid_config = serde_yaml::from_str(
        r#"
    url: https://example.com
    ignore:
      domains:
        - ignored.com
      regex:
        - ".*\\.pdf$"
    "#,
    )
    .unwrap();

    assert!(validate_config(&valid_config).is_ok());

    // Invalid config (missing url)
    let invalid_config = serde_yaml::from_str(
        r#"
    ignore:
      domains:
        - ignored.com
    "#,
    )
    .unwrap();

    assert!(matches!(
        validate_config(&invalid_config),
        Err(ConfigError::MissingField(_))
    ));

    // Invalid config (wrong type for url)
    let invalid_config = serde_yaml::from_str(
        r#"
    url: 123
    "#,
    )
    .unwrap();

    assert!(matches!(
        validate_config(&invalid_config),
        Err(ConfigError::InvalidFieldType(_))
    ));
}
