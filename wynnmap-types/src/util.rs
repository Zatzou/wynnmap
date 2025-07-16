use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::ExTerrInfo;

/// Finds the externals of a given territory.
///
/// An external is a territory which is within 3 connections of the given territory.
pub fn find_externals(
    name: &Arc<str>,
    extradata: &HashMap<Arc<str>, ExTerrInfo>,
) -> HashSet<Arc<str>> {
    let mut externals = HashSet::new();

    externals.insert(name.clone());

    for _ in 0..3 {
        let exts = externals.clone();

        for ext in exts {
            if let Some(external) = extradata.get(&ext) {
                for territory in &external.conns {
                    externals.insert(territory.clone());
                }
            }
        }
    }

    externals.remove(name);

    externals
}
