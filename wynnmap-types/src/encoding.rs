use serde::{
    Serialize,
    de::{DeserializeOwned, Error as _},
    ser::Error as _,
};

pub fn encode_data<T: Serialize>(data: &T) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    let data = rmp_serde::to_vec(data)?;

    zstd::stream::encode_all(data.as_slice(), 5).map_err(rmp_serde::encode::Error::custom)
}

pub fn decode_data<T: DeserializeOwned>(data: &[u8]) -> Result<T, rmp_serde::decode::Error> {
    let data = zstd::stream::decode_all(data).map_err(rmp_serde::decode::Error::custom)?;

    rmp_serde::from_slice(&data)
}
