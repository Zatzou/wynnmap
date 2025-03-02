use std::{collections::HashMap, io::Cursor, sync::Arc};

use crate::{ImageState, config::Config};
use axum::body::Bytes;
use image::ImageReader;
use tracing::{error, info};
use webp::Encoder;
use wynnmap_types::WynntilsMapTile;

pub(crate) async fn create_image_tracker(config: Arc<Config>) -> ImageState {
    let state = ImageState {
        config: config.clone(),
        maps: Default::default(),
        map_cache: Default::default(),
    };

    tokio::spawn(image_tracker(state.clone()));

    state
}

async fn image_tracker(state: ImageState) {
    // restart the image tracker if it fails for some reason
    loop {
        let r = image_tracker_inner(&state).await;

        if let Err(e) = r {
            error!("Image tracker failed: {}", e);
        }

        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }

    async fn image_tracker_inner(state: &ImageState) -> Result<(), reqwest::Error> {
        let mut etag_cache: HashMap<Arc<str>, Arc<str>> = HashMap::new();
        let client = reqwest::Client::builder()
            .user_agent(format!(
                "{}/{} ({})",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                state.config.client.ua_contact
            ))
            .build()
            .unwrap();

        loop {
            info!("Loading wynntils/static-storage maps.json");
            let maps_res = client
                .get("https://raw.githubusercontent.com/Wynntils/Static-Storage/refs/heads/main/Reference/maps.json")
                .header(
                    "If-None-Match",
                    &**etag_cache.get("maps.json").unwrap_or(&Arc::from("")),
                )
                .send()
                .await?;

            let status = maps_res.status();
            if status.is_success() {
                let etag = maps_res
                    .headers()
                    .get("etag")
                    .map(|h| h.to_str().unwrap_or(""))
                    .unwrap_or("");
                etag_cache.insert(Arc::from("maps.json"), Arc::from(etag));

                // load the json
                let mut data: Vec<WynntilsMapTile> = maps_res.json().await?;

                // load each map tile into the cache
                for item in &data {
                    load_image(&state, item, client.clone(), &mut etag_cache).await?;
                }

                // replace the urls in the data with the local url
                let format = if state.config.images.use_webp {
                    "webp"
                } else {
                    "png"
                };
                for item in &mut data {
                    item.url = Arc::from(format!(
                        "{}/v1/images/{}.{}",
                        state.config.server.base_url, item.md5, format
                    ));
                }

                // replace the cache with the new data
                *state.maps.write().await = data.clone();

                // drop old images from the cache
                let mut map_cache = state.map_cache.write().await;
                map_cache.retain(|k, _| data.iter().find(|d| d.md5 == *k).is_some());

                info!("Loaded maps.json");
            } else if status == reqwest::StatusCode::NOT_MODIFIED {
                info!("Maps.json not modified");
            } else {
                error!("Failed to load maps.json: {}", status);
            }

            tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;
        }
    }

    async fn load_image(
        state: &ImageState,
        item: &WynntilsMapTile,
        client: reqwest::Client,
        etag_cache: &mut HashMap<Arc<str>, Arc<str>>,
    ) -> Result<(), reqwest::Error> {
        let res = client
            .get(&*item.url)
            .header(
                "If-None-Match",
                &**etag_cache.get(&item.md5).unwrap_or(&Arc::from("")),
            )
            .send()
            .await?;

        let status = res.status();
        if status.is_success() {
            let data = res.bytes().await?;

            let data = if state.config.images.use_webp {
                let img = ImageReader::new(Cursor::new(data))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap();

                let encoder = Encoder::from_image(&img).unwrap();
                let out = encoder.encode_lossless();

                Bytes::from_iter(out.iter().cloned())
            } else {
                data
            };

            state.map_cache.write().await.insert(item.md5.clone(), data);
        } else if status == reqwest::StatusCode::NOT_MODIFIED {
            info!("Image {} not modified", item.url);
        } else {
            error!("Failed to load image {}: {}", item.url, status);
        }

        info!("Loaded image {}", item.url);

        Ok(())
    }
}
