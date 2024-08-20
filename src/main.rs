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

/// Configuration structure for the Inspector CLI
#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
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

/// Load configuration from a file or use default settings
fn load_config(config_path: Option<&str>) -> Result<Config, Box<dyn Error>> {
    let config_path = config_path
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".inspector-config.yml"));

    if config_path.exists() {
        let config_str = fs::read_to_string(&config_path)?;
        let config: Config = serde_yaml::from_str(&config_str)
            .map_err(|e| Box::<dyn Error>::from(format!("Failed to parse config: {}", e)))?;
        
        println!("Loaded configuration:");
        println!("  ignored_childs: {:?}", config.ignored_childs);
        println!("  forbidden_domains: {:?}", config.forbidden_domains);
        println!("  ignore: {:?}", config.ignore);
        println!("  timeout: {:?}", config.timeout);
        println!("  default_output: {:?}", config.default_output);
        
        Ok(config)
    } else {
        println!("Config file not found at {:?}, using default configuration", config_path);
        Ok(Config::default())
    }
}

/// Main function to run the Inspector CLI
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let matches = App::new("inspector-cli")
        .version("0.1.0")
        .about("Inspects links on a documentation site")
        .arg(Arg::with_name("URL")
            .help("The URL of the documentation site")
            .required(true)
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
        .arg(Arg::with_name("strict")
            .long("strict")
            .help("Only scan links that are under or children of the passed URL"))
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

    let url = matches.value_of("URL").unwrap();
    let log_level = matches.value_of("log-level").unwrap();
    let show_links = matches.is_present("show-links");
    let detailed = matches.is_present("detailed");
    let strict = matches.is_present("strict");

    // Set log level
    std::env::set_var("RUST_LOG", log_level);

    info!("Starting link inspection for {}", url);

    // Load configuration
    let mut config = load_config(matches.value_of("config"))?;

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

    let (links, ignored_links) = inspect_links(url, show_links, &config, strict)?;

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
fn should_ignore_url(url: &str, config: &Config, base_url: &str, strict: bool) -> bool {
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(_) => return true, // Ignore invalid URLs
    };
    let domain = parsed_url.domain().unwrap_or("");
    let path = parsed_url.path();

    if strict {
        let base_parsed = Url::parse(base_url).unwrap();
        if !url.starts_with(base_url) && parsed_url.domain() != base_parsed.domain() {
            return true;
        }
    }

    if let Some(ignore) = &config.ignore {
        if let Some(domains) = &ignore.domains {
            if domains.iter().any(|ignored| domain.ends_with(ignored)) {
                return true;
            }
        }

        if let Some(regex_patterns) = &ignore.regex {
            for pattern in regex_patterns {
                if let Ok(regex) = Regex::new(pattern) {
                    if regex.is_match(url) {
                        return true;
                    }
                }
            }
        }
    }

    if let Some(forbidden_domains) = &config.forbidden_domains {
        if forbidden_domains.iter().any(|forbidden| domain.ends_with(forbidden)) {
            return true;
        }
    }

    if let Some(ignored_childs) = &config.ignored_childs {
        let base_parsed = Url::parse(base_url).unwrap();
        for ignored_child in ignored_childs {
            let full_ignored_path = if base_parsed.path().ends_with('/') {
                format!("{}{}", base_parsed.path(), ignored_child.trim_start_matches('/'))
            } else {
                format!("{}/{}", base_parsed.path(), ignored_child.trim_start_matches('/'))
            };
            if path.starts_with(&full_ignored_path) {
                println!("Ignoring URL due to ignored_childs: {}", url); // Debug print
                return true;
            }
        }
    }

    false
}

/// Inspect links starting from a given URL
fn inspect_links(url: &str, show_links: bool, config: &Config, strict: bool) -> Result<(Vec<LinkInfo>, Vec<LinkInfo>), Box<dyn Error>> {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(config.timeout.unwrap_or(30)))
        .build()?;
    let mut links = Vec::new();
    let mut ignored_links = Vec::new();
    let mut visited = HashSet::new();
    let mut to_visit = vec![url.to_string()];

    while let Some(current_url) = to_visit.pop() {
        if visited.contains(&current_url) {
            continue;
        }

        visited.insert(current_url.clone());

        if should_ignore_url(&current_url, config, url, strict) {
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
        // Add some unit tests for the should_ignore_url function
    }

    // Add more tests as needed
}