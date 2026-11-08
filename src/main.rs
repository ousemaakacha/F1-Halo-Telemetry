#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    use f1_halo_telemetry::app;
    if let Err(error) = app::Options::parse().and_then(app::run) {
        eprintln!("{error}");
        app::show_error(&error);
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("F1 Halo Telemetry requires Windows 11 x64. Parser and tests are cross-platform.");
}
