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
            Ok(t) => t,
            Err(e) => {
                log::error!("Error getting {}: {}", u, e);
                continue;
            }
        };

        for url in extract_urls(&base_url, &body) {
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

pub async fn get(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(reqwest::get(url).await?.text().await?)
}

pub fn extract_urls(base_url: &url::Url, body: &str) -> Vec<url::Url> {
    Html::parse_document(body)
        .select(&SELECTOR)
        .flat_map(|n| n.value().attr("href"))
        .flat_map(|u| base_url.join(&u))
        .collect()
}

#[cfg(test)]
mod test {
    use super::extract_urls;

    #[test]
    fn test_extract_urls() {
        let body = r#"
            <a href="/">Home</a>
            <a href="/about">About</a>
            <p><a href="https://external-link.com">External</a></p>
        "#;
        let base_url = url::Url::parse("http://example.com").unwrap();
        let result: Vec<_> = extract_urls(&base_url, body)
            .into_iter()
            .map(|u| u.to_string())
            .collect();
        assert_eq!(
            vec![
                "http://example.com/",
                "http://example.com/about",
                "https://external-link.com/"
            ],
            result
        );
    }
}
