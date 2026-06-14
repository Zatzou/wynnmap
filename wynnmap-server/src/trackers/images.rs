use std::{collections::HashMap, io::Cursor, sync::Arc, time::Duration};

use axum::body::Bytes;
use image::ImageReader;
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::{sync::RwLock, task::JoinSet};
use tracing::{Instrument, error, info, info_span};
use webp::Encoder;
use wynnmap_types::{Region, maptile::MapTile};

use crate::{
    AnyError,
    config::Config,
    etag::{sha224_etag, sha224_etag_json},
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
                .get("https://raw.githubusercontent.com/Wynntils/Static-Storage/refs/heads/main/Reference/maps.json")
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

        // download and process the tiles, limit concurrency to 4
        let mut processed_images = Vec::new();
        for chunk in tiles.chunks(4) {
            let mut tasks: JoinSet<Result<_, AnyError>> = JoinSet::new();

            for tile in chunk {
                let etag = self
                    .etag_cache
                    .read()
                    .await
                    .get(&tile.name)
                    .cloned()
                    .unwrap_or_default();
                let tile = tile.clone();
                let self2 = self.clone();
                tasks.spawn(
                    async move {
                        let data = self2.download_image(&tile.url, &etag).await?;

                        if let (Some(data), etag) = data {
                            let img = if self2.config.images.use_webp {
                                tokio::task::spawn_blocking(|| encode_image(data))
                                    .instrument(info_span!("encode_image"))
                                    .await??
                            } else {
                                data
                            };

                            Ok(Some(((tile.md5.clone(), img), (tile.name.clone(), etag))))
                        } else {
                            Ok(None)
                        }
                    }
                    .instrument(info_span!("load_image")),
                );
            }

            for res in tasks.join_all().await {
                if let Some(res) = res? {
                    processed_images.push(res);
                }
            }
        }

        let mut maps_cache = self.state.map_cache.write().await;
        let mut etags_cache = self.etag_cache.write().await;

        // add the images to the cache
        for ((name, img), (tname, t_etag)) in processed_images {
            if let Some(etag) = t_etag {
                etags_cache.insert(tname, etag.into());
            }

            let etag = sha224_etag(&img);
            maps_cache.insert(name, (etag, img));
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

        let mut maps = self.state.maps.write().await;
        let mut maps_etag = self.state.maps_etag.write().await;

        // remove old images from the cache
        maps_cache.retain(|k, _| tiles.iter().any(|d| d.md5 == *k));

        // calculate the new etag
        *maps_etag = sha224_etag_json(&tiles);

        // replace the cache with the new data
        *maps = tiles;

        info!("completed image update");

        Ok(())
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn download_image(
        &self,
        url: &str,
        etag: &str,
    ) -> Result<(Option<Bytes>, Option<String>), reqwest::Error> {
        let res = self
            .client
            .get(url.replace("cdn.wynntils.com/static", "raw.githubusercontent.com/Wynntils/Static-Storage/refs/heads/main"))
            .header("If-None-Match", etag)
            .send()
            .await?;

        if res.status() == StatusCode::NOT_MODIFIED {
            return Ok((None, None));
        }

        let recv_etag = res.get_header("etag").map(String::from);

        let data = res.bytes().await?;

        Ok((Some(data), recv_etag))
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
