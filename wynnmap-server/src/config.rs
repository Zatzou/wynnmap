use std::sync::Arc;

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub(crate) struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
    pub images: ImagesConfig,
}

#[derive(Clone, Deserialize)]
pub(crate) struct ServerConfig {
    pub bind: Arc<str>,
    pub port: u16,
    pub base_url: Arc<str>,
}

#[derive(Clone, Deserialize)]
pub(crate) struct ClientConfig {
    pub ua_contact: Arc<str>,
}

#[derive(Clone, Deserialize)]
pub(crate) struct ImagesConfig {
    pub use_webp: bool,
}

pub(crate) async fn load_config() -> Arc<Config> {
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
