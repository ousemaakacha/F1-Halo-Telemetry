pub const APPLICATION_NAME: &str = "F1 Halo Telemetry";

#[cfg(windows)]
pub mod app;
pub mod diagnostics;
pub mod game_data;
#[cfg(windows)]
pub mod graphics;
#[cfg(windows)]
pub mod hud;
#[cfg(windows)]
pub mod input;
#[cfg(windows)]
pub mod overlay;
pub mod settings;
pub mod telemetry;
pub mod udp;
