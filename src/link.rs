use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use url::Url;

/// Information about a link
#[derive(Debug, Serialize)]
pub struct LinkInfo {
    pub url: String,
    pub status: LinkStatus,
}

/// Status of a link
#[derive(Debug, Serialize)]
pub enum LinkStatus {
    Valid,
    NotFound,
    Error(String),
    Ignored,
}

/// Inspect a single link and return its status and HTML content if successful
pub fn inspect_single_link(client: &Client, url: &str) -> Result<(LinkInfo, String), LinkInfo> {
    match client.get(url).send() {
        Ok(response) => {
            let status = response.status();
            let link_status = if status.is_success() {
                LinkStatus::Valid
            } else if status.as_u16() == 404 {
                LinkStatus::NotFound
            } else {
                LinkStatus::Error(status.to_string())
            };

            let link_info = LinkInfo {
                url: url.to_string(),
                status: link_status,
            };

            if status.is_success() {
                let html = response.text().map_err(|e| LinkInfo {
                    url: url.to_string(),
                    status: LinkStatus::Error(e.to_string()),
                })?;
                Ok((link_info, html))
            } else {
                Err(link_info)
            }
        }
        Err(e) => Err(LinkInfo {
            url: url.to_string(),
            status: LinkStatus::Error(e.to_string()),
        }),
    }
}

/// Extract links from HTML content and add them to the to_visit queue
pub fn extract_links_from_html(html: &str, base_url: &str, to_visit: &mut Vec<String>) {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a").unwrap();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(absolute_url) = Url::parse(base_url).and_then(|base| base.join(href)) {
                to_visit.push(absolute_url.into());
            }
        }
    }
}
