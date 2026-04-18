#![allow(clippy::missing_errors_doc)]

use std::cmp::Ordering;
use std::net::IpAddr;
use std::time::Duration;

use anyhow::Context;
use reqwest::{Client, Url};

pub const HOMEBREW_FORMULA_URL: &str =
    "https://raw.githubusercontent.com/markpasternak/homebrew-tap/main/Formula/terminal-weather.rb";
pub const UPDATE_CHECK_DISABLE_ENV: &str = "TERMINAL_WEATHER_DISABLE_UPDATE_CHECK";
pub const UPDATE_CHECK_TIMEOUT_SECS: u64 = 3;
pub const UPDATE_CHECK_INTERVAL_SECS: i64 = 24 * 60 * 60;
pub const MAX_UPDATE_RESPONSE_SIZE: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    Unknown,
    UpToDate,
    UpdateAvailable { latest: String },
}

#[must_use]
pub fn now_unix_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[must_use]
pub fn should_check(now_unix: i64, last_check_unix: Option<i64>) -> bool {
    let Some(last_check_unix) = last_check_unix else {
        return true;
    };
    if last_check_unix > now_unix {
        return true;
    }
    now_unix - last_check_unix >= UPDATE_CHECK_INTERVAL_SECS
}

#[must_use]
pub fn formula_url() -> String {
    std::env::var("TERMINAL_WEATHER_UPDATE_FORMULA_URL")
        .ok()
        .filter(|url| {
            let lower = url.trim().to_ascii_lowercase();
            lower.starts_with("https://")
                || lower.starts_with("http://127.0.0.1")
                || lower.starts_with("http://localhost")
        })
        .unwrap_or_else(|| HOMEBREW_FORMULA_URL.to_string())
}

#[must_use]
pub fn update_check_disabled() -> bool {
    std::env::var(UPDATE_CHECK_DISABLE_ENV)
        .ok()
        .is_some_and(|value| disable_update_check_value(&value))
}

fn disable_update_check_value(raw: &str) -> bool {
    let value = raw.trim().to_ascii_lowercase();
    !matches!(value.as_str(), "0" | "false" | "no" | "off")
}

pub async fn check_latest_version() -> anyhow::Result<Option<String>> {
    check_latest_version_from_url(&formula_url()).await
}

