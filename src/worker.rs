use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;
use scraper::{Html, Selector};

/// Storage maps a crawled domain (`String`)
/// to a set of links under the domain (`HashSet<String>`).
pub type Storage = RwLock<HashMap<String, HashSet<String>>>;

lazy_static! {
    static ref SELECTOR: Selector = Selector::parse("a").unwrap();
}

pub async fn crawl(storage: Arc<Storage>, mut base_url: url::Url) {
    // Keep only domain
    base_url.set_path("");
    base_url.set_query(None);
    base_url.set_fragment(None);

    let base_host = base_url.host_str().unwrap().to_owned(); // host presence validated in `crawl` handler

    let mut visited = HashSet::new();
    let mut locations = VecDeque::new();
    locations.push_back(base_url.to_string());

    while let Some(u) = locations.pop_front() {
        if visited.contains(&u) {
            continue;
        }
        visited.insert(u.clone());

        let body = match get(u.as_str()).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                log::debug!("Not html: {}", u);
                continue;
            }
            Err(e) => {
                log::error!("Error getting {}: {}", u, e);
                continue;
            }
        };

        if let Ok(urls) = extract_urls(&body) {
            for url in urls.into_iter().flat_map(|u| base_url.join(&u)) {
                // TODO: Add to storage
                let url_string = url.to_string();
                storage
                    .write()
                    .unwrap()
                    .entry(base_host.clone())
                    .or_default()
                    .insert(url_string.clone());

                // If internal URL: crawl it too
                if url.host_str() == Some(&base_host) && !visited.contains(&url_string) {
                    locations.push_back(url_string)
                }
            }
        }
    }
}

pub async fn get(url: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?;
    if resp.headers().get("Content-Type").map(|v| v.as_bytes()) != Some(b"text/html") {
        return Ok(None);
    }
    Ok(Some(resp.text().await?))
}

pub fn extract_urls(body: &str) -> Result<Vec<String>, ()> {
    Ok(Html::parse_document(body)
        .select(&SELECTOR)
        .flat_map(|n| n.value().attr("href"))
        .map(ToOwned::to_owned)
        .collect())
}
