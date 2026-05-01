use std::sync::Arc;

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
    pub images: ImagesConfig,
    pub otel: Option<OtelConfig>,
}

#[derive(Clone, Deserialize)]
pub struct ServerConfig {
    pub bind: Arc<str>,
    pub port: u16,
    pub base_url: Arc<str>,
    pub fe_dir: Arc<str>,
}

#[derive(Clone, Deserialize)]
pub struct ClientConfig {
    pub ua_contact: Arc<str>,
}

#[derive(Clone, Deserialize)]
pub struct ImagesConfig {
    pub use_webp: bool,
}

#[derive(Clone, Deserialize)]
pub struct OtelConfig {
    pub endpoint: Arc<str>,
    pub env_name: Arc<str>,
}

pub async fn load_config() -> Arc<Config> {
    let config = tokio::fs::read_to_string("config.toml")
        .await
        .expect("Failed to read config.toml");

    let r = toml::from_str(&config);

    match r {
        Ok(conf) => conf,
        Err(e) => {
            panic!(
                "Error loading configuration at {:?}: {}",
                e.span(),
                e.message()
            )
        }
    }
}
