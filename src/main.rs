//! Inspector CLI
//!
//! A CLI tool for inspecting and analyzing web links.

use clap::{App, Arg};
use reqwest::blocking::ClientBuilder;
use scraper::{Html, Selector};
use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use log::{info, error};
use url::Url;
use serde::{Deserialize, Serialize};
use clipboard::{ClipboardContext, ClipboardProvider};
use regex::Regex;
use std::fs;
use serde_yaml::Value;
use thiserror::Error;

/// Configuration structure for the Inspector CLI
#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    url: String,
    ignore: Option<IgnoreConfig>,
    forbidden_domains: Option<Vec<String>>,
    ignored_childs: Option<Vec<String>>,
    timeout: Option<u64>,
    default_output: Option<String>,
}

/// Ignore configuration structure
#[derive(Debug, Serialize, Deserialize, Default)]
struct IgnoreConfig {
    domains: Option<Vec<String>>,
    regex: Option<Vec<String>>,
}

/// Information about a link
#[derive(Debug, Serialize)]
struct LinkInfo {
    url: String,
    status: LinkStatus,
}

/// Status of a link
#[derive(Debug, Serialize)]
enum LinkStatus {
    Valid,
    NotFound,
    Error(String),
    Ignored,
}

#[derive(Error, Debug)]
enum ConfigError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid field type: {0}")]
    InvalidFieldType(String),
}

fn validate_config(config: &Value) -> Result<(), ConfigError> {
    // Check for required fields
    if !config.get("url").is_some() {
        return Err(ConfigError::MissingField("url".to_string()));
    }

    // Validate field types
    if let Some(url) = config.get("url") {
        if !url.is_string() {
            return Err(ConfigError::InvalidFieldType("url must be a string".to_string()));
        }
    }

    if let Some(ignore) = config.get("ignore") {
        if !ignore.is_mapping() {
            return Err(ConfigError::InvalidFieldType("ignore must be an object".to_string()));
        }
        if let Some(domains) = ignore.get("domains") {
            if !domains.is_sequence() {
                return Err(ConfigError::InvalidFieldType("ignore.domains must be an array".to_string()));
            }
        }
        if let Some(regex) = ignore.get("regex") {
            if !regex.is_sequence() {
                return Err(ConfigError::InvalidFieldType("ignore.regex must be an array".to_string()));
            }
        }
    }

    // Add similar checks for other fields...

    Ok(())
}

/// Load configuration from a file or use default settings
fn load_config(config_path: Option<&str>) -> Result<Option<Config>, Box<dyn Error>> {
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
            println!("  url: {}", config.url);
            println!("  ignored_childs: {:?}", config.ignored_childs);
            println!("  forbidden_domains: {:?}", config.forbidden_domains);
            println!("  ignore: {:?}", config.ignore);
            println!("  timeout: {:?}", config.timeout);
            println!("  default_output: {:?}", config.default_output);
            
            Ok(Some(config))
        } else {
            println!("Config file not found at {:?}", config_path);
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Configuration file not found")))
        }
    } else {
        println!("No config file specified, using default configuration");
        Ok(None)
    }
}

