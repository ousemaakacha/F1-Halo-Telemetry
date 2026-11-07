//! Authored miniature ERPK/BXML fixtures; no EA game content is redistributed.
use f1_halo_telemetry::{
    game_data::{BatteryCatalog, capacity_from_bxml, read_archive},
    telemetry::{Aero, CapacitySource, Snapshot, state::Timed},
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

fn record(kind: u8, attributes: u16, body: &[u8]) -> Vec<u8> {
    let mut out = ((body.len() + 4) as u16).to_le_bytes().to_vec();
    out.extend([kind, 0]);
    out.extend(attributes.to_le_bytes());
    out.extend(body);
    out
}

fn document(value: &str) -> Vec<u8> {
    let mut out = b"\x01BXML".to_vec();
    for tag in ["VTF", "EnergyRecoverySystem", "Common"] {
        out.extend(record(0, 0, format!("{tag}\0").as_bytes()));
    }
    out.extend(record(
        0,
        1,
        format!("EnergyStoreCapacity\0value\0{value}\0").as_bytes(),
    ));
    for _ in 0..4 {
        out.extend(record(5, 0, &[]));
    }
    out.extend(record(6, 0, &[]));
    out
}

fn archive(version: u32, payload: &[u8], compressed: bool) -> Vec<u8> {
    // A valid Zstandard single-segment frame with one raw block.
    let packed = if compressed {
        let mut frame = vec![0x28, 0xb5, 0x2f, 0xfd, 0xa0];
        frame.extend((payload.len() as u32).to_le_bytes());
        let block = ((payload.len() as u32) << 3) | 1;
        frame.extend(&block.to_le_bytes()[..3]);
        frame.extend(payload);
        frame
    } else {
        payload.to_vec()
    };
    let name = b"eaid://f1_2026_vehicle_package/teams/example/handling/example.vtf\0";
    let mut entry = (name.len() as u16).to_le_bytes().to_vec();
    entry.extend(name);
    entry.extend([0; 16]);
    entry.extend(vec![0; if version == 4 { 6 } else { 4 }]);
    entry.push(1);
    entry.extend(b"temp");
    entry.extend(0u64.to_le_bytes());
    entry.extend((payload.len() as u64).to_le_bytes());
    entry.extend(0u32.to_le_bytes());
    entry.push(if compressed { 0x11 } else { 0x91 });
    entry.extend((packed.len() as u64).to_le_bytes());
    entry.extend([0; 16]);
    let mut index = 1u32.to_le_bytes().to_vec();
    index.extend(1u32.to_le_bytes());
    index.extend((entry.len() as u32).to_le_bytes());
    index.extend(entry);
    let mut out = b"ERPK".to_vec();
    out.extend(version.to_le_bytes());
    out.extend([0; 8]);
    out.extend(48u64.to_le_bytes());
    out.extend((index.len() as u64).to_le_bytes());
    out.extend((48 + index.len() as u64).to_le_bytes());
    out.extend([0; 8]);
    out.extend(index);
    out.extend(packed);
    out
}

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "f1-battery-test-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn write(&self, relative: &str, data: &[u8]) -> PathBuf {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, data).unwrap();
        path
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn reads_scientific_joules_and_rejects_invalid_or_truncated_bxml() {
    let valid = document("4.0e6");
    assert_eq!(capacity_from_bxml(&valid).unwrap(), Some(4_000_000.0));
    for value in ["NaN", "inf", "-1", "0", "100000001", "wat"] {
        assert!(capacity_from_bxml(&document(value)).is_err(), "{value}");
    }
    for end in 0..valid.len() - 6 {
        assert!(capacity_from_bxml(&valid[..end]).is_err(), "cut {end}");
    }
}

#[test]
fn unrelated_capacity_and_upgrade_values_are_not_used() {
    let mut out = b"\x01BXML".to_vec();
    for tag in ["VTF", "Upgrade", "EnergyRecoverySystem", "Common"] {
        out.extend(record(0, 0, format!("{tag}\0").as_bytes()));
    }
    out.extend(record(0, 1, b"EnergyStoreCapacity\0value\099999999\0"));
    for _ in 0..5 {
        out.extend(record(5, 0, &[]));
    }
    assert_eq!(capacity_from_bxml(&out).unwrap(), None);
    let mut duplicate = document("4e6");
    let last_common_close = duplicate.len() - 24;
    let second = record(0, 1, b"EnergyStoreCapacity\0value\x005e6\0");
    duplicate.splice(
        last_common_close..last_common_close,
        second.into_iter().chain(record(5, 0, &[])),
    );
    assert!(capacity_from_bxml(&duplicate).is_err());
}

#[test]
fn reads_both_erp_versions_and_both_fragment_encodings() {
    let temp = Temp::new();
    for version in [3, 4] {
        for compressed in [false, true] {
            let bytes = archive(version, &document("4200000"), compressed);
            let path = temp.write("example.erp", &bytes);
            let records = read_archive(&path).unwrap();
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].joules, 4_200_000.0);
            for end in [0, 4, 47, 60, bytes.len() - 1] {
                assert!(read_archive(&temp.write("broken.erp", &bytes[..end])).is_err());
            }
        }
    }
}

#[test]
fn malformed_extents_and_decompression_bombs_are_rejected() {
    let temp = Temp::new();
    let mut bytes = archive(3, &document("4e6"), true);
    bytes[24..32].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(read_archive(&temp.write("bad-index.erp", &bytes)).is_err());
    let mut bytes = archive(3, &document("4e6"), true);
    let data = u64::from_le_bytes(bytes[32..40].try_into().unwrap()) as usize;
    bytes[data + 5..data + 9].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(read_archive(&temp.write("bomb.erp", &bytes)).is_err());
    let mut bytes = archive(3, &document("4e6"), false);
    bytes[4..8].copy_from_slice(&99u32.to_le_bytes());
    assert!(read_archive(&temp.write("future.erp", &bytes)).is_err());
}

#[test]
fn catalog_uses_game_values_only_for_matching_formula_and_uniform_cars() {
    let temp = Temp::new();
    for year in [2025, 2026] {
        temp.write(
            &format!("2025_asset_groups/f1_{year}_vehicle_package/teams/common.erp"),
            &archive(3, &document("4500000"), true),
        );
    }
    let mut snapshot = Snapshot {
        formula: Some(13),
        ..Default::default()
    };
    let catalog = BatteryCatalog::from_game_root(&temp.0);
    let capacity = catalog.resolve(&snapshot).unwrap();
    assert_eq!(capacity.joules, 4_500_000.0);
    assert_eq!(capacity.year, 2026);
    assert_eq!(capacity.source, CapacitySource::GameFiles);
    snapshot.formula = Some(2);
    assert!(catalog.resolve(&snapshot).is_none());
    snapshot.formula = None;
    assert!(catalog.resolve(&snapshot).is_none());
    snapshot.aero = Some(Timed {
        value: Aero {
            regulations_2026: true,
            ..Default::default()
        },
        received: Instant::now(),
        overall_frame: 1,
    });
    assert_eq!(catalog.resolve(&snapshot).unwrap().year, 2026);
    temp.write(
        "2025_asset_groups/f1_2026_vehicle_package/teams/example/wep/example.erp",
        &archive(4, &document("6000000"), false),
    );
    let catalog = BatteryCatalog::from_game_root(&temp.0);
    assert!(catalog.resolve(&snapshot).is_none());
    assert!(!catalog.errors.is_empty());
    snapshot.formula = Some(0);
    assert_eq!(catalog.resolve(&snapshot).unwrap().year, 2025);
}
