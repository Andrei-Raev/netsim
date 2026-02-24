use super::{
    ActiveWindow, FieldShape, FieldSource, InfluenceType, TimeProfile, Vec2, WorldBase,
    WorldConfig, WorldFieldType,
};

/// Описание базовой сцены мира (набор источников + конфиг сетки).
#[derive(Debug, Clone, PartialEq)]
pub struct WorldScene {
    pub config: WorldConfig,
    pub sources: Vec<FieldSource>,
    pub seed: u64,
}

impl WorldScene {
    /// Создаёт сцену с фиксированным набором источников.
    pub fn new(config: WorldConfig, sources: Vec<FieldSource>, seed: u64) -> Self {
        Self {
            config,
            sources,
            seed,
        }
    }
}

/// Готовая сцена «минимальный мир».
///
/// Цель: обеспечить рабочий базовый набор источников для визуализации и тестов.
pub fn minimal_scene(width: usize, height: usize, cell_size: f32, seed: u64) -> WorldScene {
    let config = WorldConfig {
        width,
        height,
        cell_size,
        base: WorldBase {
            load: 0.1,
            noise: 0.05,
            bandwidth: 1.0,
            cost: 0.2,
        },
    };

    let sources = vec![
        FieldSource {
            id: 1,
            field_type: WorldFieldType::Noise,
            shape: FieldShape::Circle {
                center: Vec2::new(cell_size * 2.0, cell_size * 2.0),
                radius: cell_size * 3.0,
            },
            influence: InfluenceType::Gaussian,
            strength: 1.5,
            time_profile: TimeProfile::Static,
            active_window: ActiveWindow {
                start: 0,
                end: 10_000,
            },
        },
        FieldSource {
            id: 2,
            field_type: WorldFieldType::Bandwidth,
            shape: FieldShape::Line {
                from: Vec2::new(0.0, cell_size * 2.0),
                to: Vec2::new(cell_size * width as f32, cell_size * 2.0),
                width: cell_size * 1.5,
            },
            influence: InfluenceType::Linear,
            strength: 2.0,
            time_profile: TimeProfile::Static,
            active_window: ActiveWindow {
                start: 0,
                end: 10_000,
            },
        },
        FieldSource {
            id: 3,
            field_type: WorldFieldType::Cost,
            shape: FieldShape::Rect {
                center: Vec2::new(cell_size * 4.0, cell_size * 4.0),
                half_extents: Vec2::new(cell_size * 1.5, cell_size * 1.5),
            },
            influence: InfluenceType::Hard,
            strength: 1.0,
            time_profile: TimeProfile::Static,
            active_window: ActiveWindow {
                start: 0,
                end: 10_000,
            },
        },
        FieldSource {
            id: 4,
            field_type: WorldFieldType::Load,
            shape: FieldShape::Circle {
                center: Vec2::new(cell_size * 6.0, cell_size * 1.5),
                radius: cell_size * 2.0,
            },
            influence: InfluenceType::Linear,
            strength: 0.8,
            time_profile: TimeProfile::Wave {
                period_ticks: 50,
                amplitude: 0.3,
                phase: 0.0,
            },
            active_window: ActiveWindow {
                start: 0,
                end: 10_000,
            },
        },
    ];

    WorldScene::new(config, sources, seed)
}
