use std::{collections::HashMap, io::Cursor, sync::Arc, time::Duration};

use axum::body::Bytes;
use image::ImageReader;
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{Instrument, error, info_span};
use webp::Encoder;
use wynnmap_types::{Region, maptile::MapTile};

use crate::{
    AnyError,
    config::Config,
    state::ImageState,
    trackers::util::{self, ResponseExt},
};

pub struct ImageTracker {
    client: reqwest::Client,
    config: Arc<Config>,

    etag_cache: RwLock<HashMap<Arc<str>, Arc<str>>>,

    state: Arc<ImageState>,
}

impl ImageTracker {
    pub fn from_config(config: Arc<Config>) -> Self {
        let client = util::reqwest_client_from_conf(&config);

        Self {
            state: Arc::new(ImageState {
                use_webp: config.images.use_webp,
                ..Default::default()
            }),

            client,
            config,

            etag_cache: Default::default(),
        }
    }

    pub fn run(self) -> Arc<ImageState> {
        let state2 = self.state.clone();

        tokio::spawn(async move {
            let tracker = Arc::new(self);

            loop {
                let tracker = tracker.clone();
                let res = tracker.query_images().await;

                let waittime = match res {
                    Ok(_) => Duration::from_hours(1),
                    Err(e) => {
                        error!(error = ?e, "Error occured while querying extra data");
                        Duration::from_mins(10)
                    }
                };

                tokio::time::sleep(waittime).await;
            }
        });

        state2
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn query_images(self: Arc<Self>) -> Result<(), AnyError> {
        let tiles: Vec<WynntilsMapTile> = async {
            let res = self
                .client
                .get("https://cdn.wynntils.com/static/Reference/maps.json")
                .header(
                    "If-None-Match",
                    &*self
                        .etag_cache
                        .read()
                        .await
                        .get("maps.json")
                        .cloned()
                        .unwrap_or_default(),
                )
                .send()
                .await?;

            if res.status() == StatusCode::NOT_MODIFIED {
                return Ok(Vec::new());
            }

            if let Some(etag) = res.get_header("etag") {
                self.etag_cache
                    .write()
                    .await
                    .insert(Arc::from("maps.json"), Arc::from(etag));
            }

            res.parse_json().await
        }
        .instrument(info_span!("fetch"))
        .await?;

        // empty = 304 Not modified
        if tiles.is_empty() {
            return Ok(());
        }

        // download and process the tiles
        let mut tasks = Vec::new();
        for tile in tiles.clone() {
            let self2 = self.clone();
            let task: JoinHandle<Result<_, AnyError>> = tokio::task::spawn(
                async move {
                    let data = self2.download_image(&tile.url, &tile.name).await?;

                    if let Some(data) = data {
                        let img = if self2.config.images.use_webp {
                            tokio::task::spawn_blocking(|| encode_image(data))
                                .instrument(info_span!("encode_image"))
                                .await??
                        } else {
                            data
                        };

                        Ok(Some((tile.md5.clone(), img)))
                    } else {
                        Ok(None)
                    }
                }
                .instrument(info_span!("load_image")),
            );

            tasks.push(task);
        }

        // reprocess the json data
        let tiles = {
            let format = if self.config.images.use_webp {
                "webp"
            } else {
                "png"
            };

            let mut tiles: Vec<MapTile> = tiles.into_iter().map(Into::into).collect::<Vec<_>>();

            for item in &mut tiles {
                item.url = Arc::from(format!(
                    "{}/api/v1/images/{}.{}",
                    self.config.server.base_url, item.md5, format
                ));
            }

            tiles
        };

        let mut maps_cache = self.state.map_cache.write().await;

        // add the images to the cache
        for task in tasks {
            if let Some((name, img)) = task.await?? {
                maps_cache.insert(name, img);
            }
        }

        let mut maps = self.state.maps.write().await;

        // remove old images from the cache
        maps_cache.retain(|k, _| tiles.iter().any(|d| d.md5 == *k));

        // replace the cache with the new data
        *maps = tiles;

        Ok(())
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn download_image(&self, url: &str, name: &str) -> Result<Option<Bytes>, reqwest::Error> {
        let res = self
            .client
            .get(url)
            .header(
                "If-None-Match",
                &*self
                    .etag_cache
                    .read()
                    .await
                    .get(name)
                    .cloned()
                    .unwrap_or_default(),
            )
            .send()
            .await?;

        if res.status() == StatusCode::NOT_MODIFIED {
            return Ok(None);
        }

        if let Some(etag) = res.get_header("etag") {
            self.etag_cache
                .write()
                .await
                .insert(Arc::from("maps.json"), Arc::from(etag));
        }

        let data = res.bytes().await?;

        Ok(Some(data))
    }
}

fn encode_image(data: Bytes) -> Result<Bytes, AnyError> {
    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;

    let encoder = Encoder::from_image(&img)?;
    let out = encoder.encode_lossless();

    Ok(out.iter().copied().collect())
}

/// The deserialization format for the wynntils `maps.json`
#[derive(Deserialize, Clone)]
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
