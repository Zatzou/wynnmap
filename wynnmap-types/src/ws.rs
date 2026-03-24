//! Websocket type definitions
//!
//! This module contains the type definitions for the websocket server and client.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::terr::TerrOwner;

/// Messages which may be passed on the websocket
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TerrSockMessage {
    /// Territory capture message
    ///
    /// This message is used to indicate that a territory has been captured. The message contains the territory name, old owner and the new owner.
    Capture {
        name: Arc<str>,
        old: Option<TerrOwner>,
        new: TerrOwner,
    },

    /// Message giving the user the last timestamp of when the territory tracker has received an update from wynn
    ///
    /// This message is used to tell the user if the wynn api is malfunctioning so these malfunctions show up and dont take users by surprise.
    LastUpdate { ts: DateTime<Utc> },
}
