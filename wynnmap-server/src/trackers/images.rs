use std::{collections::HashMap, io::Cursor, sync::Arc, time::Duration};

use axum::{body::Bytes, http::HeaderValue};
use image::ImageReader;
use reqwest::{StatusCode, header};
use serde::Deserialize;
use tokio::{
    join,
    task::{JoinSet, spawn_blocking},
};
use tracing::{error, info};
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
    client: Arc<reqwest::Client>,
    base_url: Arc<str>,

    list_etag: HeaderValue,
    etag_cache: HashMap<Arc<str>, Arc<str>>,

    state: Arc<ImageState>,
}

impl ImageTracker {
    pub fn with_config(config: &Config) -> Self {
        let client = Arc::new(util::reqwest_client_from_conf(config));

        Self {
            state: Arc::new(ImageState {
                use_webp: config.images.use_webp,
                ..Default::default()
            }),

            client,
            base_url: config.server.base_url.clone(),

            list_etag: HeaderValue::from_static(""),
            etag_cache: Default::default(),
        }
    }

    pub fn run(self) -> Arc<ImageState> {
        let state2 = self.state.clone();

        tokio::spawn(async move {
            let mut tracker = self;

            loop {
                let res = tracker.query_images().await;

                let waittime = match res {
                    Ok(_) => Duration::from_hours(1),
                    Err(e) => {
                        error!(error = ?e, "Error occured while querying images");
                        Duration::from_mins(10)
                    }
                };

                tokio::time::sleep(waittime).await;
            }
        });

        state2
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn query_images(&mut self) -> Result<(), AnyError> {
        if let Some(tiles) = self.list_tiles().await? {
            let mut tasks = tiles
                .iter()
                .map(|t| self.new_imagetask(t))
                .collect::<Vec<_>>()
                .into_iter();
            let mut joinset = JoinSet::new();

            // run 4 concurrent tasks
            for task in tasks.by_ref().take(4) {
                joinset.spawn(async move { task.execute().await });
            }

            let mut processed_images = Vec::new();

            while let Some(res) = joinset.join_next().await {
                // spawn a new task
                if let Some(task) = tasks.next() {
                    joinset.spawn(async move { task.execute().await });
                }

                if let Some((tile, image, etag)) = res?? {
                    if let Some(etag) = etag {
                        self.etag_cache.insert(tile.name, etag.clone());
                    }

                    let etag = sha224_etag(&image);

                    processed_images.push((tile.md5, (etag, image)));
                }
            }

            let mut cache = self.state.map_cache.write().await;

            for (md5, entry) in processed_images {
                cache.insert(md5, entry);
            }

            // remove old images from the cache
            cache.retain(|k, _| tiles.iter().any(|d| d.md5 == *k));

            drop(cache);

            // reformat the tiles
            let tiles = tiles
                .into_iter()
                .map(MapTile::from)
                .map(|mut tile| {
                    tile.url = format!("{}/api/v1/images/{}", self.base_url, tile.md5).into();
                    tile
                })
                .collect::<Vec<_>>();

            let etag = sha224_etag_json(&tiles);

            {
                let (mut state, mut etag_state) =
                    join!(self.state.maps.write(), self.state.maps_etag.write());

                *state = tiles;
                *etag_state = etag;
            }

            info!("completed image update");
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn list_tiles(&mut self) -> Result<Option<Vec<WynntilsMapTile>>, util::RequestError> {
        let res = self.client.get("https://raw.githubusercontent.com/Wynntils/Static-Storage/refs/heads/main/Reference/maps.json")
            .header(header::ACCEPT, "application/json")
            .header(header::IF_NONE_MATCH, self.list_etag.clone())
            .send()
            .await?;

        if res.status() == StatusCode::NOT_MODIFIED {
            Ok(None)
        } else {
            self.list_etag = res
                .headers()
                .get(header::ETAG)
                .cloned()
                .unwrap_or_else(|| HeaderValue::from_static(""));

            res.parse_json().await
        }
    }

    fn new_imagetask(&self, tile: &WynntilsMapTile) -> ImageTask {
        let tile = tile.clone();

        ImageTask {
            client: self.client.clone(),
            use_webp: self.state.use_webp,
            etag: self.etag_cache.get(&tile.name).cloned().unwrap_or_default(),
            tile,
        }
    }
}

struct ImageTask {
    client: Arc<reqwest::Client>,
    use_webp: bool,

    tile: WynntilsMapTile,

    etag: Arc<str>,
}

impl ImageTask {
    #[tracing::instrument(skip(self), err(Debug))]
    async fn execute(self) -> Result<Option<(WynntilsMapTile, Bytes, Option<Arc<str>>)>, AnyError> {
        let (image, etag) = self.download_image().await?;

        if let Some(image) = image {
            let image = if self.use_webp {
                spawn_blocking(|| ImageTask::encode_image(image)).await??
            } else {
                image
            };

            return Ok(Some((self.tile, image, etag)));
        }

        Ok(None)
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn download_image(
        &self,
    ) -> Result<(Option<Bytes>, Option<Arc<str>>), util::RequestError> {
        let res = self
            .client
            .get(self.tile.url.replace(
                "cdn.wynntils.com/static",
                "raw.githubusercontent.com/Wynntils/Static-Storage/refs/heads/main",
            ))
            .header(header::ACCEPT, "image/png")
            .header(
                header::IF_NONE_MATCH,
                HeaderValue::from_str(&self.etag).unwrap(),
            )
            .send()
            .await?;

        let recv_etag = res.get_header("etag").map(Into::into);

        if res.status() == StatusCode::NOT_MODIFIED {
            return Ok((None, recv_etag));
        }

        let data = res.bytes().await?;

        Ok((Some(data), recv_etag))
    }

    #[tracing::instrument(skip(data), err(Debug))]
    fn encode_image(data: Bytes) -> Result<Bytes, AnyError> {
        let img = ImageReader::new(Cursor::new(data))
            .with_guessed_format()?
            .decode()?;

        let encoder = Encoder::from_image(&img)?;
        let out = encoder.encode_lossless();

        Ok(out.iter().copied().collect())
    }
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
