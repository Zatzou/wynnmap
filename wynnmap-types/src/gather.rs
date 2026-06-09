use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GatherSpots {
    pub resources: Vec<Material>,
    pub spots: Vec<GatherSpot>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Material {
    pub name: Arc<str>,
    pub level: i32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct GatherSpot {
    pub pos: [i32; 3],
    pub resource: usize,
}

#[derive(Deserialize, Default, Clone)]
pub struct MatData {
    pub prof: Profession,
    pub color: Arc<str>,
}

#[derive(Deserialize, Default, Clone)]
pub enum Profession {
    #[default]
    #[serde(rename = "mining")]
    Mining,
    #[serde(rename = "woodcutting")]
    Woodcutting,
    #[serde(rename = "fishing")]
    Fishing,
    #[serde(rename = "farming")]
    Farming,
}

impl Profession {
    pub const fn color(&self) -> &'static str {
        match self {
            Profession::Mining => "#AA0000",
            Profession::Woodcutting => "#00AA00",
            Profession::Fishing => "#55FFFF",
            Profession::Farming => "#FFFF55",
        }
    }
}
