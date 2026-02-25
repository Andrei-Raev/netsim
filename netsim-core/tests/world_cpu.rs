use netsim_core::{
    ActiveWindow, CpuWorldGenerator, FieldShape, FieldSource, InfluenceType, TimeProfile, Vec2,
    WorldBase, WorldConfig, WorldFieldType, WorldGridGenerator,
};

#[test]
fn cpu_world_generator_is_deterministic() {
    let config = WorldConfig {
        width: 2,
        height: 2,
        cell_size: 1.0,
        base: WorldBase {
            load: 0.1,
            noise: 0.2,
            bandwidth: 1.0,
            cost: 0.5,
        },
    };

    let sources = vec![FieldSource {
        id: 1,
        field_type: WorldFieldType::Noise,
        shape: FieldShape::Circle {
            center: Vec2::new(0.5, 0.5),
            radius: 2.0,
        },
        influence: InfluenceType::Gaussian,
        strength: 1.2,
        time_profile: TimeProfile::Static,
        active_window: ActiveWindow { start: 0, end: 10 },
    }];

    let generator = CpuWorldGenerator::new(config, sources, 42);
    let first = generator.build_grid(3);
    let second = generator.build_grid(3);

    assert_eq!(first, second);
}

#[test]
fn cpu_world_generator_differs_for_different_seed() {
    let config = WorldConfig {
        width: 2,
        height: 2,
        cell_size: 1.0,
        base: WorldBase {
            load: 0.0,
            noise: 0.0,
            bandwidth: 0.0,
            cost: 0.0,
        },
    };

    let sources = vec![FieldSource {
        id: 7,
        field_type: WorldFieldType::Load,
        shape: FieldShape::Circle {
            center: Vec2::new(0.5, 0.5),
            radius: 1.0,
        },
        influence: InfluenceType::Custom { scale: 1.0 },
        strength: 1.0,
        time_profile: TimeProfile::Static,
        active_window: ActiveWindow { start: 0, end: 10 },
    }];

    let generator_a = CpuWorldGenerator::new(config, sources.clone(), 10);
    let generator_b = CpuWorldGenerator::new(config, sources, 11);

    let grid_a = generator_a.build_grid(1);
    let grid_b = generator_b.build_grid(1);

    assert_ne!(grid_a, grid_b);
}

#[test]
fn cpu_world_generator_uses_base_values() {
    let config = WorldConfig {
        width: 1,
        height: 1,
        cell_size: 1.0,
        base: WorldBase {
            load: 0.2,
            noise: 0.3,
            bandwidth: 0.4,
            cost: 0.5,
        },
    };

    let generator = CpuWorldGenerator::new(config, Vec::new(), 0);
    let grid = generator.build_grid(0);

    let cell = &grid.cells[0];
    assert!((cell.load - 0.2).abs() < f32::EPSILON);
    assert!((cell.noise - 0.3).abs() < f32::EPSILON);
    assert!((cell.bandwidth - 0.4).abs() < f32::EPSILON);
    assert!((cell.cost - 0.5).abs() < f32::EPSILON);
}

#[test]
fn cpu_world_generator_respects_active_window() {
    let config = WorldConfig {
        width: 1,
        height: 1,
        cell_size: 1.0,
        base: WorldBase {
            load: 0.0,
            noise: 0.0,
            bandwidth: 0.0,
            cost: 0.0,
        },
    };

    let sources = vec![FieldSource {
        id: 1,
        field_type: WorldFieldType::Load,
        shape: FieldShape::Circle {
            center: Vec2::new(0.5, 0.5),
            radius: 1.0,
        },
        influence: InfluenceType::Hard,
        strength: 1.0,
        time_profile: TimeProfile::Static,
        active_window: ActiveWindow { start: 2, end: 2 },
    }];

    let generator = CpuWorldGenerator::new(config, sources, 0);

    let inactive = generator.build_grid(1);
    let active = generator.build_grid(2);

    assert_eq!(inactive.cells[0].load, 0.0);
    assert!(active.cells[0].load > 0.0);
}

#[test]
fn cpu_world_generator_handles_zero_dimensions() {
    let config = WorldConfig {
        width: 0,
        height: 0,
        cell_size: 1.0,
        base: WorldBase {
            load: 0.0,
            noise: 0.0,
            bandwidth: 0.0,
            cost: 0.0,
        },
    };

    let generator = CpuWorldGenerator::new(config, Vec::new(), 0);
    let grid = generator.build_grid(0);

    assert!(grid.cells.is_empty());
}