/// Main function to run the Inspector CLI
fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("inspector-cli")
        .version("0.1.0")
        .about("Inspects links on a documentation site")
        .arg(Arg::with_name("URL")
            .help("The URL of the documentation site")
            .required_unless("config")
            .index(1))
        .arg(Arg::with_name("output-format")
            .long("output-format")
            .short("o")
            .value_name("FORMAT")
            .help("Output format: json, yaml, txt, or clipboard")
            .takes_value(true))
        .arg(Arg::with_name("output-file")
            .long("output-file")
            .short("f")
            .value_name("FILE")
            .help("Output file name (default: inspect-result-<domain>.<format>)")
            .takes_value(true))
        .arg(Arg::with_name("log-level")
            .long("log-level")
            .short("l")
            .value_name("LEVEL")
            .help("Log level: info, debug, or error")
            .takes_value(true)
            .default_value("info"))
        .arg(Arg::with_name("show-links")
            .long("show-links")
            .short("s")
            .help("Show links in the terminal"))
        .arg(Arg::with_name("detailed")
            .long("detailed")
            .short("d")
            .help("Show detailed information including ignored links"))
        .arg(Arg::with_name("config")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .arg(Arg::with_name("ignore-domains")
            .long("ignore-domains")
            .value_name("DOMAINS")
            .help("Comma-separated list of domains to ignore")
            .takes_value(true))
        .arg(Arg::with_name("ignore-regex")
            .long("ignore-regex")
            .value_name("REGEX")
            .help("Comma-separated list of regex patterns to ignore URLs")
            .takes_value(true))
        .arg(Arg::with_name("forbidden-domains")
            .long("forbidden-domains")
            .value_name("DOMAINS")
            .help("Comma-separated list of forbidden domains")
            .takes_value(true))
        .arg(Arg::with_name("ignored-childs")
            .long("ignored-childs")
            .value_name("PATHS")
            .help("Comma-separated list of child paths to ignore")
            .takes_value(true))
        .arg(Arg::with_name("timeout")
            .long("timeout")
            .value_name("SECONDS")
            .help("Timeout in seconds for each HTTP request")
            .takes_value(true))
        .get_matches();

    let log_level = matches.value_of("log-level").unwrap();

    // Initialize the logger only once
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    // Load configuration if a config file is specified
    let config = load_config(matches.value_of("config"))?;

    // Create a mutable config, either from the loaded config or default
    let mut config = config.unwrap_or_default();

    // Use URL from command line or config file
    let url = matches.value_of("URL").or_else(|| Some(&config.url))
        .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "URL is required when no config file is provided")))?;

    let show_links = matches.is_present("show-links");
    let detailed = matches.is_present("detailed");

    info!("Starting link inspection for {}", url);

    // Override config with command-line arguments
    if let Some(ignore_domains) = matches.value_of("ignore-domains") {
        config.ignore.get_or_insert(IgnoreConfig::default()).domains = Some(ignore_domains.split(',').map(String::from).collect());
    }
    if let Some(ignore_regex) = matches.value_of("ignore-regex") {
        config.ignore.get_or_insert(IgnoreConfig::default()).regex = Some(ignore_regex.split(',').map(String::from).collect());
    }
    if let Some(forbidden_domains) = matches.value_of("forbidden-domains") {
        config.forbidden_domains = Some(forbidden_domains.split(',').map(String::from).collect());
    }
    if let Some(ignored_childs) = matches.value_of("ignored-childs") {
        config.ignored_childs = Some(ignored_childs.split(',').map(String::from).collect());
    }
    if let Some(timeout) = matches.value_of("timeout") {
        config.timeout = Some(timeout.parse().expect("Invalid timeout value"));
    }

    let (links, ignored_links) = inspect_links(url, show_links, &config)?;

    println!("Discovered {} valid links to scan.", links.len());

    let output_format = matches.value_of("output-format").unwrap_or_else(|| {
        config.default_output.as_deref().unwrap_or("json")
    });
    let output_file = matches.value_of("output-file").map(String::from).unwrap_or_else(|| {
        format!(
            "inspect-result-{}.{}",
            Url::parse(url).unwrap().domain().unwrap_or("unknown").to_string(),
            output_format
        )
    });

    match output_format {
        "json" => output_json(&links, &ignored_links, detailed, &output_file)?,
        "yaml" => output_yaml(&links, &ignored_links, detailed, &output_file)?,
        "txt" => output_txt(&links, &output_file)?,
        "clipboard" => output_clipboard(&links)?,
        _ => error!("Invalid output format"),
    }

    if detailed {
        println!("Ignored {} links.", ignored_links.len());
    }

    Ok(())
}

