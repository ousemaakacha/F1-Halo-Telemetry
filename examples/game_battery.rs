//! Read-only inspection of installed nominal battery capacities; no game assets are exported.
use f1_halo_telemetry::{game_data::BatteryCatalog, telemetry::Snapshot};

fn main() {
    let catalog = std::env::args_os()
        .nth(1)
        .map_or_else(BatteryCatalog::discover, |path| {
            BatteryCatalog::from_game_root(std::path::Path::new(&path))
        });
    let years: Vec<_> = [(2025, 0), (2026, 13)].into_iter().map(|(year, formula)| {
        let snapshot = Snapshot { formula: Some(formula), ..Default::default() };
        serde_json::json!({"year": year, "nominal_joules": catalog.resolve(&snapshot).map(|c| c.joules)})
    }).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "game_root": catalog.game_root, "capacities": years,
            "records": catalog.records.iter().map(|r| serde_json::json!({
                "archive": r.archive, "resource": r.resource, "joules": r.joules
            })).collect::<Vec<_>>(), "errors": catalog.errors
        }))
        .unwrap()
    );
}