pub(crate) async fn check_latest_version_from_url(url: &str) -> anyhow::Result<Option<String>> {
    let disable_proxy = should_bypass_proxy(url);
    let client = Client::builder()
        .user_agent(concat!("terminal-weather/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(UPDATE_CHECK_TIMEOUT_SECS))
        .no_proxy_if(disable_proxy)
        .build()
        .context("failed to build update client")?;
    let mut response = client
        .get(url)
        .send()
        .await
        .context("update formula request failed")?
        .error_for_status()
        .context("update formula request returned non-success status")?;

    let mut body_bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .context("reading update formula chunk failed")?
    {
        if body_bytes.len() + chunk.len() > MAX_UPDATE_RESPONSE_SIZE {
            anyhow::bail!("update check response too large");
        }
        body_bytes.extend_from_slice(&chunk);
    }
    let body =
        String::from_utf8(body_bytes).context("update check response was not valid UTF-8")?;

    Ok(parse_formula_version(&body))
}

fn should_bypass_proxy(url: &str) -> bool {
    Url::parse(url).ok().is_some_and(|parsed| {
        parsed.host_str().is_some_and(|host| {
            host.eq_ignore_ascii_case("localhost")
                || host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
        })
    })
}

trait ClientBuilderExt {
    fn no_proxy_if(self, condition: bool) -> Self;
}

impl ClientBuilderExt for reqwest::ClientBuilder {
    fn no_proxy_if(self, condition: bool) -> Self {
        if condition { self.no_proxy() } else { self }
    }
}

#[must_use]
pub fn parse_formula_version(formula: &str) -> Option<String> {
    formula.lines().find_map(parse_version_line)
}

fn parse_version_line(line: &str) -> Option<String> {
    let line = line.trim_start();
    let remainder = line.strip_prefix("version")?.trim_start();
    let first_quote = remainder.find('"')?;
    let value = &remainder[first_quote + 1..];
    let end_quote = value.find('"')?;
    let parsed = value[..end_quote].trim();
    (!parsed.is_empty()).then_some(parsed.to_string())
}

#[must_use]
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    matches!(compare_versions(current, latest), Some(Ordering::Less))
}

fn compare_versions(current: &str, latest: &str) -> Option<Ordering> {
    let current = parse_version(current)?;
    let latest = parse_version(latest)?;
    let ordering = current
        .major
        .cmp(&latest.major)
        .then_with(|| current.minor.cmp(&latest.minor))
        .then_with(|| current.patch.cmp(&latest.patch));
    if ordering != Ordering::Equal {
        return Some(ordering);
    }

    Some(compare_prerelease(
        current.pre_release.as_deref(),
        latest.pre_release.as_deref(),
    ))
}

fn compare_prerelease(current: Option<&str>, latest: Option<&str>) -> Ordering {
    match (current, latest) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(b),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedVersion {
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: Option<String>,
}

fn parse_version(raw: &str) -> Option<ParsedVersion> {
    let trimmed = raw.trim().trim_start_matches('v');
    let (core, pre_release) = split_core_and_prerelease(trimmed);
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    Some(ParsedVersion {
        major,
        minor,
        patch,
        pre_release,
    })
}

fn split_core_and_prerelease(value: &str) -> (&str, Option<String>) {
    value
        .split_once('-')
        .map_or((value, None), |(core, pre)| (core, Some(pre.to_string())))
}

pub async fn check_for_update(current_version: &str) -> UpdateStatus {
    check_for_update_with_url(current_version, &formula_url()).await
}

pub(crate) async fn check_for_update_with_url(current_version: &str, url: &str) -> UpdateStatus {
    match check_latest_version_from_url(url).await {
        Ok(Some(latest)) if is_newer_version(current_version, &latest) => {
            UpdateStatus::UpdateAvailable { latest }
        }
        Ok(_) => UpdateStatus::UpToDate,
        Err(_) => UpdateStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn parse_formula_version_extracts_standard_line() {
        let formula = r#"
class TerminalWeather < Formula
  desc "Weather"
  version "0.7.0"
end
"#;
        assert_eq!(parse_formula_version(formula).as_deref(), Some("0.7.0"));
    }

    #[test]
    fn parse_formula_version_handles_spacing() {
        let formula = "  version   \"1.2.3-beta.1\"";
        assert_eq!(
            parse_formula_version(formula).as_deref(),
            Some("1.2.3-beta.1")
        );
    }

    #[test]
    fn parse_formula_version_returns_none_without_version() {
        assert!(parse_formula_version("class Formula; end").is_none());
    }

    #[test]
    fn is_newer_version_compares_semver_like_values() {
        assert!(is_newer_version("0.6.0", "0.7.0"));
        assert!(!is_newer_version("0.7.0", "0.7.0"));
        assert!(!is_newer_version("0.7.0", "nope"));
    }

    #[test]
    fn should_check_respects_interval_and_future_timestamps() {
        let now = 2_000_000;
        assert!(should_check(now, None));
        assert!(!should_check(now, Some(now - 60)));
        assert!(should_check(now, Some(now - UPDATE_CHECK_INTERVAL_SECS)));
        assert!(should_check(now, Some(now + 60)));
    }

    #[test]
    fn disable_update_check_value_parses_bool_like_strings() {
        assert!(disable_update_check_value("1"));
        assert!(disable_update_check_value("true"));
        assert!(disable_update_check_value("YES"));
        assert!(!disable_update_check_value("0"));
        assert!(!disable_update_check_value("false"));
        assert!(!disable_update_check_value("no"));
        assert!(!disable_update_check_value("off"));
    }

    #[tokio::test]
    async fn check_for_update_with_url_detects_update() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/Formula/terminal-weather.rb"))
            .respond_with(ResponseTemplate::new(200).set_body_string("version \"0.7.0\""))
            .mount(&server)
            .await;
        let status = check_for_update_with_url(
            "0.6.0",
            &format!("{}/Formula/terminal-weather.rb", server.uri()),
        )
        .await;
        assert_eq!(
            status,
            UpdateStatus::UpdateAvailable {
                latest: "0.7.0".to_string()
            }
        );
    }

    #[tokio::test]
    async fn check_for_update_with_url_returns_unknown_on_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/Formula/terminal-weather.rb"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let status = check_for_update_with_url(
            "0.6.0",
            &format!("{}/Formula/terminal-weather.rb", server.uri()),
        )
        .await;

        assert_eq!(status, UpdateStatus::Unknown);
    }

    #[tokio::test]
    async fn check_latest_version_rejects_large_response() {
        let server = MockServer::start().await;
        let big_body = "a".repeat(MAX_UPDATE_RESPONSE_SIZE + 1);
        Mock::given(method("GET"))
            .and(path("/Formula/terminal-weather.rb"))
            .respond_with(ResponseTemplate::new(200).set_body_string(big_body))
            .mount(&server)
            .await;

        let result =
            check_latest_version_from_url(&format!("{}/Formula/terminal-weather.rb", server.uri()))
                .await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("response too large")
        );
    }
}
