//! Inspector CLI
//!
//! A CLI tool for inspecting and analyzing web links.

use clap::{App, Arg};
use log::{error, info};
use regex::Regex;
use reqwest::blocking::ClientBuilder;
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;
use url::Url;

mod config;
mod link;
mod output;

use crate::config::{load_config, Config, IgnoreConfig};
use crate::link::{extract_links_from_html, inspect_single_link, LinkInfo, LinkStatus};
use crate::output::{output_clipboard, output_json, output_txt, output_yaml};

/// Main function to run the Inspector CLI
fn main() -> Result<(), Box<dyn Error>> {
    let matches = create_cli_app().get_matches();
    setup_logger(&matches);

    let config = load_and_merge_config(&matches)?;
    let url = get_url(&matches, &config)?;
    let show_links = matches.is_present("show-links");
    let detailed = matches.is_present("detailed");

    info!("Starting link inspection for {}", url);

    let (links, ignored_links) = inspect_links(&url, show_links, &config)?;

    println!("Discovered {} valid links to scan.", links.len());

    output_results(&matches, &config, &links, &ignored_links, detailed)?;

    if detailed {
        println!("Ignored {} links.", ignored_links.len());
    }

    Ok(())
}

/// Create the CLI application with all arguments
fn create_cli_app() -> App<'static, 'static> {
    App::new("inspector-cli")
        .version("0.1.0")
        .about("Inspects links on a documentation site")
        .arg(
            Arg::with_name("URL")
                .help("The URL of the documentation site")
                .required_unless("config")
                .index(1),
        )
        .arg(
            Arg::with_name("output-format")
                .long("output-format")
                .short("o")
                .value_name("FORMAT")
                .help("Output format: json, yaml, txt, or clipboard")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output-file")
                .long("output-file")
                .short("f")
                .value_name("FILE")
                .help("Output file name (default: inspect-result-<domain>.<format>)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("log-level")
                .long("log-level")
                .short("l")
                .value_name("LEVEL")
                .help("Log level: info, debug, or error")
                .takes_value(true)
                .default_value("info"),
        )
        .arg(
            Arg::with_name("show-links")
                .long("show-links")
                .short("s")
                .help("Show links in the terminal"),
        )
        .arg(
            Arg::with_name("detailed")
                .long("detailed")
                .short("d")
                .help("Show detailed information including ignored links"),
        )
        .arg(
            Arg::with_name("config")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ignore-domains")
                .long("ignore-domains")
                .value_name("DOMAINS")
                .help("Comma-separated list of domains to ignore")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ignore-regex")
                .long("ignore-regex")
                .value_name("REGEX")
                .help("Comma-separated list of regex patterns to ignore URLs")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("forbidden-domains")
                .long("forbidden-domains")
                .value_name("DOMAINS")
                .help("Comma-separated list of forbidden domains")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ignored-childs")
                .long("ignored-childs")
                .value_name("PATHS")
                .help("Comma-separated list of child paths to ignore")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .value_name("SECONDS")
                .help("Timeout in seconds for each HTTP request")
                .takes_value(true),
        )
}

/// Setup the logger based on the provided log level
fn setup_logger(matches: &clap::ArgMatches) {
    let log_level = matches.value_of("log-level").unwrap();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();
}

/// Load and merge configuration from file and command-line arguments
fn load_and_merge_config(matches: &clap::ArgMatches) -> Result<Config, Box<dyn Error>> {
    let mut config = load_config(matches.value_of("config"))?.unwrap_or_default();

    // Override config with command-line arguments
    if let Some(ignore_domains) = matches.value_of("ignore-domains") {
        config.ignore.get_or_insert(IgnoreConfig::default()).domains =
            Some(ignore_domains.split(',').map(String::from).collect());
    }
    if let Some(ignore_regex) = matches.value_of("ignore-regex") {
        config.ignore.get_or_insert(IgnoreConfig::default()).regex =
            Some(ignore_regex.split(',').map(String::from).collect());
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

    Ok(config)
}

/// Get the URL from command-line arguments or config file
fn get_url(matches: &clap::ArgMatches, config: &Config) -> Result<String, Box<dyn Error>> {
    matches
        .value_of("URL")
        .map(String::from)
        .or_else(|| config.url.clone())
        .ok_or_else(|| {
            Box::<dyn Error>::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "URL is required when no config file is provided",
            ))
        })
}

