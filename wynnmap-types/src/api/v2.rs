use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct RespWrapper<T> {
    pub data: T,
    pub updated: DateTime<Utc>,
}