/// Determine if a URL should be ignored based on configuration
fn should_ignore_url(url: &str, config: &Config, base_url: &str) -> bool {
    println!("Checking URL: {}", url);
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(_) => {
            println!("Invalid URL, ignoring: {}", url);
            return true;
        }
    };
    let base_parsed = Url::parse(base_url).unwrap();

    // Always enforce strict mode
    if !url.starts_with(base_url) || parsed_url.domain() != base_parsed.domain() {
        println!("Ignoring due to strict mode: {}", url);
        return true;
    }

    let domain = parsed_url.domain().unwrap_or("");
    let path = parsed_url.path();

    println!("URL domain: {}, path: {}", domain, path);

    if let Some(ignore) = &config.ignore {
        if let Some(domains) = &ignore.domains {
            if domains.iter().any(|ignored| domain.ends_with(ignored)) {
                println!("Ignoring due to ignore domains: {}", url);
                return true;
            }
        }

        if let Some(regex_patterns) = &ignore.regex {
            for pattern in regex_patterns {
                if let Ok(regex) = Regex::new(pattern) {
                    if regex.is_match(url) {
                        println!("Ignoring due to ignore regex: {}", url);
                        return true;
                    }
                }
            }
        }
    }

    if let Some(forbidden_domains) = &config.forbidden_domains {
        if forbidden_domains.iter().any(|forbidden| domain.ends_with(forbidden)) {
            println!("Ignoring due to forbidden domains: {}", url);
            return true;
        }
    }

    if let Some(ignored_childs) = &config.ignored_childs {
        for ignored_child in ignored_childs {
            let full_ignored_path = if base_parsed.path().ends_with('/') {
                format!("{}{}", base_parsed.path(), ignored_child.trim_start_matches('/'))
            } else {
                format!("{}/{}", base_parsed.path(), ignored_child.trim_start_matches('/'))
            };
            println!("Checking against ignored child path: {}", full_ignored_path);
            if url.starts_with(&(base_parsed.origin().ascii_serialization() + &full_ignored_path)) {
                println!("Ignoring URL due to ignored_childs: {}", url);
                return true;
            }
        }
    }

    false
}

/// Inspect links starting from a given URL
fn inspect_links(base_url: &str, show_links: bool, config: &Config) -> Result<(Vec<LinkInfo>, Vec<LinkInfo>), Box<dyn Error>> {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(config.timeout.unwrap_or(30)))
        .build()?;
    let mut links = Vec::new();
    let mut ignored_links = Vec::new();
    let mut visited = HashSet::new();
    let mut to_visit = vec![base_url.to_string()];

    while let Some(current_url) = to_visit.pop() {
        if visited.contains(&current_url) {
            continue;
        }

        visited.insert(current_url.clone());

        if should_ignore_url(&current_url, config, base_url) {
            ignored_links.push(LinkInfo {
                url: current_url,
                status: LinkStatus::Ignored,
            });
            continue;
        }

        let response = match client.get(&current_url).send() {
            Ok(resp) => resp,
            Err(e) => {
                links.push(LinkInfo {
                    url: current_url,
                    status: LinkStatus::Error(e.to_string()),
                });
                continue;
            }
        };

        let status = response.status();
        let link_status = if status.is_success() {
            LinkStatus::Valid
        } else if status.as_u16() == 404 {
            LinkStatus::NotFound
        } else {
            LinkStatus::Error(status.to_string())
        };

        let link_info = LinkInfo {
            url: current_url.clone(),
            status: link_status,
        };

        if show_links {
            println!("Inspected: {:?}", link_info);
        }

        links.push(link_info);

        if status.is_success() {
            let html = response.text()?;
            let document = Html::parse_document(&html);
            let selector = Selector::parse("a").unwrap();

            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if let Ok(absolute_url) = Url::parse(&current_url).and_then(|base| base.join(href)) {
                        to_visit.push(absolute_url.into());
                    }
                }
            }
        }
    }

    Ok((links, ignored_links))
}

/// Output results in JSON format
fn output_json(links: &[LinkInfo], ignored_links: &[LinkInfo], detailed: bool, output_file: &str) -> Result<(), Box<dyn Error>> {
    let mut output = HashMap::new();
    output.insert("scanned_links", links);
    if detailed {
        output.insert("ignored_links", ignored_links);
    }
    let json = serde_json::to_string_pretty(&output)?;
    let mut file = File::create(output_file)?;
    file.write_all(json.as_bytes())?;
    println!("JSON output written to {}", output_file);
    Ok(())
}

