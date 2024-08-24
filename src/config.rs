use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Configuration structure for the Inspector CLI
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub url: Option<String>,
    pub ignore: Option<IgnoreConfig>,
    pub forbidden_domains: Option<Vec<String>>,
    pub ignored_childs: Option<Vec<String>>,
    pub timeout: Option<u64>,
    pub default_output: Option<String>,
}

/// Ignore configuration structure
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct IgnoreConfig {
    pub domains: Option<Vec<String>>,
    pub regex: Option<Vec<String>>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid field type: {0}")]
    InvalidFieldType(String),
}

/// Load configuration from a file or use default settings
pub fn load_config(config_path: Option<&str>) -> Result<Option<Config>, Box<dyn Error>> {
    if let Some(path) = config_path {
        let config_path = PathBuf::from(path);
        println!("Attempting to load config from: {:?}", config_path);

        if config_path.exists() {
            println!("Config file found, reading contents...");
            let config_str = fs::read_to_string(&config_path)?;
            println!("Config file contents:\n{}", config_str);

            let config_value: Value = serde_yaml::from_str(&config_str)?;
            validate_config(&config_value)?;

            let config: Config = serde_yaml::from_str(&config_str)?;

            println!("Loaded configuration:");
            println!("  url: {:?}", config.url);
            println!("  ignored_childs: {:?}", config.ignored_childs);
            println!("  forbidden_domains: {:?}", config.forbidden_domains);
            println!("  ignore: {:?}", config.ignore);
            println!("  timeout: {:?}", config.timeout);
            println!("  default_output: {:?}", config.default_output);

            Ok(Some(config))
        } else {
            println!("Config file not found at {:?}", config_path);
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Configuration file not found",
            )))
        }
    } else {
        println!("No config file specified, using default configuration");
        Ok(None)
    }
}

pub fn validate_config(config: &Value) -> Result<(), ConfigError> {
    // Check for required fields
    if config.get("url").is_none() {
        return Err(ConfigError::MissingField("url".to_string()));
    }

    // Validate field types
    if let Some(url) = config.get("url") {
        if !url.is_string() {
            return Err(ConfigError::InvalidFieldType(
                "url must be a string".to_string(),
            ));
        }
    }

    if let Some(ignore) = config.get("ignore") {
        if !ignore.is_mapping() {
            return Err(ConfigError::InvalidFieldType(
                "ignore must be an object".to_string(),
            ));
        }
        if let Some(domains) = ignore.get("domains") {
            if !domains.is_sequence() {
                return Err(ConfigError::InvalidFieldType(
                    "ignore.domains must be an array".to_string(),
                ));
            }
        }
        if let Some(regex) = ignore.get("regex") {
            if !regex.is_sequence() {
                return Err(ConfigError::InvalidFieldType(
                    "ignore.regex must be an array".to_string(),
                ));
            }
        }
    }

    // Add similar checks for other fields...

    Ok(())
}
