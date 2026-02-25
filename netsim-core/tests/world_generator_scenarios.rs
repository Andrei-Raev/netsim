use netsim_core::world::cpu::CpuWorldGenerator;
use netsim_core::{
    ActiveWindow, FieldShape, FieldSource, InfluenceType, TimeProfile, Vec2, WorldBase,
    WorldConfig, WorldFieldType, WorldGrid, WorldGridGenerator,
};

#[test]
fn world_generator_is_deterministic_for_same_seed() {
    let config = WorldConfig {
        width: 2,
        height: 1,
        cell_size: 1.0,
        base: WorldBase {
            load: 1.0,
            noise: 0.5,
            bandwidth: 2.0,
            cost: 3.0,
        },
    };

    let source = FieldSource {
        id: 10,
        field_type: WorldFieldType::Load,
        shape: FieldShape::Circle {
            center: Vec2::new(0.5, 0.5),
            radius: 10.0,
        },
        influence: InfluenceType::Linear,
        strength: 2.0,
        time_profile: TimeProfile::Static,
        active_window: ActiveWindow { start: 0, end: 10 },
    };

    let generator_a = CpuWorldGenerator::new(config, vec![source.clone()], 42);
    let generator_b = CpuWorldGenerator::new(config, vec![source], 42);

    let grid_a = generator_a.build_grid(5);
    let grid_b = generator_b.build_grid(5);

    assert_eq!(grid_a.cells, grid_b.cells);
}

#[test]
fn world_generator_respects_active_window() {
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

    let source = FieldSource {
        id: 1,
        field_type: WorldFieldType::Load,
        shape: FieldShape::Circle {
            center: Vec2::new(0.5, 0.5),
            radius: 10.0,
        },
        influence: InfluenceType::Hard,
        strength: 3.0,
        time_profile: TimeProfile::Static,
        active_window: ActiveWindow { start: 5, end: 10 },
    };

    let generator = CpuWorldGenerator::new(config, vec![source], 7);

    let inactive_grid = generator.build_grid(3);
    let active_grid = generator.build_grid(6);

    assert!(inactive_grid.cells[0].load.abs() < f32::EPSILON);
    assert!(active_grid.cells[0].load > 0.0);
}

#[test]
fn world_generator_applies_multiple_sources() {
    let config = WorldConfig {
        width: 1,
        height: 1,
        cell_size: 1.0,
        base: WorldBase {
            load: 1.0,
            noise: 1.0,
            bandwidth: 1.0,
            cost: 1.0,
        },
    };

    let sources = vec![
        FieldSource {
            id: 11,
            field_type: WorldFieldType::Load,
            shape: FieldShape::Circle {
                center: Vec2::new(0.5, 0.5),
                radius: 10.0,
            },
            influence: InfluenceType::Hard,
            strength: 2.0,
            time_profile: TimeProfile::Static,
            active_window: ActiveWindow { start: 0, end: 10 },
        },
        FieldSource {
            id: 12,
            field_type: WorldFieldType::Cost,
            shape: FieldShape::Circle {
                center: Vec2::new(0.5, 0.5),
                radius: 10.0,
            },
            influence: InfluenceType::Hard,
            strength: 3.0,
            time_profile: TimeProfile::Static,
            active_window: ActiveWindow { start: 0, end: 10 },
        },
    ];

    let generator = CpuWorldGenerator::new(config, sources, 1);
    let grid = generator.build_grid(1);

    assert_eq!(grid.cells[0].load, 3.0);
    assert_eq!(grid.cells[0].cost, 4.0);
    assert_eq!(grid.cells[0].noise, 1.0);
    assert_eq!(grid.cells[0].bandwidth, 1.0);
}

#[test]
fn world_generator_handles_zero_sized_grid() {
    let config = WorldConfig {
        width: 0,
        height: 0,
        cell_size: 1.0,
        base: WorldBase {
            load: 1.0,
            noise: 1.0,
            bandwidth: 1.0,
            cost: 1.0,
        },
    };

    let generator = CpuWorldGenerator::new(config, Vec::new(), 0);
    let grid = generator.build_grid(0);

    assert!(grid.cells.is_empty());
    assert_eq!(grid.width, 0);
    assert_eq!(grid.height, 0);
}

#[test]
fn time_profile_curve_interpolates_linearly() {
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

    let source = FieldSource {
        id: 2,
        field_type: WorldFieldType::Load,
        shape: FieldShape::Circle {
            center: Vec2::new(0.5, 0.5),
            radius: 10.0,
        },
        influence: InfluenceType::Hard,
        strength: 10.0,
        time_profile: TimeProfile::Curve {
            points: vec![(0, 0.0), (10, 1.0)],
        },
        active_window: ActiveWindow { start: 0, end: 10 },
    };

    let generator = CpuWorldGenerator::new(config, vec![source], 9);
    let grid = generator.build_grid(5);

    assert!((grid.cells[0].load - 5.0).abs() < 1e-4);
}

#[test]
fn pulse_profile_disables_when_duty_zero() {
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

    let source = FieldSource {
        id: 3,
        field_type: WorldFieldType::Load,
        shape: FieldShape::Circle {
            center: Vec2::new(0.5, 0.5),
            radius: 10.0,
        },
        influence: InfluenceType::Hard,
        strength: 10.0,
        time_profile: TimeProfile::Pulse {
            period_ticks: 10,
            duty: 0.0,
        },
        active_window: ActiveWindow { start: 0, end: 10 },
    };

    let generator = CpuWorldGenerator::new(config, vec![source], 11);
    let grid = generator.build_grid(5);

    assert!(grid.cells[0].load.abs() < f32::EPSILON);
}

#[test]
fn world_grid_cell_out_of_bounds_returns_none() {
    let mut grid = WorldGrid {
        width: 1,
        height: 1,
        cell_size: 1.0,
        cells: vec![netsim_core::WorldCell {
            load: 0.0,
            noise: 0.0,
            bandwidth: 0.0,
            cost: 0.0,
        }],
    };

    assert!(grid.cell(2, 0).is_none());
    assert!(grid.cell_mut(0, 2).is_none());
}
