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

/// Детерминированный генератор сцен на базе seed.
///
/// Создает заданное число источников, чередуя типы полей и формы.
pub fn generate_scene(config: WorldConfig, seed: u64, source_count: usize) -> WorldScene {
    let mut rng = SceneRng::new(seed);
    let mut sources = Vec::with_capacity(source_count);

    let world_width = config.width as f32 * config.cell_size;
    let world_height = config.height as f32 * config.cell_size;
    let max_radius = world_width.min(world_height) * 0.35;
    let min_radius = config.cell_size.max(0.1);

    for index in 0..source_count {
        let field_type = match index % 4 {
            0 => WorldFieldType::Load,
            1 => WorldFieldType::Noise,
            2 => WorldFieldType::Bandwidth,
            _ => WorldFieldType::Cost,
        };

        let radius = rng.range_f32(min_radius, max_radius.max(min_radius));
        let center = Vec2::new(
            rng.range_f32(0.0, world_width),
            rng.range_f32(0.0, world_height),
        );
        let shape = match index % 3 {
            0 => FieldShape::Circle { center, radius },
            1 => FieldShape::Rect {
                center,
                half_extents: Vec2::new(radius * 0.6, radius * 0.6),
            },
            _ => {
                let from = Vec2::new(
                    rng.range_f32(0.0, world_width),
                    rng.range_f32(0.0, world_height),
                );
                let to = Vec2::new(
                    rng.range_f32(0.0, world_width),
                    rng.range_f32(0.0, world_height),
                );
                FieldShape::Line {
                    from,
                    to,
                    width: radius * 0.5,
                }
            }
        };

        let influence = match index % 3 {
            0 => InfluenceType::Hard,
            1 => InfluenceType::Linear,
            _ => InfluenceType::Gaussian,
        };

        let strength = rng.range_f32(0.4, 2.0);
        let time_profile = match index % 4 {
            0 => TimeProfile::Static,
            1 => TimeProfile::Pulse {
                period_ticks: rng.range_u64(20, 120),
                duty: rng.range_f32(0.1, 0.9),
            },
            2 => TimeProfile::Wave {
                period_ticks: rng.range_u64(30, 160),
                amplitude: rng.range_f32(0.1, 0.6),
                phase: rng.range_f32(0.0, std::f32::consts::TAU),
            },
            _ => TimeProfile::Curve {
                points: vec![
                    (0, rng.range_f32(0.4, 0.9)),
                    (rng.range_u64(20, 80), rng.range_f32(0.8, 1.4)),
                ],
            },
        };

        sources.push(FieldSource {
            id: index as u64 + 1,
            field_type,
            shape,
            influence,
            strength,
            time_profile,
            active_window: ActiveWindow {
                start: 0,
                end: 10_000,
            },
        });
    }

    WorldScene::new(config, sources, seed)
}

#[derive(Debug, Clone)]
struct SceneRng {
    state: u64,
}

impl SceneRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_f32(&mut self) -> f32 {
        let value = (self.next_u64() >> 40) as u32;
        value as f32 / u32::MAX as f32
    }

    fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        if max <= min {
            return min;
        }
        min + (max - min) * self.next_f32()
    }

    fn range_u64(&mut self, min: u64, max: u64) -> u64 {
        if max <= min {
            return min;
        }
        let span = max - min;
        min + (self.next_u64() % span)
    }
}
