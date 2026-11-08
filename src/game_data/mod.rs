//! Reads nominal ERS capacity from the installed game's ERPK/BXML handling data.
//! No process memory is opened, and game files are never written or extracted.
use crate::telemetry::{CapacitySource, ErsCapacity, Snapshot};
use std::{
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

const MAX_INDEX: u64 = 8 * 1024 * 1024;
const MAX_VTF: u64 = 2 * 1024 * 1024;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

struct Bytes<'a> {
    data: &'a [u8],
    at: usize,
}
impl<'a> Bytes<'a> {
    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(count)
            .ok_or_else(|| invalid("offset overflow"))?;
        let value = self
            .data
            .get(self.at..end)
            .ok_or_else(|| invalid("truncated asset"))?;
        self.at = end;
        Ok(value)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn text(&mut self) -> io::Result<&'a str> {
        let rest = self
            .data
            .get(self.at..)
            .ok_or_else(|| invalid("string offset"))?;
        let length = rest
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| invalid("unterminated string"))?;
        let text =
            std::str::from_utf8(self.take(length)?).map_err(|_| invalid("invalid asset UTF-8"))?;
        self.take(1)?;
        Ok(text)
    }
}

#[derive(Debug)]
pub struct CapacityRecord {
    pub archive: PathBuf,
    pub resource: String,
    pub joules: f32,
}

/// A small streaming BXML reader. Only the base VTF/ERS/Common value counts;
/// an R&D upgrade or an unrelated field named Capacity is not a denominator.
pub fn capacity_from_bxml(data: &[u8]) -> io::Result<Option<f32>> {
    if data.len() as u64 > MAX_VTF || !data.starts_with(b"\x01BXML") {
        return Err(invalid("unsupported handling document"));
    }
    let mut cursor = Bytes { data, at: 5 };
    let mut stack: Vec<&str> = Vec::with_capacity(16);
    let mut root_closed = false;
    let mut capacity = None;
    while cursor.at < data.len() {
        let start = cursor.at;
        let length = cursor.u16()? as usize;
        let end = start
            .checked_add(length + 2)
            .ok_or_else(|| invalid("BXML length overflow"))?;
        if length < 4 || end > data.len() {
            return Err(invalid("invalid BXML record length"));
        }
        let kind = cursor.u8()?;
        cursor.u8()?;
        let attributes = cursor.u16()?;
        if attributes > 128 {
            return Err(invalid("too many BXML attributes"));
        }
        let mut body = Bytes {
            data: &data[cursor.at..end],
            at: 0,
        };
        match kind {
            0 => {
                if root_closed || stack.len() >= 64 {
                    return Err(invalid("invalid BXML nesting"));
                }
                let name = body.text()?;
                if stack.is_empty() && name != "VTF" {
                    return Err(invalid("not a vehicle handling document"));
                }
                let mut value = None;
                let mut operation = None;
                for _ in 0..attributes {
                    let key = body.text()?;
                    let text = body.text()?;
                    if key == "value" && value.replace(text).is_some() {
                        return Err(invalid("duplicate BXML value"));
                    }
                    if key == "op" {
                        operation = Some(text);
                    }
                }
                stack.push(name);
                if stack
                    == [
                        "VTF",
                        "EnergyRecoverySystem",
                        "Common",
                        "EnergyStoreCapacity",
                    ]
                {
                    if capacity.is_some() || operation.is_some_and(|v| v != "=") {
                        return Err(invalid("ambiguous nominal battery capacity"));
                    }
                    let number: f64 = value
                        .ok_or_else(|| invalid("missing battery capacity"))?
                        .parse()
                        .map_err(|_| invalid("invalid battery capacity"))?;
                    if !number.is_finite() || number <= 0.0 || number > 100_000_000.0 {
                        return Err(invalid("battery capacity outside supported range"));
                    }
                    capacity = Some(number as f32);
                }
            }
            1 if !root_closed && attributes == 0 => {
                body.text()?;
            }
            5 if length == 4 && attributes == 0 => {
                if stack.pop().is_none() {
                    return Err(invalid("unmatched BXML close"));
                }
                root_closed = stack.is_empty();
            }
            6 if root_closed && length == 4 && attributes == 0 => (),
            _ => return Err(invalid("unsupported BXML record")),
        }
        if body.at != body.data.len() {
            return Err(invalid("BXML record has trailing bytes"));
        }
        cursor.at = end;
    }
    if !root_closed || !stack.is_empty() {
        return Err(invalid("incomplete BXML document"));
    }
    Ok(capacity)
}

