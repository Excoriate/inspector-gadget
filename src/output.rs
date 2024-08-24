use crate::link::LinkInfo;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;

/// Output results in JSON format
pub fn output_json(
    links: &[LinkInfo],
    ignored_links: &[LinkInfo],
    detailed: bool,
    output_file: &str,
) -> Result<(), Box<dyn Error>> {
    let mut output = HashMap::new();
    output.insert("scanned_links", links);
    if detailed {
        output.insert("ignored_links", ignored_links);
    }
    let json = serde_json::to_string_pretty(&output)?;
    let mut file = File::create(output_file)?;

    file.write_all(json.as_bytes())?;

    Ok(())
}

/// Output results in YAML format
pub fn output_yaml(
    links: &[LinkInfo],
    ignored_links: &[LinkInfo],
    detailed: bool,
    output_file: &str,
) -> Result<(), Box<dyn Error>> {
    let mut output = HashMap::new();
    output.insert("scanned_links", links);
    if detailed {
        output.insert("ignored_links", ignored_links);
    }
    let yaml = serde_yaml::to_string(&output)?;
    let mut file = File::create(output_file)?;

    file.write_all(yaml.as_bytes())?;

    Ok(())
}

/// Output results in plain text format
pub fn output_txt(links: &[LinkInfo], output_file: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(output_file)?;

    for link in links {
        writeln!(file, "{:?}", link)?;
    }

    Ok(())
}

/// Output results to the clipboard
pub fn output_clipboard(links: &[LinkInfo]) -> Result<(), Box<dyn Error>> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    let content = links
        .iter()
        .map(|link| link.url.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    ctx.set_contents(content)?;
    println!("Links copied to clipboard.");
    Ok(())
}
