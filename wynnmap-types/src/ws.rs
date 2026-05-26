//! Websocket type definitions
//!
//! This module contains the type definitions for the websocket server and client.

use std::{collections::BTreeMap, sync::Arc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::terr::TerrState;

/// Messages which may be passed on the websocket
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum TerrSockMessage {
    /// Territory update message
    ///
    /// This message gives the client an list with all updated territories. Only certain parts may have changed but the whole list is still sent
    Update(BTreeMap<Arc<str>, TerrState>),

    /// Message giving the user the last timestamp of when the territory tracker has received an update from wynn
    ///
    /// This message is used to tell the user if the wynn api is malfunctioning so these malfunctions show up and dont take users by surprise.
    LastUpdate { ts: DateTime<Utc> },
}
