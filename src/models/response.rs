use serde::Serialize;

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
    pub success: bool,
}
