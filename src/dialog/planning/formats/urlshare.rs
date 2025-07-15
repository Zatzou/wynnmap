use std::{collections::HashMap, sync::Arc};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use bitcode::{Decode, Encode};
use leptos::{
    leptos_dom::logging::console_log,
    prelude::{ArcRwSignal, GetUntracked},
};
use thiserror::Error;
use wynnmap_types::Guild;

#[derive(Debug, Clone, Encode, Decode)]
pub enum WynnmapData {
    V1 {
        terrhash: u32,
        guilds: Vec<V1Guild>,
        territories: Vec<usize>,
    },
}

impl WynnmapData {
    pub fn from_data(
        terrs: &HashMap<Arc<str>, wynnmap_types::Territory>,
        guilds: &[ArcRwSignal<Guild>],
        owned: &HashMap<Arc<str>, ArcRwSignal<Guild>>,
    ) -> Self {
        let mut newguilds: Vec<V1Guild> = Vec::new();

        for guild in guilds.iter().skip(1) {
            newguilds.push(guild.get_untracked().into());
        }

        let mut terrnames = terrs.keys().map(Clone::clone).collect::<Vec<_>>();

        // sort the terr names
        terrnames.sort();

        // generate the terr name hash to check if terrs have updated
        let mut hasher = crc32fast::Hasher::new();
        for t in &terrnames {
            hasher.update(t.as_bytes());
        }
        let terrhash = hasher.finalize();

        let mut owneds = Vec::new();

        for territory in terrnames {
            if let Some(owner) = owned.get(&territory) {
                // figure out the index to the guilds array
                let idx = guilds
                    .iter()
                    .enumerate()
                    .find(|(_, g)| *g == owner)
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                owneds.push(idx);
            } else {
                owneds.push(0);
            }
        }

        Self::V1 {
            terrhash,
            guilds: newguilds,
            territories: owneds,
        }
    }

    /// Turn the url share back into the planning mode data
    pub fn into_data(
        self,
        terrs: &HashMap<Arc<str>, wynnmap_types::Territory>,
    ) -> (
        Vec<ArcRwSignal<Guild>>,
        HashMap<Arc<str>, ArcRwSignal<Guild>>,
    ) {
        let Self::V1 {
            terrhash: _,
            guilds,
            territories,
        } = self;

        let mut guilds2 = vec![ArcRwSignal::new(Guild::default())];

        for guild in guilds {
            guilds2.push(ArcRwSignal::new(guild.into()));
        }

        let mut terrnames = terrs.keys().map(Clone::clone).collect::<Vec<_>>();

        // sort the terr names
        terrnames.sort();

        let mut terrs2 = HashMap::new();

        for (name, idx) in terrnames.into_iter().zip(territories) {
            let guild = guilds2
                .iter()
                .enumerate()
                .find(|(i, _)| *i == idx)
                .map(|(_, g)| g)
                .unwrap_or_else(|| guilds2.first().unwrap());

            terrs2.insert(name, guild.clone());
        }

        (guilds2, terrs2)
    }

    pub fn from_string(input: impl AsRef<str>) -> Result<Self, UrlShareDecodeError> {
        let compressed_bytes = URL_SAFE_NO_PAD.decode(input.as_ref())?;

        let decompressed_bytes = zstd::decode_all(compressed_bytes.as_slice())?;

        let data = bitcode::decode(&decompressed_bytes)?;

        Ok(data)
    }

    pub fn to_string(&self) -> String {
        let bytes = bitcode::encode(self);

        let zstd = zstd::encode_all(bytes.as_slice(), 22).unwrap();

        URL_SAFE_NO_PAD.encode(zstd)
    }

    pub fn verify_terrhash(&self, terrs: &HashMap<Arc<str>, wynnmap_types::Territory>) -> bool {
        let mut terrnames = terrs.keys().map(Clone::clone).collect::<Vec<_>>();

        // sort the terr names
        terrnames.sort();

        // generate the terr name hash to check if terrs have updated
        let mut hasher = crc32fast::Hasher::new();
        for t in &terrnames {
            hasher.update(t.as_bytes());
        }
        let terrhash2 = hasher.finalize();

        let Self::V1 { terrhash, .. } = self;

        console_log(&format!("th1: {} th2: {}", terrhash, terrhash2));

        *terrhash == terrhash2
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct V1Guild {
    name: String,
    prefix: String,
    color: (u8, u8, u8),
}

impl From<Guild> for V1Guild {
    fn from(value: Guild) -> Self {
        let col = value.get_color();

        Self {
            name: value.name.unwrap_or_default().to_string(),
            prefix: value.prefix.unwrap_or_default().to_string(),
            color: col,
        }
    }
}

impl From<V1Guild> for Guild {
    fn from(value: V1Guild) -> Self {
        Self {
            uuid: None,
            name: Some(Arc::from(value.name)),
            prefix: Some(Arc::from(value.prefix)),
            color: Some(Arc::from(format!(
                "#{:02X}{:02X}{:02X}",
                value.color.0, value.color.1, value.color.2
            ))),
        }
    }
}

#[derive(Debug, Error)]
pub enum UrlShareDecodeError {
    #[error("Failed to decode share string using base64: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Failed to decompress share string: {0}")]
    DecompressError(#[from] std::io::Error),
    #[error("Failed to decode share string: {0}")]
    DecodeError(#[from] bitcode::Error),
}
