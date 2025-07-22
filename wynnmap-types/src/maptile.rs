use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::Region;

/// A map tile image
#[derive(Clone, Deserialize, Serialize)]
pub struct MapTile {
    /// The name of the map tile
    pub name: Arc<str>,

    /// The url where the map tile image can be found.
    pub url: Arc<str>,

    /// The location of this map tile
    pub location: Region,

    /// The md5 hash of the original map tile.
    ///
    /// This may not necessarirly be the hash of the file returned by the url, but it is the hash of the original file the file is encoded from.
    pub md5: Arc<str>,
}
