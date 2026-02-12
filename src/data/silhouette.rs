use std::sync::OnceLock;

use anyhow::{Context, Result};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use img_to_ascii::{
    convert::{char_rows_to_string, get_conversion_algorithm, get_converter, img_to_char_rows},
    font::{Character, Font},
    image::LumaImage as AsciiLumaImage,
};
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

            for title in prioritize_titles(location, &query, titles) {
                if title.to_ascii_lowercase().contains("disambiguation") {
                    continue;
                }

                let image_url = match self.page_image_for_title(&title).await {
                    Ok(Some(url)) => Some(url),
                    Ok(None) | Err(_) => self.thumbnail_for_title(&title).await.unwrap_or(None),
                };

                let Some(url) = image_url else {
                    continue;
                };

                if !url.ends_with(".jpg")
                    && !url.ends_with(".jpeg")
                    && !url.ends_with(".png")
                    && !url.ends_with(".webp")
                {
                    continue;
                }

                let bytes = match self.fetch_bytes(&url).await {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if let Some(lines) = image_to_ascii(&bytes, 96, 42) {
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
            .map(|q| {
                q.search
                    .into_iter()
                    .map(|s| s.title)
                    .filter(|title| !title.to_ascii_lowercase().contains("disambiguation"))
                    .collect()
            })
            .unwrap_or_default();

        Ok(titles)
    }

    async fn page_image_for_title(&self, title: &str) -> Result<Option<String>> {
        let response = self
            .client
            .get(WIKI_SEARCH_URL)
            .query(&[
                ("action", "query"),
                ("prop", "pageimages"),
                ("piprop", "original|thumbnail"),
                ("pithumbsize", "1600"),
                ("redirects", "1"),
                ("format", "json"),
                ("formatversion", "2"),
                ("titles", title),
            ])
            .send()
            .await
            .context("wikipedia page image request failed")?
            .error_for_status()
            .context("wikipedia page image non-success status")?;

        let payload: PageImageResponse = response
            .json()
            .await
            .context("failed to decode wikipedia page image response")?;

        let Some(query) = payload.query else {
            return Ok(None);
        };

        for page in query.pages {
            if let Some(title) = page.title.as_deref()
                && title.to_ascii_lowercase().contains("disambiguation")
            {
                continue;
            }

            if let Some(image) = page.original.as_ref().or(page.thumbnail.as_ref())
                && image_is_useful(&image.source, image.width, image.height)
            {
                return Ok(Some(image.source.clone()));
            }
        }

        Ok(None)
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

        Ok(payload.thumbnail.and_then(|thumb| {
            if image_is_useful(&thumb.source, thumb.width, thumb.height) {
                Some(thumb.source)
            } else {
                None
            }
        }))
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
            "Gamla stan waterfront".to_string(),
        ]
    } else if norm.contains("paris") {
        vec![
            "Eiffel Tower".to_string(),
            "Paris skyline".to_string(),
            "Notre-Dame de Paris".to_string(),
        ]
    } else if norm.contains("new york") || norm.contains("nyc") {
        vec![
            "New York City skyline".to_string(),
            "Statue of Liberty".to_string(),
            "Manhattan skyline".to_string(),
        ]
    } else if norm.contains("tokyo") {
        vec![
            "Tokyo Tower".to_string(),
            "Tokyo skyline".to_string(),
            "Tokyo Skytree".to_string(),
        ]
    } else if norm.contains("london") {
        vec![
            "Elizabeth Tower".to_string(),
            "London skyline".to_string(),
            "Tower Bridge".to_string(),
        ]
    } else if norm.contains("sydney") {
        vec![
            "Sydney Opera House".to_string(),
            "Sydney skyline".to_string(),
            "Sydney Harbour Bridge".to_string(),
        ]
    } else if norm.contains("san diego") {
        vec![
            "San Diego skyline".to_string(),
            "Balboa Park".to_string(),
            "Hotel del Coronado".to_string(),
            "Cabrillo National Monument".to_string(),
            "Coronado Bridge".to_string(),
        ]
    } else {
        vec![
            format!("{city} skyline"),
            format!("{city} landmark"),
            format!("{city} architecture"),
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

fn image_to_ascii(bytes: &[u8], width: u32, height: u32) -> Option<Vec<String>> {
    let image = image::load_from_memory(bytes).ok()?;
    let converter = get_converter("direction-and-intensity");
    let algorithm = get_conversion_algorithm("edge-augmented");
    let char_rows = img_to_char_rows(
        ascii_font(),
        &AsciiLumaImage::from(&image),
        converter,
        Some(width.max(8) as usize),
        0.02,
        &algorithm,
    );
    if char_rows.is_empty() {
        return None;
    }

    let text = char_rows_to_string(&char_rows);
    let lines = text
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    if lines.is_empty() || lines.iter().all(|line| line.trim().is_empty()) {
        return None;
    }

    let mut lines = trim_ascii_whitespace(lines)?;

    let limit = height.max(4) as usize;
    if lines.len() > limit {
        let start = (lines.len() - limit) / 2;
        lines = lines[start..start + limit].to_vec();
    }

    let non_space = lines
        .iter()
        .flat_map(|line| line.chars())
        .filter(|ch| !ch.is_whitespace())
        .count();
    let total = lines.iter().map(|line| line.chars().count()).sum::<usize>();
    if total == 0 {
        return None;
    }

    let ratio = non_space as f32 / total as f32;
    if !(0.03..=0.85).contains(&ratio) {
        return None;
    }

    Some(lines)
}

fn trim_ascii_whitespace(lines: Vec<String>) -> Option<Vec<String>> {
    if lines.is_empty() {
        return None;
    }

    let top = lines.iter().position(|line| !line.trim().is_empty())?;
    let bottom = lines.iter().rposition(|line| !line.trim().is_empty())?;
    let rows = lines[top..=bottom].to_vec();

    let mut left = usize::MAX;
    let mut right = 0usize;
    for row in &rows {
        for (i, ch) in row.chars().enumerate() {
            if !ch.is_whitespace() {
                left = left.min(i);
                right = right.max(i);
            }
        }
    }
    if left == usize::MAX || right < left {
        return None;
    }

    let trimmed = rows
        .into_iter()
        .map(|row| {
            row.chars()
                .skip(left)
                .take(right.saturating_sub(left) + 1)
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>();

    Some(trimmed)
}

fn image_is_useful(source: &str, width: Option<u32>, height: Option<u32>) -> bool {
    let source = source.to_ascii_lowercase();
    if source.ends_with(".svg") || source.ends_with(".gif") {
        return false;
    }

    const BAD_IMAGE_TOKENS: [&str; 9] = [
        "logo",
        "wordmark",
        "seal",
        "flag",
        "map",
        "coat_of_arms",
        "coat-of-arms",
        "crest",
        "icon",
    ];
    if BAD_IMAGE_TOKENS.iter().any(|token| source.contains(token)) {
        return false;
    }

    if let (Some(width), Some(height)) = (width, height) {
        if width < 280 || height < 190 {
            return false;
        }
        let aspect = width as f32 / height as f32;
        if !(0.4..=4.2).contains(&aspect) {
            return false;
        }
    }
    true
}

fn ascii_font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(build_ascii_font)
}

fn build_ascii_font() -> Font {
    let alphabet: Vec<char> =
        " .'`^\",:;Il!i~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"
            .chars()
            .collect();
    let mut chars = Vec::new();

    for ch in alphabet.iter().copied() {
        let Some(glyph) = BASIC_FONTS.get(ch) else {
            continue;
        };
        let mut bitmap = Vec::with_capacity(64);
        for row in glyph {
            for bit in 0..8 {
                let on = ((row >> bit) & 1) == 1;
                bitmap.push(if on { 1.0 } else { 0.0 });
            }
        }
        chars.push(Character::new(ch, bitmap, 8, 8));
    }

    Font::new(&chars, &alphabet)
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

fn prioritize_titles(location: &Location, query: &str, titles: Vec<String>) -> Vec<String> {
    let city = normalize_key_part(&location.name);
    let query_norm = normalize_key_part(query);
    let admin = location.admin1.as_deref().map(normalize_key_part);
    let country = location.country.as_deref().map(normalize_key_part);

    let mut scored = titles
        .into_iter()
        .filter(|title| !normalize_key_part(title).contains("disambiguation"))
        .map(|title| {
            let score = title_score(
                &title,
                &city,
                &query_norm,
                admin.as_deref(),
                country.as_deref(),
            );
            (score, title)
        })
        .collect::<Vec<_>>();

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.len().cmp(&b.1.len())));
    scored
        .into_iter()
        .filter(|(score, _)| *score > -120)
        .map(|(_, title)| title)
        .collect()
}

fn title_score(
    title: &str,
    city: &str,
    query_norm: &str,
    admin: Option<&str>,
    country: Option<&str>,
) -> i32 {
    let norm = normalize_key_part(title);
    let mut score = 0;

    if !city.is_empty() && norm.contains(city) {
        score += 80;
    }
    if !query_norm.is_empty() && norm.contains(query_norm) {
        score += 45;
    }
    if let Some(admin) = admin
        && !admin.is_empty()
        && norm.contains(admin)
    {
        score += 18;
    }
    if let Some(country) = country
        && !country.is_empty()
        && norm.contains(country)
    {
        score += 12;
    }

    const LANDMARK_TOKENS: [&str; 16] = [
        "skyline",
        "tower",
        "bridge",
        "city hall",
        "harbor",
        "harbour",
        "waterfront",
        "cathedral",
        "museum",
        "opera",
        "landmark",
        "monument",
        "park",
        "square",
        "castle",
        "gate",
    ];
    for token in LANDMARK_TOKENS {
        if norm.contains(token) {
            score += if token == "skyline" { 42 } else { 18 };
        }
    }

    const BAD_TITLE_TOKENS: [&str; 20] = [
        " fc",
        " football club",
        " soccer",
        " baseball",
        " basketball",
        " roster",
        " season",
        " album",
        " song",
        " discography",
        " film",
        " episode",
        " tv series",
        " election",
        " constituency",
        " district",
        " list of",
        " university",
        " high school",
        " airline",
    ];
    for token in BAD_TITLE_TOKENS {
        if norm.contains(token) {
            score -= 95;
        }
    }

    if norm.len() > 90 {
        score -= 16;
    }
    if norm.contains("disambiguation") {
        score -= 500;
    }

    score
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
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct PageImageResponse {
    query: Option<PageImageQuery>,
}

#[derive(Debug, Deserialize)]
struct PageImageQuery {
    pages: Vec<PageImagePage>,
}

#[derive(Debug, Deserialize)]
struct PageImagePage {
    title: Option<String>,
    original: Option<WikiImage>,
    thumbnail: Option<WikiImage>,
}

#[derive(Debug, Deserialize)]
struct WikiImage {
    source: String,
    width: Option<u32>,
    height: Option<u32>,
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
