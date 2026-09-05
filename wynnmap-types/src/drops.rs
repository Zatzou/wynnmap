use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

pub type ItemDrops = Vec<Item>;

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
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
    #[inline]
    fn from(value: [i32; 4]) -> Self {
        let [x, y, z, r] = value;

        Self {
            location: [x, y, z],
            radius: r,
        }
    }
}

impl DropArea {
    #[inline]
    pub fn within(&self, pos: [i32; 2], min_r: i32) -> bool {
        let [x, _, y] = self.location;

        let dist_x = x.abs_diff(pos[0]);
        let dist_y = y.abs_diff(pos[1]);

        let r = self.radius.max(min_r);

        dist_x.pow(2) + dist_y.pow(2) <= r.pow(2) as u32
    }
}
