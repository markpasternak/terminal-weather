use anyhow::{Context, Result};
use image::imageops::FilterType;
use reqwest::Client;
use serde::Deserialize;

use crate::domain::weather::{Location, SilhouetteArt};

const WIKI_SEARCH_URL: &str = "https://en.wikipedia.org/w/api.php";
const WIKI_SUMMARY_BASE: &str = "https://en.wikipedia.org/api/rest_v1/page/summary";

#[derive(Debug, Clone)]
pub struct SilhouetteClient {
    client: Client,
}

impl Default for SilhouetteClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SilhouetteClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .user_agent("atmos-tui/0.1 silhouette fetch")
                .build()
                .expect("reqwest client"),
        }
    }

    pub async fn fetch_for_location(&self, location: &Location) -> Result<Option<SilhouetteArt>> {
        let queries = queries_for_location(location);
        for query in queries {
            let titles = match self.search_titles(&query).await {
                Ok(v) => v,
                Err(_) => continue,
            };

            for title in titles {
                let thumb_url = match self.thumbnail_for_title(&title).await {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let Some(url) = thumb_url else {
                    continue;
                };

                let bytes = match self.fetch_bytes(&url).await {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if let Some(lines) = image_to_ascii(&bytes, 30, 11)? {
                    return Ok(Some(SilhouetteArt {
                        label: title,
                        lines,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn search_titles(&self, query: &str) -> Result<Vec<String>> {
        let response = self
            .client
            .get(WIKI_SEARCH_URL)
            .query(&[
                ("action", "query"),
                ("list", "search"),
                ("srsearch", query),
                ("format", "json"),
                ("srlimit", "5"),
                ("utf8", "1"),
            ])
            .send()
            .await
            .context("wikipedia search request failed")?
            .error_for_status()
            .context("wikipedia search non-success status")?;

        let payload: SearchResponse = response
            .json()
            .await
            .context("failed to decode wikipedia search response")?;

        let titles = payload
            .query
            .map(|q| q.search.into_iter().map(|s| s.title).collect())
            .unwrap_or_default();

        Ok(titles)
    }

    async fn thumbnail_for_title(&self, title: &str) -> Result<Option<String>> {
        let encoded = urlencoding::encode(title);
        let url = format!("{WIKI_SUMMARY_BASE}/{encoded}");
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("wikipedia summary request failed")?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let payload: SummaryResponse = response
            .json()
            .await
            .context("failed to decode wikipedia summary response")?;

        Ok(payload.thumbnail.map(|thumb| thumb.source))
    }

    async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let bytes = self
            .client
            .get(url)
            .send()
            .await
            .context("thumbnail request failed")?
            .error_for_status()
            .context("thumbnail request non-success status")?
            .bytes()
            .await
            .context("reading thumbnail bytes failed")?;
        Ok(bytes.to_vec())
    }
}

pub fn cache_key(location: &Location) -> String {
    cache_key_from_parts(&location.name, location.country.as_deref())
}

pub fn cache_key_from_parts(name: &str, country: Option<&str>) -> String {
    let city = normalize_key_part(name);
    let country = country
        .map(normalize_key_part)
        .filter(|part| !part.is_empty())
        .unwrap_or_else(|| "na".to_string());
    format!("{city}::{country}")
}

fn queries_for_location(location: &Location) -> Vec<String> {
    let city = location.name.trim();
    let norm = normalize_key_part(city);

    let mut out = if norm.contains("stockholm") {
        vec![
            "Stockholm City Hall".to_string(),
            "Stockholm skyline".to_string(),
        ]
    } else if norm.contains("paris") {
        vec!["Eiffel Tower".to_string(), "Paris skyline".to_string()]
    } else if norm.contains("new york") || norm.contains("nyc") {
        vec![
            "New York City skyline".to_string(),
            "Statue of Liberty".to_string(),
        ]
    } else if norm.contains("tokyo") {
        vec!["Tokyo Tower".to_string(), "Tokyo skyline".to_string()]
    } else if norm.contains("london") {
        vec!["Elizabeth Tower".to_string(), "London skyline".to_string()]
    } else if norm.contains("sydney") {
        vec![
            "Sydney Opera House".to_string(),
            "Sydney skyline".to_string(),
        ]
    } else {
        vec![
            format!("{city} skyline"),
            format!("{city} landmark"),
            city.to_string(),
        ]
    };

    if let Some(country) = &location.country {
        out.push(format!("{city} {country} skyline"));
    }

    out.sort();
    out.dedup();
    out
}

fn image_to_ascii(bytes: &[u8], width: u32, height: u32) -> Result<Option<Vec<String>>> {
    let image = image::load_from_memory(bytes).context("failed to decode image bytes")?;
    let gray = image
        .grayscale()
        .resize_exact(width.max(4), height.max(4), FilterType::Triangle)
        .to_luma8();

    let mut min = u8::MAX;
    let mut max = u8::MIN;
    let mut sum = 0u64;
    for pixel in gray.pixels() {
        let value = pixel[0];
        min = min.min(value);
        max = max.max(value);
        sum += u64::from(value);
    }

    if max.saturating_sub(min) < 12 {
        return Ok(None);
    }

    let mean = sum as f32 / gray.as_raw().len() as f32;
    let mut lines = Vec::with_capacity(gray.height() as usize);
    let mut non_space = 0usize;
    let total = (gray.width() * gray.height()) as usize;
    for y in 0..gray.height() {
        let mut line = String::with_capacity(gray.width() as usize);
        for x in 0..gray.width() {
            let value = gray.get_pixel(x, y)[0] as f32;
            let ch = if value <= mean * 0.72 {
                '@'
            } else if value <= mean * 0.84 {
                '#'
            } else if value <= mean * 0.94 {
                '+'
            } else if value <= mean * 1.02 {
                '.'
            } else {
                ' '
            };
            if ch != ' ' {
                non_space += 1;
            }
            line.push(ch);
        }
        lines.push(line);
    }

    if non_space < total / 35 || non_space > (total * 3) / 4 {
        return Ok(None);
    }

    let first = lines.iter().position(|line| line.trim().len() >= 3);
    let last = lines.iter().rposition(|line| line.trim().len() >= 3);
    let (Some(first), Some(last)) = (first, last) else {
        return Ok(None);
    };
    if last < first {
        return Ok(None);
    }

    Ok(Some(lines[first..=last].to_vec()))
}

fn normalize_key_part(input: &str) -> String {
    input
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '_', ','], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    query: Option<SearchQuery>,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    search: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    title: String,
}

#[derive(Debug, Deserialize)]
struct SummaryResponse {
    thumbnail: Option<SummaryThumbnail>,
}

#[derive(Debug, Deserialize)]
struct SummaryThumbnail {
    source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_normalization_is_stable() {
        assert_eq!(
            cache_key_from_parts("  Sao-Paulo,  ", Some("BR")),
            "sao paulo::br"
        );
        assert_eq!(
            cache_key_from_parts("Stockholm", None),
            "stockholm::na".to_string()
        );
    }
}
