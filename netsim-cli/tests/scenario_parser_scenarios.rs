use std::path::PathBuf;

use netsim_cli::scenario_config::load_scenario;

#[test]
fn scenario_parser_rejects_multiple_scene_modes() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("invalid_multi_scene.scenario.toml");

    let result = load_scenario(fixture.to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn scenario_parser_accepts_traffic_area() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("valid_traffic_area.scenario.toml");

    let result = load_scenario(fixture.to_str().unwrap());
    assert!(result.is_ok());
}
