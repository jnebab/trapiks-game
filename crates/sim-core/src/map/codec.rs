use super::{MapData, MapError, validate};
use crate::MAP_FORMAT_VERSION;
use crate::fnv::fnv1a64;

pub const MAP_MAGIC: [u8; 4] = *b"TRPK";

const HEADER_LEN: usize = 8;

pub fn to_bytes(map: &MapData) -> Result<Vec<u8>, MapError> {
    let mut bytes = Vec::from(MAP_MAGIC);
    bytes.extend_from_slice(&MAP_FORMAT_VERSION.to_le_bytes());
    let payload = postcard::to_allocvec(map).map_err(|e| MapError::Encode(e.to_string()))?;
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

pub fn from_bytes(bytes: &[u8]) -> Result<MapData, MapError> {
    if bytes.len() < HEADER_LEN || bytes[..4] != MAP_MAGIC {
        return Err(MapError::BadMagic);
    }
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != MAP_FORMAT_VERSION {
        return Err(MapError::UnsupportedVersion(version));
    }
    let map: MapData =
        postcard::from_bytes(&bytes[HEADER_LEN..]).map_err(|e| MapError::Decode(e.to_string()))?;
    validate(&map)?;
    Ok(map)
}

pub fn map_hash(bytes: &[u8]) -> u64 {
    fnv1a64(bytes)
}