fn zstd_window_is_bounded(raw: &[u8]) -> io::Result<()> {
    let mut bytes = Bytes { data: raw, at: 0 };
    if bytes.take(4)? != [0x28, 0xb5, 0x2f, 0xfd] {
        return Err(invalid("not a Zstandard frame"));
    }
    let descriptor = bytes.u8()?;
    if descriptor & 0x18 != 0 {
        return Err(invalid("reserved Zstandard flags"));
    }
    let single = descriptor & 0x20 != 0;
    let window = if !single {
        let descriptor = bytes.u8()?;
        let base = 1u64
            .checked_shl(10 + (descriptor >> 3) as u32)
            .ok_or_else(|| invalid("Zstandard window overflow"))?;
        Some(base + (base / 8) * (descriptor & 7) as u64)
    } else {
        None
    };
    bytes.take([0, 1, 2, 4][(descriptor & 3) as usize])?;
    let content_size = match descriptor >> 6 {
        0 if single => Some(bytes.u8()? as u64),
        0 => None,
        1 => Some(bytes.u16()? as u64 + 256),
        2 => Some(bytes.u32()? as u64),
        _ => Some(bytes.u64()?),
    };
    if window.or(content_size).is_some_and(|n| n > MAX_VTF)
        || content_size.is_some_and(|n| n > MAX_VTF)
    {
        return Err(invalid("handling Zstandard window exceeds memory limit"));
    }
    Ok(())
}

pub fn read_archive(path: &Path) -> io::Result<Vec<CapacityRecord>> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let mut header = [0; 48];
    file.read_exact(&mut header)?;
    let mut h = Bytes {
        data: &header,
        at: 0,
    };
    if h.take(4)? != b"ERPK" {
        return Err(invalid("not an ERP archive"));
    }
    let version = h.u32()?;
    if ![3, 4].contains(&version) {
        return Err(invalid("unsupported ERP version"));
    }
    h.take(8)?;
    let info_start = h.u64()?;
    let info_size = h.u64()?;
    let data_start = h.u64()?;
    if info_start != 48
        || info_size > MAX_INDEX
        || 48u64.checked_add(info_size) != Some(data_start)
        || data_start > file_size
    {
        return Err(invalid("invalid ERP index extent"));
    }
    let mut index = vec![0; info_size as usize];
    file.read_exact(&mut index)?;
    let mut bytes = Bytes {
        data: &index,
        at: 0,
    };
    let resources = bytes.u32()?;
    let _fragments = bytes.u32()?;
    if resources > 100_000 {
        return Err(invalid("too many ERP resources"));
    }
    let mut wanted = Vec::new();
    for _ in 0..resources {
        let entry_length = bytes.u32()? as usize;
        let end = bytes
            .at
            .checked_add(entry_length)
            .ok_or_else(|| invalid("ERP entry overflow"))?;
        let record = bytes.take(entry_length)?;
        let mut entry = Bytes {
            data: record,
            at: 0,
        };
        let name_len = entry.u16()? as usize;
        let name = std::str::from_utf8(entry.take(name_len)?)
            .map_err(|_| invalid("invalid ERP name"))?
            .trim_end_matches('\0');
        entry.take(16 + if version == 4 { 6 } else { 4 })?;
        let fragment_count = entry.u8()?;
        for fragment in 0..fragment_count {
            entry.take(4)?;
            let offset = entry.u64()?;
            let size = entry.u64()?;
            entry.u32()?;
            let compression = entry.u8()?;
            let packed = entry.u64()?;
            let absolute = data_start
                .checked_add(offset)
                .ok_or_else(|| invalid("ERP fragment overflow"))?;
            if absolute.checked_add(packed).is_none_or(|n| n > file_size) {
                return Err(invalid("ERP fragment outside archive"));
            }
            if fragment == 0 && name.contains("/handling/") && name.ends_with(".vtf") {
                if size > MAX_VTF || packed > MAX_VTF || size == 0 {
                    return Err(invalid("handling resource too large"));
                }
                wanted.push((name.to_owned(), absolute, size, compression, packed));
            }
        }
        entry.take(16)?;
        if entry.at != entry.data.len() || bytes.at != end {
            return Err(invalid("invalid ERP record length"));
        }
    }
    if bytes.at != bytes.data.len() {
        return Err(invalid("ERP index has trailing bytes"));
    }
    let mut records = Vec::new();
    for (resource, absolute, size, compression, packed) in wanted {
        file.seek(SeekFrom::Start(absolute))?;
        let mut raw = vec![0; packed as usize];
        file.read_exact(&mut raw)?;
        let decoded = match compression {
            0x91 if packed == size => raw,
            0x11 => {
                zstd_window_is_bounded(&raw)?;
                let decoder = ruzstd::decoding::StreamingDecoder::new(raw.as_slice())
                    .map_err(io::Error::other)?;
                let mut decoded = Vec::with_capacity(size as usize);
                decoder.take(MAX_VTF + 1).read_to_end(&mut decoded)?;
                if decoded.len() as u64 != size {
                    return Err(invalid("handling decompressed size mismatch"));
                }
                decoded
            }
            _ => return Err(invalid("unsupported handling compression")),
        };
        let joules = capacity_from_bxml(&decoded)?
            .ok_or_else(|| invalid("handling has no nominal ERS capacity"))?;
        records.push(CapacityRecord {
            archive: path.to_owned(),
            resource,
            joules,
        });
    }
    Ok(records)
}

