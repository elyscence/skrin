use axum::http::header::{AUTHORIZATION, HeaderMap, HeaderValue};

pub fn get_token(request: &HeaderMap<HeaderValue>) -> Option<String> {
    request
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(String::from)
}
