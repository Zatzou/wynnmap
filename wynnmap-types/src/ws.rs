//! Websocket type definitions
//!
//! This module contains the type definitions for the websocket server and client.

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::Territory;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TerrSockMessage {
    /// Generic territory update message
    ///
    /// This message is mainly sent when connecting to the websocket. This message is also used for territory updates where there is not necessarirly a capture.
    Territory(HashMap<Arc<str>, Territory>),

    /// Territory capture message
    ///
    /// This message is used to indicate that a territory has been captured. The message contains both the old and new territory data.
    Capture {
        name: Arc<str>,
        old: Territory,
        new: Territory,
    },
}
