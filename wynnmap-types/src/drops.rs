use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

pub type ItemDrops = Vec<Item>;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Item {
    pub name: Arc<str>,
    pub kind: Arc<str>,

    pub dropped_by: BTreeMap<Arc<str>, BTreeSet<DropArea>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DropArea {
    pub location: [i32; 3],
    pub radius: i32,
}

impl From<[i32; 4]> for DropArea {
    fn from(value: [i32; 4]) -> Self {
        let [x, y, z, r] = value;

        Self {
            location: [x, y, z],
            radius: r,
        }
    }
}
