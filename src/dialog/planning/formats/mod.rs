use std::{collections::HashMap, sync::Arc};

use leptos::prelude::ArcRwSignal;
use thiserror::Error;
use wynnmap_types::Guild;

pub mod rueaes;
pub mod urlshare;
pub mod wynnmap;

/// The internal representation of the planning mode data variables
pub struct PlanningModeData {
    pub guilds: Vec<ArcRwSignal<Guild>>,
    pub owned_territories: HashMap<Arc<str>, ArcRwSignal<Guild>>,
}

pub trait DataConvert {
    fn from_data(
        terrs: &HashMap<Arc<str>, wynnmap_types::Territory>,
        guilds: &[ArcRwSignal<Guild>],
        owned: &HashMap<Arc<str>, ArcRwSignal<Guild>>,
    ) -> Self;

    fn to_data(self) -> PlanningModeData;
}

pub trait FileConvert {
    /// Serialize the datastructure to bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// Deserialize this datastructure from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, FileConvertError>
    where
        Self: Sized;
}

#[derive(Debug, Error)]
pub enum FileConvertError {
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
}
