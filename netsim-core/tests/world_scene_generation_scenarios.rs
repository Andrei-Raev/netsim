use netsim_core::world::cpu::CpuWorldGenerator;
use netsim_core::{WorldConfig, WorldFieldType, WorldGridGenerator, generate_scene};

#[test]
fn generate_scene_is_deterministic_for_same_seed() {
    let config = WorldConfig {
        width: 6,
        height: 6,
        cell_size: 1.0,
        base: netsim_core::WorldBase {
            load: 0.0,
            noise: 0.0,
            bandwidth: 0.0,
            cost: 0.0,
        },
    };

    let scene_a = generate_scene(config, 42, 6);
    let scene_b = generate_scene(config, 42, 6);

    assert_eq!(scene_a, scene_b);

    let gen_a = CpuWorldGenerator::new(scene_a.config, scene_a.sources.clone(), scene_a.seed);
    let gen_b = CpuWorldGenerator::new(scene_b.config, scene_b.sources.clone(), scene_b.seed);

    let grid_a = gen_a.build_grid(3);
    let grid_b = gen_b.build_grid(3);

    assert_eq!(grid_a.cells, grid_b.cells);
}

#[test]
fn generate_scene_respects_source_count_and_types() {
    let config = WorldConfig {
        width: 4,
        height: 4,
        cell_size: 1.0,
        base: netsim_core::WorldBase {
            load: 0.0,
            noise: 0.0,
            bandwidth: 0.0,
            cost: 0.0,
        },
    };

    let scene = generate_scene(config, 1, 5);
    assert_eq!(scene.sources.len(), 5);

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
