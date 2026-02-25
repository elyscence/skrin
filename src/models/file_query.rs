use serde::Deserialize;

#[derive(Deserialize)]
pub struct FileQuery {
    pub thumb: Option<bool>,
}
