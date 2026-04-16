use crate::constants::{DEFAULT_USER_AGENT, DOUYIN_ORIGIN, DOUYIN_REFERER};
use reqwest::header::{HeaderMap, HeaderValue, ORIGIN, REFERER, USER_AGENT};

pub fn build_common_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));
    headers.insert(ORIGIN, HeaderValue::from_static(DOUYIN_ORIGIN));
    headers.insert(REFERER, HeaderValue::from_static(DOUYIN_REFERER));
    headers
}