#[derive(Default, Debug)]
pub struct BatteryCatalog {
    pub game_root: Option<PathBuf>,
    pub records: Vec<CapacityRecord>,
    pub errors: Vec<String>,
    capacities: [Option<f32>; 2],
}

impl BatteryCatalog {
    pub fn discover() -> Self {
        let Some(root) = discover_game() else {
            return Self::default();
        };
        Self::from_game_root(&root)
    }

    pub fn from_game_root(root: &Path) -> Self {
        let mut catalog = Self {
            game_root: Some(root.to_owned()),
            ..Self::default()
        };
        for (slot, year) in [2025, 2026].into_iter().enumerate() {
            let directory = root.join(format!("2025_asset_groups/f1_{year}_vehicle_package/teams"));
            let mut paths = vec![directory.join("common.erp")];
            if let Ok(teams) = fs::read_dir(&directory) {
                for team in teams
                    .flatten()
                    .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                {
                    let name = team.file_name();
                    let canonical = team.path().join("wep").join(&name).with_extension("erp");
                    if canonical.is_file() {
                        paths.push(canonical);
                    }
                }
            }
            paths.sort();
            let first = catalog.records.len();
            let mut failed = false;
            for path in paths {
                match read_archive(&path) {
                    Ok(records) => catalog.records.extend(records),
                    Err(error) => {
                        failed = true;
                        catalog.errors.push(format!("{}: {error}", path.display()));
                    }
                }
            }
            let records = &catalog.records[first..];
            let uniform = records
                .first()
                .map(|r| r.joules)
                .filter(|&v| records.iter().all(|r| r.joules == v));
            // shortcut: mixed per-car capacities are rejected; add team/equal-performance
            // routing before supporting mods with different nominal capacities per car.
            if !failed {
                catalog.capacities[slot] = uniform;
            }
            if uniform.is_none() && !records.is_empty() {
                catalog
                    .errors
                    .push(format!("{year}: ERS capacities differ between cars"));
            }
        }
        catalog
    }

    pub fn resolve(&self, snapshot: &Snapshot) -> Option<ErsCapacity> {
        let year = match snapshot.formula {
            Some(13) => 2026,
            Some(0) => 2025,
            Some(_) => return None,
            None if snapshot.aero.is_some_and(|a| a.value.regulations_2026) => 2026,
            None => return None,
        };
        self.capacities[usize::from(year == 2026)].map(|joules| ErsCapacity {
            joules,
            year,
            source: CapacitySource::GameFiles,
        })
    }
}

