use super::{FieldSource, Vec2, WorldCell, WorldConfig, WorldGrid, apply_source};

/// CPU‑референс генератора мира.
///
/// Stateless: каждый тик строит сетку заново из конфигурации и источников.
#[derive(Debug, Clone, PartialEq)]
pub struct CpuWorldGenerator {
    pub config: WorldConfig,
    pub sources: Vec<FieldSource>,
    pub seed: u64,
}

impl CpuWorldGenerator {
    /// Создаёт генератор мира на CPU.
    pub fn new(config: WorldConfig, sources: Vec<FieldSource>, seed: u64) -> Self {
        Self {
            config,
            sources,
            seed,
        }
    }
}

impl super::WorldGridGenerator for CpuWorldGenerator {
    fn build_grid(&self, tick: u64) -> WorldGrid {
        let mut grid = WorldGrid {
            width: self.config.width,
            height: self.config.height,
            cell_size: self.config.cell_size,
            cells: vec![
                WorldCell {
                    load: self.config.base.load,
                    noise: self.config.base.noise,
                    bandwidth: self.config.base.bandwidth,
                    cost: self.config.base.cost,
                };
                self.config.width * self.config.height
            ],
        };

        if self.config.width == 0 || self.config.height == 0 {
            return grid;
        }

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let center = Vec2::new(
                    (x as f32 + 0.5) * self.config.cell_size,
                    (y as f32 + 0.5) * self.config.cell_size,
                );
                if let Some(cell) = grid.cell_mut(x, y) {
                    for source in &self.sources {
                        if !source.is_active(tick) {
                            continue;
                        }
                        apply_source(cell, source, center, tick, self.seed);
                    }
                }
            }
        }

        grid
    }
}
