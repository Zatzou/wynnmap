use std::{collections::HashMap, sync::Arc};

use leptos::prelude::{ArcRwSignal, GetUntracked};
use serde::{Deserialize, Serialize};
use wynnmap_types::Guild;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "version", content = "content")]
pub enum WynnmapData {
    V1 {
        guilds: Vec<V1Guild>,
        territories: HashMap<String, V1Territory>,
    },
}

impl WynnmapData {
    pub fn from_data(
        terrs: &HashMap<Arc<str>, wynnmap_types::Territory>,
        guilds: &Vec<ArcRwSignal<Guild>>,
        owned: &HashMap<Arc<str>, ArcRwSignal<Guild>>,
    ) -> Self {
        let mut newguilds = Vec::new();

        for guild in guilds {
            newguilds.push(guild.get_untracked().into());
        }

        let mut newterrs = HashMap::new();

        for terr in terrs {
            let t = V1Territory {
                location: terr.1.location.clone(),
                owner: if let Some(own) = owned.get(terr.0) {
                    guilds
                        .iter()
                        .enumerate()
                        .find(|(_, g)| *g == own)
                        .map(|(i, _)| i)
                        .unwrap_or(0)
                } else {
                    0
                },
            };

            newterrs.insert(terr.0.to_string(), t);
        }

        Self::V1 {
            guilds: newguilds,
            territories: newterrs,
        }
    }

    pub fn into_data(
        self,
    ) -> (
        Vec<ArcRwSignal<Guild>>,
        HashMap<Arc<str>, ArcRwSignal<Guild>>,
    ) {
        let Self::V1 {
            guilds,
            territories,
        } = self;

        let mut guilds2 = vec![];

        for guild in guilds {
            guilds2.push(ArcRwSignal::new(guild.into()));
        }

        let mut terrs2 = HashMap::new();

        // TODO: load the entire territories from the file once that is implemented
        for (name, terr) in territories {
            let guild = guilds2
                .iter()
                .enumerate()
                .find(|(i, _)| *i == terr.owner)
                .map(|(_, g)| g)
                .unwrap_or_else(|| guilds2.first().unwrap());

            terrs2.insert(Arc::from(name), guild.clone());
        }

        (guilds2, terrs2)
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        serde_json::from_slice(&bytes).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V1Guild {
    name: String,
    prefix: String,
    color: String,
}

impl From<Guild> for V1Guild {
    fn from(value: Guild) -> Self {
        let col = value.hex_color();

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
            color: Some(Arc::from(value.color)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V1Territory {
    location: wynnmap_types::Location,
    owner: usize,
}
