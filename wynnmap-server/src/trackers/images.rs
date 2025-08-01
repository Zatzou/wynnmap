use std::{collections::HashMap, io::Cursor, sync::Arc};

use crate::{ImageState, config::Config};
use axum::body::Bytes;
use image::ImageReader;
use serde::Deserialize;
use tracing::{error, info};
use webp::Encoder;
use wynnmap_types::{Region, maptile::MapTile};

pub(crate) async fn create_image_tracker(config: Arc<Config>) -> ImageState {
    let state = ImageState {
        config,
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
                    .map_or("", |h| h.to_str().unwrap_or(""));
                etag_cache.insert(Arc::from("maps.json"), Arc::from(etag));

                // load the json
                let mut data: Vec<WynntilsMapTile> = maps_res.json().await?;

                // load each map tile into the cache
                for item in &data {
                    download_image(state, item, client.clone(), &mut etag_cache).await?;
                }

                // replace the urls in the data with the local url
                let format = if state.config.images.use_webp {
                    "webp"
                } else {
                    "png"
                };
                for item in &mut data {
                    item.url = Arc::from(format!(
                        "{}/api/v1/images/{}.{}",
                        state.config.server.base_url, item.md5, format
                    ));
                }

                let tiles = data.into_iter().map(|t| t.into()).collect::<Vec<_>>();

                // replace the cache with the new data
                state.maps.write().await.clone_from(&tiles);

                // drop old images from the cache
                state
                    .map_cache
                    .write()
                    .await
                    .retain(|k, _| tiles.iter().any(|d| d.md5 == *k));

                info!("Loaded maps.json");
            } else if status == reqwest::StatusCode::NOT_MODIFIED {
                info!("Maps.json not modified");
            } else {
                error!("Failed to load maps.json: {}", status);
            }

            tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;
        }
    }
}

/// Download the given file from the github
async fn download_image(
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

            Bytes::from_iter(out.iter().copied())
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

/// The deserialization format for the wynntils `maps.json`
#[derive(Deserialize)]
struct WynntilsMapTile {
    name: Arc<str>,
    url: Arc<str>,

    x1: i32,
    z1: i32,
    x2: i32,
    z2: i32,

    md5: Arc<str>,
}

impl From<WynntilsMapTile> for MapTile {
    fn from(v: WynntilsMapTile) -> Self {
        Self {
            name: v.name,
            url: v.url,
            location: Region {
                start: [v.x1, v.z1],
                end: [v.x2, v.z2],
            },
            md5: v.md5,
        }
    }
}
