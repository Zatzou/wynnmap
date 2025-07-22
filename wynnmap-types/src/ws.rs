//! Websocket type definitions
//!
//! This module contains the type definitions for the websocket server and client.

use std::sync::Arc;

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
}
