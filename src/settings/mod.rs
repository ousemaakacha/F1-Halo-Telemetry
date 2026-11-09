use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs, io,
    net::SocketAddr,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum TyreMode {
    Surface,
    Inner,
    #[default]
    Both,
}

impl TyreMode {
    pub fn next(self) -> Self {
        match self {
            Self::Surface => Self::Inner,
            Self::Inner => Self::Both,
            Self::Both => Self::Surface,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Surface => "Surface",
            Self::Inner => "Inner",
            Self::Both => "Both",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum HaloModel {
    #[default]
    Wide,
    Classic,
    Modern,
}

impl HaloModel {
    pub const ALL: [Self; 3] = [Self::Wide, Self::Classic, Self::Modern];

    pub fn label(self) -> &'static str {
        match self {
            Self::Wide => "Wide",
            Self::Classic => "Classic",
            Self::Modern => "modern",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum FpsLimit {
    #[default]
    Fps60,
    Fps120,
    Adaptive,
}

impl FpsLimit {
    pub fn hz(self, monitor_hz: u32) -> u32 {
        match self {
            Self::Fps60 => 60,
            Self::Fps120 => 120,
            Self::Adaptive => monitor_hz.clamp(30, 360),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Layout {
    pub x: i32,
    pub y: i32,
    pub scale: f32,
    pub opacity: f32,
    pub visible: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            x: 650,
            y: 700,
            scale: 1.0,
            opacity: 0.94,
            visible: true,
        }
    }
}

impl Layout {
    pub fn with_global_visibility(mut self, visible: bool) -> Self {
        self.visible &= visible;
        self
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct HaloProfile {
    pub x: i32,
    pub y: i32,
    pub scale: f32,
}

impl HaloProfile {
    pub fn apply(self, layout: &mut Layout) {
        layout.x = self.x;
        layout.y = self.y;
        layout.scale = self.scale;
    }

    fn validate(self) -> Result<(), String> {
        if !self.scale.is_finite() || !(0.35..=3.0).contains(&self.scale) {
            return Err("Halo profile scale: range 0.35–3.00".into());
        }
        Ok(())
    }
}

fn profile_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() || name.encode_utf16().count() > 64 || name.chars().any(char::is_control) {
        return Err("Profile name: 1 to 64 characters, without control characters.".into());
    }
    Ok(name)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Colours {
    pub accent: [u8; 3],
    pub throttle: [u8; 3],
    pub brake: [u8; 3],
    pub cold: [u8; 3],
    pub optimal: [u8; 3],
    pub hot: [u8; 3],
}

impl Default for Colours {
    fn default() -> Self {
        Self {
            accent: [255, 145, 40],
            throttle: [65, 235, 24],
            brake: [255, 48, 40],
            cold: [83, 163, 255],
            optimal: [71, 229, 142],
            hot: [255, 102, 78],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub schema_version: u32,
    pub bind_ip: String,
    pub port: u16,
    pub halo: Layout,
    pub halo_model: HaloModel,
    pub halo_profiles: BTreeMap<String, HaloProfile>,
    pub active_halo_profile: Option<String>,
    pub hide_halo_rear_view: bool,
    pub tyres: Layout,
    pub tyre_mode: TyreMode,
    pub temperature_min_c: f32,
    pub temperature_max_c: f32,
    pub wear_warning_percent: f32,
    pub wear_critical_percent: f32,
    pub colours: Colours,
    pub fps: FpsLimit,
    pub hotkey: String,
    pub visibility_hotkey: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            bind_ip: "127.0.0.1".into(),
            port: 20777,
            halo: Layout::default(),
            halo_model: HaloModel::default(),
            halo_profiles: BTreeMap::new(),
            active_halo_profile: None,
            hide_halo_rear_view: true,
            tyres: Layout {
                x: 45,
                y: 600,
                ..Layout::default()
            },
            tyre_mode: TyreMode::Both,
            temperature_min_c: 85.0,
            temperature_max_c: 105.0,
            wear_warning_percent: 35.0,
            wear_critical_percent: 65.0,
            colours: Colours::default(),
            fps: FpsLimit::Fps60,
            hotkey: "Ctrl+Shift+H".into(),
            visibility_hotkey: "Ctrl+Shift+O".into(),
        }
    }
}

impl Settings {
    fn canonical_profile_name(&self, name: &str) -> Result<String, String> {
        let name = profile_name(name)?;
        let lower = name.to_lowercase();
        Ok(self
            .halo_profiles
            .keys()
            .find(|key| key.to_lowercase() == lower)
            .map(String::as_str)
            .unwrap_or(name)
            .to_owned())
    }

    pub fn save_halo_profile(&mut self, name: &str) -> Result<String, String> {
        let name = self.canonical_profile_name(name)?;
        let profile = HaloProfile {
            x: self.halo.x,
            y: self.halo.y,
            scale: self.halo.scale,
        };
        profile.validate()?;
        self.halo_profiles.insert(name.clone(), profile);
        self.active_halo_profile = Some(name.clone());
        Ok(name)
    }

    pub fn load_halo_profile(&mut self, name: &str) -> Result<String, String> {
        let name = self.canonical_profile_name(name)?;
        let profile = self
            .halo_profiles
            .get(&name)
            .ok_or("Profile not found. Choose a saved name or use Save profile.")?;
        profile.validate()?;
        profile.apply(&mut self.halo);
        self.active_halo_profile = Some(name.clone());
        Ok(name)
    }

    pub fn effective_layouts(&self, visible: bool, rear_view: bool) -> [Layout; 2] {
        [
            self.halo.with_global_visibility(visible && !rear_view),
            self.tyres.with_global_visibility(visible),
        ]
    }

    pub fn endpoint(&self) -> Result<SocketAddr, String> {
        let ip = self
            .bind_ip
            .parse()
            .map_err(|_| "Invalid listening IP address".to_string())?;
        if self.port == 0 {
            return Err("Port must be between 1 and 65535".into());
        }
        Ok(SocketAddr::new(ip, self.port))
    }

    pub fn validate(&self) -> Result<(), String> {
        self.endpoint()?;
        if self.schema_version != 1 {
            return Err("Unsupported settings version".into());
        }
        for (name, profile) in &self.halo_profiles {
            if profile_name(name)? != name {
                return Err("Saved profile name has leading or trailing spaces.".into());
            }
            profile.validate()?;
        }
        if self
            .active_halo_profile
            .as_ref()
            .is_some_and(|name| !self.halo_profiles.contains_key(name))
        {
            return Err("The selected Halo profile does not exist.".into());
        }
        for layout in [self.halo, self.tyres] {
            if !layout.scale.is_finite() || !(0.35..=3.0).contains(&layout.scale) {
                return Err("HUD scale: range 0.35–3.00".into());
            }
            if !layout.opacity.is_finite() || !(0.15..=1.0).contains(&layout.opacity) {
                return Err("HUD opacity: range 0.15–1.00".into());
            }
        }
        if !self.temperature_min_c.is_finite()
            || !self.temperature_max_c.is_finite()
            || self.temperature_min_c < 0.0
            || self.temperature_max_c > 255.0
            || self.temperature_min_c >= self.temperature_max_c
        {
            return Err("Temperature: minimum < maximum, between 0 and 255 °C".into());
        }
        if !self.wear_warning_percent.is_finite()
            || !self.wear_critical_percent.is_finite()
            || self.wear_warning_percent < 0.0
            || self.wear_critical_percent > 100.0
            || self.wear_warning_percent >= self.wear_critical_percent
        {
            return Err("Wear: warning < critical, between 0 and 100%".into());
        }
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path).map_err(|e| format!("Reading settings: {e}"))?;
        let value: Self =
            serde_json::from_str(&content).map_err(|e| format!("Settings JSON: {e}"))?;
        value.validate()?;
        Ok(value)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let temp = path.with_extension("json.tmp");
        {
            use io::Write;
            let mut file = fs::File::create(&temp).map_err(|e| e.to_string())?;
            file.write_all(&bytes)
                .and_then(|_| file.sync_all())
                .map_err(|e| e.to_string())?;
        }
        replace_file(&temp, path).map_err(|e| format!("Saving settings: {e}"))
    }
}

#[cfg(windows)]
fn replace_file(from: &Path, to: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        },
        core::PCWSTR,
    };
    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: NUL-terminated buffers remain alive for the duration of the call.
    unsafe {
        MoveFileExW(
            PCWSTR(from.as_ptr()),
            PCWSTR(to.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(io::Error::other)
}

#[cfg(not(windows))]
fn replace_file(from: &Path, to: &Path) -> io::Result<()> {
    fs::rename(from, to)
}

pub fn default_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("F1HaloTelemetry")
        .join("settings.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_settings_start_without_profiles_and_keep_the_existing_layout() {
        let settings: Settings =
            serde_json::from_str(r#"{"halo":{"x":455,"y":537,"scale":1.0369048,"opacity":0.94}}"#)
                .unwrap();
        assert!(settings.halo_profiles.is_empty());
        assert_eq!(settings.active_halo_profile, None);
        assert_eq!(
            (settings.halo.x, settings.halo.y, settings.halo.scale),
            (455, 537, 1.0369048)
        );
        settings.validate().unwrap();
    }

    #[test]
    fn named_profiles_restore_only_position_and_scale_without_auto_overwriting() {
        let mut settings = Settings::default();
        settings.halo.scale = 1.0369048;
        settings.save_halo_profile(" Ferrari ").unwrap();
        let ferrari = settings.halo_profiles["Ferrari"];
        settings.halo.x = 80;
        settings.halo.y = 125;
        settings.halo.scale = 0.8123456;
        settings.save_halo_profile("Red Bull").unwrap();
        let red_bull = settings.halo_profiles["Red Bull"];
        settings.halo.x = 900;
        assert_eq!(settings.halo_profiles["Ferrari"], ferrari);
        assert_eq!(settings.halo_profiles["Red Bull"], red_bull);
        settings.halo.visible = false;
        settings.halo.opacity = 0.37;
        settings.halo_model = HaloModel::Classic;
        let tyres = serde_json::to_value(settings.tyres).unwrap();
        settings.load_halo_profile("ferrari").unwrap();
        assert_eq!(
            (settings.halo.x, settings.halo.y, settings.halo.scale),
            (ferrari.x, ferrari.y, ferrari.scale)
        );
        assert!(!settings.halo.visible);
        assert_eq!(settings.halo.opacity, 0.37);
        assert_eq!(settings.halo_model, HaloModel::Classic);
        assert_eq!(serde_json::to_value(settings.tyres).unwrap(), tyres);
        assert!(!settings.effective_layouts(false, false)[0].visible);
        assert!(!settings.effective_layouts(true, true)[0].visible);
        assert_eq!(settings.active_halo_profile.as_deref(), Some("Ferrari"));
        settings.load_halo_profile("red bull").unwrap();
        assert_eq!(settings.halo.scale, red_bull.scale);
    }

    #[test]
    fn saving_an_existing_name_updates_only_that_profile_and_survives_restart() {
        let mut settings = Settings::default();
        settings.save_halo_profile("Ferrari").unwrap();
        settings.save_halo_profile("Red Bull").unwrap();
        let red_bull = settings.halo_profiles["Red Bull"];
        settings.halo.scale = 0.8765432;
        settings.halo.x = -120;
        assert_eq!(settings.save_halo_profile(" FERRARI ").unwrap(), "Ferrari");
        assert_eq!(settings.halo_profiles.len(), 2);
        assert_eq!(settings.halo_profiles["Red Bull"], red_bull);
        let path =
            std::env::temp_dir().join(format!("f1-halo-profiles-{}.json", std::process::id()));
        settings.save(&path).unwrap();
        let mut restored = Settings::load(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(restored.active_halo_profile.as_deref(), Some("Ferrari"));
        assert_eq!(restored.halo_profiles, settings.halo_profiles);
        restored.load_halo_profile("Ferrari").unwrap();
        assert_eq!((restored.halo.x, restored.halo.scale), (-120, 0.8765432));
    }

    #[test]
    fn invalid_profile_names_scales_and_missing_profiles_leave_saved_profiles_intact() {
        let mut settings = Settings::default();
        settings.save_halo_profile("Ferrari").unwrap();
        let before = serde_json::to_value(&settings).unwrap();
        for name in ["", "   ", "Ferrari\nMonza", &"x".repeat(65)] {
            assert!(settings.save_halo_profile(name).is_err());
            assert_eq!(serde_json::to_value(&settings).unwrap(), before);
        }
        assert!(settings.load_halo_profile("Missing").is_err());
        assert_eq!(serde_json::to_value(&settings).unwrap(), before);
        for scale in [0.0, 3.01, f32::NAN, f32::INFINITY] {
            settings.halo.scale = scale;
            let profiles = settings.halo_profiles.clone();
            assert!(settings.save_halo_profile("Ferrari").is_err());
            assert_eq!(settings.halo_profiles, profiles);
            settings.halo_profiles.get_mut("Ferrari").unwrap().scale = scale;
            assert!(settings.validate().is_err());
            assert!(settings.load_halo_profile("Ferrari").is_err());
            settings.halo_profiles = profiles;
        }
    }

    #[test]
    fn rear_view_only_masks_halo_and_keeps_manual_preferences_for_release() {
        for global in [false, true] {
            for halo in [false, true] {
                for tyres in [false, true] {
                    let settings = Settings {
                        halo: Layout {
                            visible: halo,
                            ..Layout::default()
                        },
                        tyres: Layout {
                            visible: tyres,
                            ..Layout::default()
                        },
                        ..Settings::default()
                    };
                    let before = serde_json::to_value(&settings).unwrap();
                    let behind = settings.effective_layouts(global, true);
                    let front = settings.effective_layouts(global, false);
                    assert!(!behind[0].visible);
                    assert_eq!(behind[1].visible, global && tyres);
                    assert_eq!(front[0].visible, global && halo);
                    assert_eq!(front[1].visible, global && tyres);
                    assert_eq!(serde_json::to_value(&settings).unwrap(), before);
                    assert_eq!(
                        (behind[0].x, behind[0].y, behind[0].scale, behind[0].opacity),
                        (
                            settings.halo.x,
                            settings.halo.y,
                            settings.halo.scale,
                            settings.halo.opacity
                        )
                    );
                }
            }
        }
    }

    #[test]
    fn rear_view_defaults_enabled_for_existing_profiles_and_saves_opt_out() {
        let mut settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(settings.hide_halo_rear_view);
        settings.hide_halo_rear_view = false;
        let restored: Settings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert!(!restored.hide_halo_rear_view);
    }

    #[test]
    fn halo_models_default_for_legacy_profiles_and_round_trip_without_moving_layouts() {
        let legacy = r#"{"halo":{"x":440,"y":450,"scale":1.04,"opacity":0.94}}"#;
        let mut settings: Settings = serde_json::from_str(legacy).unwrap();
        assert_eq!(settings.halo_model, HaloModel::Wide);
        let original_layout = serde_json::to_value(settings.halo).unwrap();
        for model in HaloModel::ALL {
            settings.halo_model = model;
            let restored: Settings =
                serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
            assert_eq!(restored.halo_model, model);
            assert_eq!(
                serde_json::to_value(restored.halo).unwrap(),
                original_layout
            );
        }
        assert!(serde_json::from_str::<Settings>(r#"{"halo_model":"Unknown"}"#).is_err());
    }

    #[test]
    fn legacy_settings_gain_visibility_hotkey_and_save_it() {
        let legacy = r#"{"hotkey":"Alt+F12","halo":{"visible":false},"tyres":{"visible":true},"ers_capacity_joules":8500000}"#;
        let mut settings: Settings = serde_json::from_str(legacy).unwrap();
        assert_eq!(settings.hotkey, "Alt+F12");
        assert_eq!(settings.visibility_hotkey, "Ctrl+Shift+O");
        settings.visibility_hotkey = "Alt+Shift+O".into();
        let restored: Settings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(restored.visibility_hotkey, "Alt+Shift+O");
        assert!(!restored.halo.visible);
        assert!(restored.tyres.visible);
        assert!(
            serde_json::to_value(&restored)
                .unwrap()
                .get("ers_capacity_joules")
                .is_none()
        );
    }

    #[test]
    fn global_visibility_preserves_individual_preferences_and_layouts() {
        for (halo_visible, tyres_visible) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            let settings = Settings {
                halo: Layout {
                    x: 123,
                    y: 456,
                    scale: 0.7,
                    opacity: 0.8,
                    visible: halo_visible,
                },
                tyres: Layout {
                    x: 654,
                    y: 321,
                    scale: 1.2,
                    opacity: 0.6,
                    visible: tyres_visible,
                },
                ..Settings::default()
            };
            let before = serde_json::to_value(&settings).unwrap();
            for globally_visible in [false, true, false, true] {
                for layout in [settings.halo, settings.tyres] {
                    let effective = layout.with_global_visibility(globally_visible);
                    assert_eq!(effective.visible, layout.visible && globally_visible);
                    assert_eq!(
                        (effective.x, effective.y, effective.scale, effective.opacity),
                        (layout.x, layout.y, layout.scale, layout.opacity)
                    );
                }
                assert_eq!(serde_json::to_value(&settings).unwrap(), before);
            }
        }
    }
}
