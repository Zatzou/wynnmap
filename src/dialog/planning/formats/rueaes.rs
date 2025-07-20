use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use leptos::prelude::{ArcRwSignal, GetUntracked};
use serde::{Deserialize, Serialize};
use wynnmap_types::{Guild, Location};

/// Base structure for the Ruea economy studio file format
#[derive(Debug, Deserialize, Serialize)]
pub struct RueaES {
    /// Type of data should be "state_save"
    #[serde(rename = "type")]
    type_: String,
    /// Version of the data. At the time of writing "1.1"
    version: String,
    timestamp: String,

    /// Tick in the simulation
    tick: i64,

    territories: Vec<RTerritory>,
    guilds: Vec<RGuild>,

    #[serde(rename = "totalTerritories")]
    total_territories: usize,
    #[serde(rename = "totalGuilds")]
    total_guilds: usize,
}

impl RueaES {
    pub fn from_data(
        terrs: &HashMap<Arc<str>, wynnmap_types::Territory>,
        guilds: &Vec<ArcRwSignal<Guild>>,
        owned: &HashMap<Arc<str>, ArcRwSignal<Guild>>,
    ) -> Self {
        let guilds2 = guilds
            .iter()
            .map(|g| g.get_untracked().into())
            .collect::<Vec<RGuild>>();

        let mut terrs2 = Vec::new();

        for (name, terr) in terrs {
            let owner = owned.get(name).unwrap_or_else(|| guilds.first().unwrap());

            terrs2.push(RTerritory {
                name: name.to_string(),
                guild: owner.get_untracked().into(),
                location: terr.location.clone(),
            });
        }

        let tlen = terrs2.len();
        let glen = guilds2.len();

        Self {
            type_: String::from("state_save"),
            version: String::from("1.1"),
            timestamp: chrono::Utc::now().to_rfc3339(),
            tick: 0,
            territories: terrs2,
            guilds: guilds2,
            total_territories: tlen,
            total_guilds: glen,
        }
    }

    pub fn into_data(
        &self,
    ) -> (
        Vec<ArcRwSignal<Guild>>,
        HashMap<Arc<str>, ArcRwSignal<Guild>>,
    ) {
        // convert the guilds
        // first guild will always be [None]
        let mut guilds = vec![ArcRwSignal::new(Guild::default())];

        // for guild in &self.guilds {
        //     if guild.tag.to_lowercase() != "none" {
        //         guilds.push(ArcRwSignal::new(guild.into()));
        //     }
        // }

        // collect guilds from the territories to avoid getting guilds which are not on the map
        let mut gs = HashSet::new();
        for terr in &self.territories {
            if terr.guild.tag.to_lowercase() != "none" && !gs.contains(&terr.guild) {
                gs.insert(terr.guild.clone());
            }
        }

        for gu in gs {
            guilds.push(ArcRwSignal::new(gu.into()));
        }

        // then convert the territories
        let mut terrs = HashMap::new();

        for terr in &self.territories {
            let guildref = guilds
                .iter()
                .find(|g| {
                    g.get_untracked()
                        .prefix
                        .is_some_and(|pfx| pfx.to_string() == terr.guild.tag)
                })
                .cloned()
                .unwrap_or_else(|| guilds.first().unwrap().clone());

            terrs.insert(Arc::from(terr.name.as_ref()), guildref);
        }

        (guilds, terrs)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let out = Vec::new();

        let mut writer = lz4_flex::frame::FrameEncoder::new(out);

        serde_json::to_writer(&mut writer, self).expect("Serialization should not fail");

        writer.finish().expect("compression should not fail")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let mut decomp = lz4_flex::frame::FrameDecoder::new(bytes);

        serde_json::from_reader(&mut decomp)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct RTerritory {
    name: String,
    guild: RGuild,
    location: Location,
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
struct RGuild {
    name: String,
    tag: String,
}

impl From<Guild> for RGuild {
    fn from(value: Guild) -> Self {
        Self {
            name: value.name.unwrap_or_default().to_string(),
            tag: value.prefix.unwrap_or_default().to_string(),
        }
    }
}

impl From<RGuild> for Guild {
    fn from(value: RGuild) -> Self {
        Self {
            uuid: None,
            name: Some(Arc::from(value.name)),
            prefix: Some(Arc::from(value.tag)),
            color: None,
        }
    }
}
