use std::io::Read;
use std::process::Command;

use flate2::read::GzDecoder;
use trapiks_sim_core::map::from_bytes;

#[test]
fn synthetic_cli() {
    let out = std::env::temp_dir().join(format!("trapiks-synthetic-{}.bin.gz", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_trapiks-mapgen"))
        .arg("--out")
        .arg(&out)
        .args(["--synthetic", "10x10@150"])
        .output();
    let Ok(output) = status else {
        panic!("mapgen did not run");
    };
    assert!(output.status.success());
    let gzipped = std::fs::read(&out).unwrap_or_default();
    let _ = std::fs::remove_file(&out);
    let mut raw = Vec::new();
    let decoded = GzDecoder::new(gzipped.as_slice()).read_to_end(&mut raw);
    assert!(decoded.is_ok());
    let Ok(map) = from_bytes(&raw) else {
        panic!("decode failed");
    };
    assert_eq!(map.road_count(), 224);
    assert_eq!(map.areas.kind.len(), 2);
}
