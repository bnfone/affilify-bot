// src/utils.rs
use regex::Regex;
use reqwest::Client;
use url::Url;

pub async fn resolve_url(input: &str) -> reqwest::Result<String> {
    let client = Client::builder().redirect(reqwest::redirect::Policy::limited(10)).build()?;
    let resp = client.get(input).send().await?;
    Ok(resp.url().to_string())
}

/// Parse an Amazon URL and return (ASIN, region)
/// region will be everything after "amazon.", e.g. "de", "co.uk", "com"
pub fn parse_amazon_url(url_str: &str) -> Option<(String, String)> {
    let url = Url::parse(url_str).ok()?;
    let host = url.host_str()?; // e.g. "www.amazon.co.uk" or "smile.amazon.com"
    // strip “www.” or any subdomain before “amazon.”
    let domain = host
        .trim_start_matches("www.")
        .trim_start_matches("smile.")
        .trim_start_matches("smile-redirect.")
        ; // now "amazon.de", "amazon.co.uk", "amazon.com"...
    // take what comes after "amazon."
    let region = domain
        .strip_prefix("amazon.")
        .unwrap_or("de") // default to "de" if something odd happens
        .to_string();

    // capture the ASIN from a “/dp/ASIN/” path segment
    let re = Regex::new(r"/dp/([A-Z0-9]+)/?").unwrap();
    re.captures(url.path())
      .and_then(|cap| cap.get(1))
      .map(|m| (m.as_str().to_string(), region))
}