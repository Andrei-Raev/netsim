use netsim_core::world::cpu::CpuWorldGenerator;
use netsim_core::{WorldFieldType, WorldGridGenerator, minimal_scene};

#[test]
fn minimal_scene_generates_non_empty_grid() {
    let scene = minimal_scene(8, 6, 1.0, 42);
    let generator = CpuWorldGenerator::new(scene.config, scene.sources, scene.seed);

    let grid = generator.build_grid(1);
    assert_eq!(grid.width, 8);
    assert_eq!(grid.height, 6);
    assert_eq!(grid.cells.len(), 48);
}

#[test]
fn minimal_scene_applies_noise_source() {
    let scene = minimal_scene(6, 6, 1.0, 7);
    let generator = CpuWorldGenerator::new(scene.config, scene.sources, scene.seed);

    let grid = generator.build_grid(0);
    let mut has_noise = false;
    for cell in grid.cells {
        if cell.noise > scene.config.base.noise {
            has_noise = true;
            break;
        }
    }

    assert!(has_noise);
}

#[test]
fn minimal_scene_contains_expected_field_types() {
    let scene = minimal_scene(4, 4, 1.0, 1);
    let mut has_load = false;
    let mut has_noise = false;
    let mut has_bandwidth = false;
    let mut has_cost = false;

    for source in &scene.sources {
        match source.field_type {
            WorldFieldType::Load => has_load = true,
            WorldFieldType::Noise => has_noise = true,
            WorldFieldType::Bandwidth => has_bandwidth = true,
            WorldFieldType::Cost => has_cost = true,
        }
    }

    assert!(has_load && has_noise && has_bandwidth && has_cost);
}
