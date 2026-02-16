use dotenvy::{self, dotenv};
use std::env::{self, VarError};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub upload_path: String,
    pub bind_address: String,
    pub base_url: String,
}

pub enum ConfigError {
    VariableError(VarError),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            upload_path: env::var("UPLOAD_PATH")?,
            bind_address: env::var("BIND_ADDRESS")?,
            base_url: env::var("BASE_URL")?,
        })
    }
}

impl ConfigError {
    pub fn log(self) {
        match self {
            ConfigError::VariableError(err) => tracing::error!("Variable error: {}", err),
        };
    }
}

impl From<VarError> for ConfigError {
    fn from(err: VarError) -> Self {
        ConfigError::VariableError(err)
    }
}