/// Determine if a URL should be ignored based on configuration
fn should_ignore_url(url: &str, config: &Config, base_url: &str) -> bool {
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

    if let Some(ignore) = &config.ignore {
        if should_ignore_domain(domain, ignore) || should_ignore_regex(url, ignore) {
            return true;
        }
    }

    if should_ignore_forbidden_domain(domain, &config.forbidden_domains) {
        return true;
    }

    should_ignore_child_path(url, &base_parsed, &config.ignored_childs)
}

/// Check if the domain should be ignored
fn should_ignore_domain(domain: &str, ignore: &IgnoreConfig) -> bool {
    if let Some(domains) = &ignore.domains {
        if domains.iter().any(|ignored| domain.ends_with(ignored)) {
            println!("Ignoring due to ignore domains: {}", domain);
            return true;
        }
    }
    false
}

/// Check if the URL matches any ignore regex patterns
fn should_ignore_regex(url: &str, ignore: &IgnoreConfig) -> bool {
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
    false
}

/// Check if the domain is in the list of forbidden domains
fn should_ignore_forbidden_domain(domain: &str, forbidden_domains: &Option<Vec<String>>) -> bool {
    if let Some(forbidden) = forbidden_domains {
        if forbidden
            .iter()
            .any(|forbidden| domain.ends_with(forbidden))
        {
            println!("Ignoring due to forbidden domains: {}", domain);
            return true;
        }
    }
    false
}

/// Check if the URL should be ignored based on child paths
fn should_ignore_child_path(
    url: &str,
    base_parsed: &Url,
    ignored_childs: &Option<Vec<String>>,
) -> bool {
    if let Some(ignored_childs) = ignored_childs {
        for ignored_child in ignored_childs {
            let full_ignored_path = if base_parsed.path().ends_with('/') {
                format!(
                    "{}{}",
                    base_parsed.path(),
                    ignored_child.trim_start_matches('/')
                )
            } else {
                format!(
                    "{}/{}",
                    base_parsed.path(),
                    ignored_child.trim_start_matches('/')
                )
            };
            if url.starts_with(&(base_parsed.origin().ascii_serialization() + &full_ignored_path)) {
                println!("Ignoring URL due to ignored_childs: {}", url);
                return true;
            }
        }
    }
    false
}

/// Inspect links starting from a given URL
fn inspect_links(
    base_url: &str,
    show_links: bool,
    config: &Config,
) -> Result<(Vec<LinkInfo>, Vec<LinkInfo>), Box<dyn Error>> {
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

        match inspect_single_link(&client, &current_url) {
            Ok((link_info, html)) => {
                if show_links {
                    println!("Inspected: {:?}", link_info);
                }
                links.push(link_info);
                extract_links_from_html(&html, &current_url, &mut to_visit);
            }
            Err(link_info) => {
                links.push(link_info);
            }
        }
    }

    Ok((links, ignored_links))
}

/// Output results based on the specified format
fn output_results(
    matches: &clap::ArgMatches,
    config: &Config,
    links: &[LinkInfo],
    ignored_links: &[LinkInfo],
    detailed: bool,
) -> Result<(), Box<dyn Error>> {
    let output_format = matches
        .value_of("output-format")
        .unwrap_or_else(|| config.default_output.as_deref().unwrap_or("json"));
    let output_file = matches
        .value_of("output-file")
        .map(String::from)
        .unwrap_or_else(|| {
            format!(
                "inspect-result-{}.{}",
                config
                    .url
                    .as_ref()
                    .and_then(|url| Url::parse(url).ok())
                    .and_then(|url| url.domain().map(String::from))
                    .unwrap_or_else(|| "unknown".to_string()),
                output_format
            )
        });

    match output_format {
        "json" => output_json(links, ignored_links, detailed, &output_file),
        "yaml" => output_yaml(links, ignored_links, detailed, &output_file),
        "txt" => output_txt(links, &output_file),
        "clipboard" => output_clipboard(links),
        _ => {
            error!("Invalid output format");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;
