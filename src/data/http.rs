use std::net::IpAddr;

use reqwest::Url;

pub(crate) fn apply_loopback_proxy_policy(
    builder: reqwest::ClientBuilder,
    urls: &[&str],
) -> reqwest::ClientBuilder {
    if urls.iter().copied().any(should_bypass_proxy) {
        builder.no_proxy()
    } else {
        builder
    }
}

fn should_bypass_proxy(url: &str) -> bool {
    Url::parse(url).ok().is_some_and(|parsed| {
        parsed.host_str().is_some_and(|host| {
            host.eq_ignore_ascii_case("localhost")
                || host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
        })
    })
}