fn is_game(path: &Path) -> bool {
    path.join("F1_25.exe").is_file()
        && path
            .join("2025_asset_groups/f1_2026_vehicle_package/teams/common.erp")
            .is_file()
}

pub fn discover_game() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("F1_25_DIR") {
        candidates.push(PathBuf::from(path));
    }
    #[cfg(windows)]
    candidates.extend(windows_paths::running_game());
    let mut steam_roots = Vec::new();
    #[cfg(windows)]
    steam_roots.extend(windows_paths::steam_root());
    for variable in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Some(root) = std::env::var_os(variable) {
            steam_roots.push(PathBuf::from(&root).join("Steam"));
            candidates.push(PathBuf::from(root).join("EA Games/F1 25"));
        }
    }
    let mut libraries = steam_roots.clone();
    for root in steam_roots {
        if let Ok(vdf) = fs::read_to_string(root.join("steamapps/libraryfolders.vdf")) {
            libraries.extend(vdf_values(&vdf, "path").into_iter().map(PathBuf::from));
        }
    }
    for library in libraries {
        let steamapps = library.join("steamapps");
        candidates.push(steamapps.join("common/F1 25"));
        if let Ok(entries) = fs::read_dir(&steamapps) {
            for entry in entries
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().starts_with("appmanifest_"))
            {
                if let Ok(manifest) = fs::read_to_string(entry.path()) {
                    for folder in vdf_values(&manifest, "installdir") {
                        candidates.push(steamapps.join("common").join(folder));
                    }
                }
            }
        }
    }
    candidates.into_iter().find(|p| is_game(p))
}

fn vdf_values(text: &str, wanted: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '"' {
            continue;
        }
        let mut value = String::new();
        while let Some(c) = chars.next() {
            match c {
                '"' => break,
                '\\' => {
                    if let Some(next) = chars.next() {
                        value.push(next);
                    }
                }
                _ => value.push(c),
            }
        }
        tokens.push(value);
    }
    tokens
        .windows(2)
        .filter(|pair| pair[0] == wanted)
        .map(|pair| pair[1].clone())
        .collect()
}

#[cfg(windows)]
mod windows_paths {
    use std::path::PathBuf;
    use windows::{
        Win32::{
            Foundation::*,
            System::{Diagnostics::ToolHelp::*, Registry::*, Threading::*},
        },
        core::w,
    };

    pub fn steam_root() -> Option<PathBuf> {
        let mut buffer = [0u16; 4096];
        let mut size = std::mem::size_of_val(&buffer) as u32;
        // SAFETY: registry is read-only; buffer size is given in bytes.
        unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                w!("Software\\Valve\\Steam"),
                w!("SteamPath"),
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&mut size),
            )
            .ok()
            .ok()?;
        }
        let count = buffer.iter().position(|&v| v == 0).unwrap_or(buffer.len());
        Some(PathBuf::from(String::from_utf16_lossy(&buffer[..count])))
    }

    pub fn running_game() -> Option<PathBuf> {
        // SAFETY: only process metadata is queried. No PROCESS_VM_READ, memory
        // access, debugging, injection or code loading is requested.
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut found = None;
            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let end = entry
                        .szExeFile
                        .iter()
                        .position(|&v| v == 0)
                        .unwrap_or(entry.szExeFile.len());
                    if String::from_utf16_lossy(&entry.szExeFile[..end])
                        .eq_ignore_ascii_case("F1_25.exe")
                    {
                        if let Ok(process) = OpenProcess(
                            PROCESS_QUERY_LIMITED_INFORMATION,
                            false,
                            entry.th32ProcessID,
                        ) {
                            let mut path = [0u16; 32768];
                            let mut size = path.len() as u32;
                            if QueryFullProcessImageNameW(
                                process,
                                PROCESS_NAME_WIN32,
                                windows::core::PWSTR(path.as_mut_ptr()),
                                &mut size,
                            )
                            .is_ok()
                            {
                                found =
                                    PathBuf::from(String::from_utf16_lossy(&path[..size as usize]))
                                        .parent()
                                        .map(PathBuf::from);
                            }
                            let _ = CloseHandle(process);
                        }
                        break;
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
            found
        }
    }
}
