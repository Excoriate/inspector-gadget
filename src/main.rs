use clap::{App, Arg};
use reqwest::blocking::ClientBuilder;
use scraper::{Html, Selector};
use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::time::Duration;
use log::{info, error};
use url::Url;
use serde::{Serialize, Deserialize};
use clipboard::{ClipboardContext, ClipboardProvider};
use regex::Regex;
use serde_yaml::Value;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    ignore: IgnoreConfig,
    forbidden_domains: Vec<String>,
    #[serde(default)]
    ignored_childs: Vec<String>,
    timeout: u64,
    default_output: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IgnoreConfig {
    domains: Vec<String>,
    regex: Vec<String>,
}

#[derive(Debug, Serialize)]
struct LinkInfo {
    url: String,
    status: LinkStatus,
}

#[derive(Debug, Serialize)]
enum LinkStatus {
    Valid,
    NotFound,
    Error(String),
    Ignored,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let matches = App::new("inspector-gadget")
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
    let config = load_config()?;

    let (links, ignored_links) = inspect_links(url, show_links, &config, strict)?;

    println!("Discovered {} valid links to scan.", links.len());

    let output_format = matches.value_of("output-format").unwrap_or(&config.default_output);
    let output_file = matches.value_of("output-file").map(String::from).unwrap_or_else(|| {
        let domain = Url::parse(url).unwrap().domain().unwrap_or("unknown").to_string();
        format!("inspect-result-{}.{}", domain, output_format)
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

fn load_config() -> Result<Config, Box<dyn Error>> {
    let mut file = File::open(".inspector-config.yml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let mut value: Value = serde_yaml::from_str(&contents)?;
    
    // Ensure ignored_childs is a sequence, or set it to an empty vector
    if let Some(ignored_childs) = value.get_mut("ignored_childs") {
        if !ignored_childs.is_sequence() {
            *ignored_childs = Value::Sequence(vec![]);
        }
    } else {
        value["ignored_childs"] = Value::Sequence(vec![]);
    }
    
    let config: Config = serde_yaml::from_value(value)?;
    Ok(config)
}

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

    if config.ignore.domains.iter().any(|ignored| domain.ends_with(ignored)) {
        return true;
    }

    if config.forbidden_domains.iter().any(|forbidden| domain.ends_with(forbidden)) {
        return true;
    }

    if config.ignored_childs.iter().any(|ignored_child| path.starts_with(ignored_child)) {
        return true;
    }

    for pattern in &config.ignore.regex {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(url) {
                return true;
            }
        }
    }

    false
}

fn inspect_links(url: &str, show_links: bool, config: &Config, strict: bool) -> Result<(Vec<LinkInfo>, Vec<LinkInfo>), Box<dyn Error>> {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(config.timeout))
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

fn output_txt(links: &[LinkInfo], output_file: &str) -> Result<(), Box<dyn Error>> {
    let content: String = links.iter().map(|link| format!("{}\n", link.url)).collect();
    let mut file = File::create(output_file)?;
    file.write_all(content.as_bytes())?;
    println!("TXT output written to {}", output_file);
    Ok(())
}

fn output_clipboard(links: &[LinkInfo]) -> Result<(), Box<dyn Error>> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    let content: String = links.iter().map(|link| format!("{}\n", link.url)).collect();
    ctx.set_contents(content)?;
    println!("Links copied to clipboard");
    Ok(())
}