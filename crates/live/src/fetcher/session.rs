use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use common::error::{FetcherError, Result};
use reqwest::cookie::{CookieStore, Jar};
use reqwest::header::{COOKIE, HeaderValue, ORIGIN, REFERER, USER_AGENT};
use reqwest::{Client, Url};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use crate::constants::LIVE_URL;
use common::constants::{DEFAULT_USER_AGENT as UA, DOUYIN_HOST, DOUYIN_ORIGIN};
use common::cookies::parse_cookie_string;

pub struct FetchSession {
    client: Client,
    jar: Arc<Jar>,
}

impl FetchSession {
    pub fn new(cookies: BTreeMap<String, String>) -> Result<Self> {
        let jar = Arc::new(Jar::default());
        seed_cookies(&jar, cookies)?;

        let client = Client::builder()
            .cookie_provider(Arc::clone(&jar))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(FetcherError::Http)?;

        Ok(Self { client, jar })
    }

    pub async fn get_text(
        &self,
        url: &str,
        headers: reqwest::header::HeaderMap,
        _request_context: &str,
        _status_context: &str,
    ) -> Result<String> {
        self.client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(FetcherError::Http)?
            .error_for_status()
            .map_err(FetcherError::Http)?
            .text()
            .await
            .map_err(FetcherError::Http)
    }

    pub async fn warm_cookies(
        &self,
        url: &str,
        headers: reqwest::header::HeaderMap,
        _request_context: &str,
        _status_context: &str,
    ) -> Result<()> {
        self.client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(FetcherError::Http)?
            .error_for_status()
            .map_err(FetcherError::Http)?;
        Ok(())
    }

    pub fn request_headers(
        &self,
        url: &str,
        extras: &[(&str, String)],
        live_page_url: Option<&str>,
    ) -> Result<reqwest::header::HeaderMap> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));
        let cookie_header = self.request_cookie_header(url, extras)?;
        if !cookie_header.is_empty() {
            headers.insert(
                COOKIE,
                HeaderValue::from_str(&cookie_header).map_err(|e| {
                    FetcherError::Internal(format!("Failed to build request Cookie header: {e}"))
                })?,
            );
        }
        if let Some(referer) = live_page_url {
            headers.insert(
                REFERER,
                HeaderValue::from_str(referer).map_err(|e| {
                    FetcherError::Internal(format!("Failed to build request Referer header: {e}"))
                })?,
            );
            headers.insert(ORIGIN, HeaderValue::from_static(DOUYIN_ORIGIN));
        }
        Ok(headers)
    }

    pub fn websocket_request(
        &self,
        signed_url: &str,
        live_page_url: &str,
    ) -> Result<tokio_tungstenite::tungstenite::http::Request<()>> {
        let mut request = signed_url.into_client_request().map_err(|e| {
            FetcherError::Internal(format!("Failed to create WebSocket request: {e}"))
        })?;
        request
            .headers_mut()
            .insert(USER_AGENT, HeaderValue::from_static(UA));
        request.headers_mut().insert(
            REFERER,
            HeaderValue::from_str(live_page_url)
                .map_err(|e| FetcherError::Internal(format!("Invalid Referer: {e}")))?,
        );
        request
            .headers_mut()
            .insert(ORIGIN, HeaderValue::from_static(DOUYIN_ORIGIN));

        let cookie_header = self.request_cookie_header(LIVE_URL, &[])?;
        if !cookie_header.is_empty() {
            request.headers_mut().insert(
                COOKIE,
                HeaderValue::from_str(&cookie_header).map_err(|e| {
                    FetcherError::Internal(format!(
                        "Cookie header contains illegal characters: {e}"
                    ))
                })?,
            );
        }

        Ok(request)
    }

    pub fn cookie_value(&self, url: &str, key: &str) -> Result<Option<String>> {
        let url =
            Url::parse(url).map_err(|e| FetcherError::Internal(format!("Invalid URL: {e}")))?;
        Ok(cookie_pairs_from_header(self.jar.cookies(&url)).remove(key))
    }

    fn request_cookie_header(&self, url: &str, extras: &[(&str, String)]) -> Result<String> {
        let url =
            Url::parse(url).map_err(|e| FetcherError::Internal(format!("Invalid URL: {e}")))?;
        let mut pairs = cookie_pairs_from_header(self.jar.cookies(&url));
        for (key, value) in extras {
            if !value.is_empty() {
                pairs.insert((*key).to_string(), value.clone());
            }
        }

        Ok(pairs
            .into_iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("; "))
    }
}

fn seed_cookies(jar: &Jar, cookies: BTreeMap<String, String>) -> Result<()> {
    let live_url = Url::parse(LIVE_URL)
        .map_err(|e| FetcherError::Internal(format!("Invalid LIVE_URL: {e}")))?;
    let host_url = Url::parse(DOUYIN_HOST)
        .map_err(|e| FetcherError::Internal(format!("Invalid DOUYIN_HOST: {e}")))?;
    for (key, value) in cookies {
        if value.is_empty() {
            continue;
        }
        let cookie = format!("{key}={value}");
        jar.add_cookie_str(&cookie, &live_url);
        jar.add_cookie_str(&cookie, &host_url);
    }
    Ok(())
}

fn cookie_pairs_from_header(header: Option<HeaderValue>) -> BTreeMap<String, String> {
    let Some(header) = header else {
        return BTreeMap::new();
    };
    let Ok(header) = header.to_str() else {
        return BTreeMap::new();
    };

    parse_cookie_string(header)
}
