use serde::{Serialize, ser::Error as _};

pub fn encode_data<T: Serialize>(data: &T) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    let data = rmp_serde::to_vec(data)?;

    zstd::stream::encode_all(data.as_slice(), 5).map_err(rmp_serde::encode::Error::custom)
}
