use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;
use trapiks_sim_core::map::{MapData, to_bytes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteSizes {
    pub raw_bytes: usize,
    pub gzipped_bytes: usize,
}

pub fn write(map: &MapData, path: &str) -> Result<WriteSizes, String> {
    let raw = to_bytes(map).map_err(|e| e.to_string())?;
    let gzipped = gzip(&raw).map_err(|e| e.to_string())?;
    std::fs::write(path, &gzipped).map_err(|e| format!("{path}: {e}"))?;
    Ok(WriteSizes {
        raw_bytes: raw.len(),
        gzipped_bytes: gzipped.len(),
    })
}

fn gzip(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes)?;
    encoder.finish()
}