/// Output results in YAML format
fn output_yaml(links: &[LinkInfo], ignored_links: &[LinkInfo], detailed: bool, output_file: &str) -> Result<(), Box<dyn Error>> {
    let mut output = HashMap::new();
    output.insert("scanned_links", links);
    if detailed {
        output.insert("ignored_links", ignored_links);
    }
    let yaml = serde_yaml::to_string(&output)?;
    let mut file = File::create(output_file)?;
    file.write_all(yaml.as_bytes())?;
    println!("YAML output written to {}", output_file);
    Ok(())
}

/// Output results in TXT format
fn output_txt(links: &[LinkInfo], output_file: &str) -> Result<(), Box<dyn Error>> {
    let content: String = links.iter().map(|link| format!("{}\n", link.url)).collect();
    let mut file = File::create(output_file)?;
    file.write_all(content.as_bytes())?;
    println!("TXT output written to {}", output_file);
    Ok(())
}

/// Output results to clipboard
fn output_clipboard(links: &[LinkInfo]) -> Result<(), Box<dyn Error>> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    let content: String = links.iter().map(|link| format!("{}\n", link.url)).collect();
    ctx.set_contents(content)?;
    println!("Links copied to clipboard");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_ignore_url() {
        let base_url = "https://example.com";
        let config = Config {
            url: base_url.to_string(),
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
        assert!(should_ignore_url("https://ignored.com/page", &config, base_url));

        // Test ignoring based on regex
        assert!(should_ignore_url("https://example.com/document.pdf", &config, base_url));

        // Test forbidden domain
        assert!(should_ignore_url("https://forbidden.com/page", &config, base_url));

        // Test ignored child path
        assert!(should_ignore_url("https://example.com/ignore-me/page", &config, base_url));

        // Test valid URL (should not be ignored)
        assert!(!should_ignore_url("https://example.com/valid-page", &config, base_url));

        // Test strict mode (different domain)
        assert!(should_ignore_url("https://different.com/page", &config, base_url));
    }

    #[test]
    fn test_load_config() {
        use std::fs;
        use tempfile::NamedTempFile;

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
        let config = load_config(Some(temp_file.path().to_str().unwrap())).unwrap().unwrap();

        assert_eq!(config.url, "https://example.com");
        assert_eq!(config.ignore.unwrap().domains.unwrap(), vec!["ignored.com"]);
        assert_eq!(config.ignore.unwrap().regex.unwrap(), vec![".*\\.pdf$"]);
        assert_eq!(config.forbidden_domains.unwrap(), vec!["forbidden.com"]);
        assert_eq!(config.ignored_childs.unwrap(), vec!["ignore-me"]);
        assert_eq!(config.timeout.unwrap(), 30);
        assert_eq!(config.default_output.unwrap(), "json");

        // Test loading non-existent config
        assert!(load_config(Some("non_existent_config.yaml")).is_err());
    }

    #[test]
    fn test_validate_config() {
        use serde_yaml::Value;

        // Valid config
        let valid_config = serde_yaml::from_str(r#"
        url: https://example.com
        ignore:
          domains:
            - ignored.com
          regex:
            - ".*\\.pdf$"
        "#).unwrap();

        assert!(validate_config(&valid_config).is_ok());

        // Invalid config (missing url)
        let invalid_config = serde_yaml::from_str(r#"
        ignore:
          domains:
            - ignored.com
        "#).unwrap();

        assert!(matches!(validate_config(&invalid_config), Err(ConfigError::MissingField(_))));

        // Invalid config (wrong type for url)
        let invalid_config = serde_yaml::from_str(r#"
        url: 123
        "#).unwrap();

        assert!(matches!(validate_config(&invalid_config), Err(ConfigError::InvalidFieldType(_))));
    }
}