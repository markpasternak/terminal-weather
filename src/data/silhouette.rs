use anyhow::{Context, Result};
use image::GenericImageView;
use reqwest::Client;
use serde::Deserialize;

use crate::domain::weather::{ColoredGlyph, Location, SilhouetteArt};

const WIKI_SEARCH_URL: &str = "https://en.wikipedia.org/w/api.php";
const WIKI_SUMMARY_BASE: &str = "https://en.wikipedia.org/api/rest_v1/page/summary";
type ColoredRows = Vec<Vec<ColoredGlyph>>;
const WEB_ART_RENDER_WIDTH: u32 = 420;
const WEB_ART_RENDER_HEIGHT: u32 = 240;

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

                if let Some((lines, colored_lines)) =
                    image_to_ascii(&bytes, WEB_ART_RENDER_WIDTH, WEB_ART_RENDER_HEIGHT)
                {
                    return Ok(Some(SilhouetteArt {
                        label: title,
                        lines,
                        colored_lines,
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
            "Stockholm skyline silhouette".to_string(),
            "Gamla stan waterfront".to_string(),
        ]
    } else if norm.contains("paris") {
        vec![
            "Eiffel Tower".to_string(),
            "Eiffel Tower silhouette".to_string(),
            "Paris skyline silhouette".to_string(),
            "Notre-Dame de Paris".to_string(),
        ]
    } else if norm.contains("new york") || norm.contains("nyc") {
        vec![
            "Manhattan skyline silhouette".to_string(),
            "New York City skyline".to_string(),
            "Statue of Liberty".to_string(),
        ]
    } else if norm.contains("tokyo") {
        vec![
            "Tokyo Tower".to_string(),
            "Tokyo skyline silhouette".to_string(),
            "Tokyo Skytree".to_string(),
        ]
    } else if norm.contains("london") {
        vec![
            "Elizabeth Tower".to_string(),
            "London skyline silhouette".to_string(),
            "Tower Bridge".to_string(),
        ]
    } else if norm.contains("sydney") {
        vec![
            "Sydney Opera House".to_string(),
            "Sydney skyline silhouette".to_string(),
            "Sydney Harbour Bridge".to_string(),
        ]
    } else if norm.contains("san diego") {
        vec![
            "San Diego skyline silhouette".to_string(),
            "San Diego skyline".to_string(),
            "Balboa Park".to_string(),
            "Hotel del Coronado".to_string(),
            "Coronado Bridge".to_string(),
        ]
    } else if norm.contains("moscow") || norm.contains("moskva") {
        vec![
            "Moscow Kremlin".to_string(),
            "Saint Basil's Cathedral".to_string(),
            "Moscow skyline".to_string(),
        ]
    } else if norm.contains("dubai") {
        vec![
            "Burj Khalifa".to_string(),
            "Dubai skyline".to_string(),
            "Dubai Marina".to_string(),
        ]
    } else if norm.contains("rome") || norm.contains("roma") {
        vec![
            "Colosseum".to_string(),
            "Rome skyline".to_string(),
            "St. Peter's Basilica".to_string(),
        ]
    } else if norm.contains("berlin") {
        vec![
            "Brandenburg Gate".to_string(),
            "Berlin skyline".to_string(),
            "Fernsehturm Berlin".to_string(),
        ]
    } else if norm.contains("san francisco") {
        vec![
            "Golden Gate Bridge".to_string(),
            "San Francisco skyline".to_string(),
        ]
    } else if norm.contains("chicago") {
        vec![
            "Chicago skyline".to_string(),
            "Willis Tower".to_string(),
            "Cloud Gate".to_string(),
        ]
    } else if norm.contains("rio") {
        vec![
            "Christ the Redeemer".to_string(),
            "Rio de Janeiro skyline".to_string(),
            "Sugarloaf Mountain".to_string(),
        ]
    } else if norm.contains("shanghai") {
        vec![
            "Shanghai skyline".to_string(),
            "Oriental Pearl Tower".to_string(),
            "The Bund".to_string(),
        ]
    } else if norm.contains("hong kong") {
        vec![
            "Hong Kong skyline".to_string(),
            "Victoria Harbour".to_string(),
        ]
    } else if norm.contains("singapore") {
        vec![
            "Singapore skyline".to_string(),
            "Marina Bay Sands".to_string(),
            "Merlion".to_string(),
        ]
    } else {
        vec![
            format!("{city} skyline silhouette"),
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

/// Half-block rendering: each terminal cell encodes two vertical pixels using
/// `▀` (upper-half block) with fg = top pixel, bg = bottom pixel.  This is the
/// same technique used by chafa/viu/timg and gives true-color representation
/// with 2× vertical resolution compared to single-character approaches.
fn image_to_ascii(
    bytes: &[u8],
    width: u32,
    height: u32,
) -> Option<(Vec<String>, Option<ColoredRows>)> {
    let img = image::load_from_memory(bytes).ok()?;

    let chars_w = width.max(8);
    // Each character cell is 2 pixels tall, so we need 2× the pixel rows.
    let chars_h = height.max(4);
    let px_h = chars_h * 2;

    let resized = img.resize(chars_w, px_h, image::imageops::FilterType::Lanczos3);
    let (actual_w, actual_h) = resized.dimensions();

    let mut colored_rows: ColoredRows = Vec::with_capacity(chars_h as usize);

    let mut cy = 0u32;
    while cy * 2 < actual_h {
        let mut row = Vec::with_capacity(actual_w as usize);
        for cx in 0..actual_w {
            let top_y = cy * 2;
            let bot_y = (cy * 2 + 1).min(actual_h - 1);

            let [tr, tg, tb, _] = resized.get_pixel(cx, top_y).0;
            let [br, bg, bb, _] = resized.get_pixel(cx, bot_y).0;

            row.push(ColoredGlyph {
                ch: '▀',
                color: Some((tr, tg, tb)),
                bg_color: Some((br, bg, bb)),
            });
        }
        colored_rows.push(row);
        cy += 1;
    }

    let lines: Vec<String> = colored_rows
        .iter()
        .map(|row| row.iter().map(|g| g.ch).collect())
        .collect();

    if lines.is_empty() {
        return None;
    }

    Some((lines, Some(colored_rows)))
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

    const LANDMARK_TOKENS: [&str; 18] = [
        "silhouette",
        "outline",
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
            score += match token {
                "silhouette" | "outline" => 55,
                "skyline" => 42,
                _ => 18,
            };
        }
    }

    const BAD_TITLE_TOKENS: [&str; 34] = [
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
        "cruiser",
        "battleship",
        "destroyer",
        "frigate",
        "submarine",
        "warship",
        "class ship",
        "aircraft carrier",
        "regiment",
        "brigade",
        "battalion",
        "massacre",
        "earthquake",
        "hurricane",
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
