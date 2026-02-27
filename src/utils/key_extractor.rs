use axum::http::Request;
use tower_governor::GovernorError;
use tower_governor::key_extractor::KeyExtractor;

#[derive(Clone)]
pub struct SmartIpExtractor;

impl KeyExtractor for SmartIpExtractor {
    type Key = String;

    fn extract<B>(&self, req: &Request<B>) -> Result<String, GovernorError> {
        let ip = req
            .headers()
            .get("CF-Connecting-IP")
            .or_else(|| req.headers().get("X-Forwarded-For"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(",").next())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string());

        Ok(ip)
    }
}
