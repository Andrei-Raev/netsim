//! Модуль мира (CPU‑референс + типы для будущего GPU‑бэкенда).
//!
//! Цель: детерминированное, stateless‑описание мировых полей (load/noise/bandwidth/cost)
//! с минимальным API, которое потом можно заменить GPU‑реализацией без ломки интерфейса.

pub mod agents_grid;
pub mod cpu;
pub mod scene_generator;
pub mod scenes;

mod math;

use std::fmt;

/// 2D‑вектор в мировом пространстве.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Создаёт вектор из координат.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Тип поля мира, влияющий на сигнал (пакеты).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorldFieldType {
    Load,
    Noise,
    Bandwidth,
    Cost,
}

/// Тип функции влияния источника поля.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InfluenceType {
    /// Жёсткая маска: 1 внутри радиуса, иначе 0.
    Hard,
    /// Линейный спад до нуля к границе.
    Linear,
    /// Гауссово распределение вокруг центра.
    Gaussian,
    /// Пользовательская детерминированная функция (через seed).
    Custom { scale: f32 },
}

/// Геометрия источника поля.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldShape {
    Circle { center: Vec2, radius: f32 },
    Rect { center: Vec2, half_extents: Vec2 },
    Line { from: Vec2, to: Vec2, width: f32 },
    Spline { points: Vec<Vec2>, width: f32 },
}

/// Временной профиль источника.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeProfile {
    /// Постоянное влияние.
    Static,
    /// Периодический импульс (duty 0..1).
    Pulse { period_ticks: u64, duty: f32 },
    /// Волна с синусоидальным отклонением.
    Wave {
        period_ticks: u64,
        amplitude: f32,
        phase: f32,
    },
    /// Кусочно‑линейная кривая.
    Curve { points: Vec<(u64, f32)> },
}

/// Окно активности источника (включительно).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActiveWindow {
    pub start: u64,
    pub end: u64,
}

impl ActiveWindow {
    /// Проверяет, активен ли источник на данном тике.
    pub fn is_active(&self, tick: u64) -> bool {
        tick >= self.start && tick <= self.end
    }
}

/// Источник поля мира (stateless).
#[derive(Debug, Clone, PartialEq)]
pub struct FieldSource {
    pub id: u64,
    pub field_type: WorldFieldType,
    pub shape: FieldShape,
    pub influence: InfluenceType,
    pub strength: f32,
    pub time_profile: TimeProfile,
    pub active_window: ActiveWindow,
}

impl FieldSource {
    /// Проверяет, активен ли источник на данном тике.
    pub fn is_active(&self, tick: u64) -> bool {
        self.active_window.is_active(tick)
    }

    /// Возвращает множитель времени для источника.
    pub fn time_multiplier(&self, tick: u64) -> f32 {
        match &self.time_profile {
            TimeProfile::Static => 1.0,
            TimeProfile::Pulse { period_ticks, duty } => {
                if *period_ticks == 0 {
                    return 0.0;
                }
                let duty = duty.clamp(0.0, 1.0);
                let phase = tick % period_ticks;
                let active_ticks = (*period_ticks as f32 * duty).round() as u64;
                if phase < active_ticks { 1.0 } else { 0.0 }
            }
            TimeProfile::Wave {
                period_ticks,
                amplitude,
                phase,
            } => {
                if *period_ticks == 0 {
                    return 0.0;
                }
                let t = tick as f32 / *period_ticks as f32;
                let angle = std::f32::consts::TAU * t + *phase;
                1.0 + amplitude * angle.sin()
            }
            TimeProfile::Curve { points } => curve_value(points, tick),
        }
    }
}

/// Базовые (фоновые) значения поля мира.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldBase {
    pub load: f32,
    pub noise: f32,
    pub bandwidth: f32,
    pub cost: f32,
}

/// Конфигурация сетки мира.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldConfig {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub base: WorldBase,
}

/// Значения ячейки мира.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldCell {
    pub load: f32,
    pub noise: f32,
    pub bandwidth: f32,
    pub cost: f32,
}

/// Результат генерации мира на текущем тике.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldGrid {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub cells: Vec<WorldCell>,
}

/// Интерфейс генератора сетки мира (CPU/GPU‑бэкенды).
pub trait WorldGridGenerator {
    /// Строит сетку мира на заданном тике.
    fn build_grid(&self, tick: u64) -> WorldGrid;
}

impl WorldGrid {
    /// Возвращает ссылку на ячейку (или None, если индекс вне границ).
    pub fn cell(&self, x: usize, y: usize) -> Option<&WorldCell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y * self.width + x;
        self.cells.get(index)
    }

    /// Возвращает mutable‑ссылку на ячейку (или None, если индекс вне границ).
    pub fn cell_mut(&mut self, x: usize, y: usize) -> Option<&mut WorldCell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y * self.width + x;
        self.cells.get_mut(index)
    }

    /// Преобразует мировые координаты в индексы ячейки.
    pub fn world_to_cell(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        if self.cell_size <= 0.0 {
            return None;
        }
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let cell_x = (x / self.cell_size).floor() as i64;
        let cell_y = (y / self.cell_size).floor() as i64;
        if cell_x < 0 || cell_y < 0 {
            return None;
        }
        let cell_x = cell_x as usize;
        let cell_y = cell_y as usize;
        if cell_x >= self.width || cell_y >= self.height {
            return None;
        }
        Some((cell_x, cell_y))
    }

    /// Сэмплирует ячейку по мировым координатам.
    pub fn sample(&self, x: f32, y: f32) -> Option<&WorldCell> {
        let (cell_x, cell_y) = self.world_to_cell(x, y)?;
        self.cell(cell_x, cell_y)
    }
}

fn curve_value(points: &[(u64, f32)], tick: u64) -> f32 {
    if points.is_empty() {
        return 1.0;
    }
    let mut prev = points[0];
    if tick <= prev.0 {
        return prev.1;
    }
    for &point in points.iter().skip(1) {
        if tick <= point.0 {
            let span = (point.0 - prev.0) as f32;
            if span == 0.0 {
                return point.1;
            }
            let ratio = (tick - prev.0) as f32 / span;
            return prev.1 + (point.1 - prev.1) * ratio;
        }
        prev = point;
    }
    prev.1
}

fn influence_weight(
    influence: InfluenceType,
    shape: &FieldShape,
    point: Vec2,
    seed: u64,
    source_id: u64,
    tick: u64,
) -> f32 {
    match influence {
        InfluenceType::Custom { scale } => {
            let value = math::deterministic_unit(seed, source_id, tick, point.x, point.y);
            value * scale
        }
        InfluenceType::Hard => {
            let radius = math::shape_radius(shape);
            if radius <= 0.0 {
                return 0.0;
            }
            let dist = math::shape_distance(shape, point);
            if dist <= radius { 1.0 } else { 0.0 }
        }
        InfluenceType::Linear => {
            let radius = math::shape_radius(shape);
            if radius <= 0.0 {
                return 0.0;
            }
            let dist = math::shape_distance(shape, point);
            (1.0 - dist / radius).max(0.0)
        }
        InfluenceType::Gaussian => {
            let radius = math::shape_radius(shape);
            if radius <= 0.0 {
                return 0.0;
            }
            let sigma = radius / 2.0;
            if sigma <= 0.0 {
                return 0.0;
            }
            let dist = math::shape_distance(shape, point);
            let exponent = -(dist * dist) / (2.0 * sigma * sigma);
            exponent.exp()
        }
    }
}

impl fmt::Display for WorldFieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            WorldFieldType::Load => "load",
            WorldFieldType::Noise => "noise",
            WorldFieldType::Bandwidth => "bandwidth",
            WorldFieldType::Cost => "cost",
        };
        write!(f, "{name}")
    }
}

/// Применяет один источник к ячейке (внутренний шаг генератора).
pub(crate) fn apply_source(
    cell: &mut WorldCell,
    source: &FieldSource,
    point: Vec2,
    tick: u64,
    seed: u64,
) {
    let weight = influence_weight(
        source.influence,
        &source.shape,
        point,
        seed,
        source.id,
        tick,
    );
    let value = source.strength * source.time_multiplier(tick) * weight;
    match source.field_type {
        WorldFieldType::Load => cell.load += value,
        WorldFieldType::Noise => cell.noise += value,
        WorldFieldType::Bandwidth => cell.bandwidth += value,
        WorldFieldType::Cost => cell.cost += value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_cell_handles_bounds() {
        let grid = WorldGrid {
            width: 2,
            height: 2,
            cell_size: 1.0,
            cells: vec![
                WorldCell {
                    load: 0.0,
                    noise: 0.0,
                    bandwidth: 0.0,
                    cost: 0.0,
                };
                4
            ],
        };

        assert_eq!(grid.world_to_cell(0.1, 0.1), Some((0, 0)));
        assert_eq!(grid.world_to_cell(1.9, 1.9), Some((1, 1)));
        assert_eq!(grid.world_to_cell(-0.1, 0.0), None);
        assert_eq!(grid.world_to_cell(2.0, 0.0), None);
        assert_eq!(grid.world_to_cell(0.0, 2.0), None);
    }

    #[test]
    fn curve_value_handles_edges() {
        let points = vec![(5, 2.0), (10, 4.0), (15, 1.0)];

        assert!((curve_value(&points, 0) - 2.0).abs() < f32::EPSILON);
        assert!((curve_value(&points, 5) - 2.0).abs() < f32::EPSILON);
        assert!((curve_value(&points, 10) - 4.0).abs() < f32::EPSILON);
        assert!((curve_value(&points, 12) - 2.8).abs() < 1e-6);
        assert!((curve_value(&points, 20) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn influence_weight_custom_is_deterministic() {
        let shape = FieldShape::Circle {
            center: Vec2::new(0.0, 0.0),
            radius: 1.0,
        };

        let first = influence_weight(
            InfluenceType::Custom { scale: 1.0 },
            &shape,
            Vec2::new(0.5, 0.5),
            42,
            7,
            10,
        );
        let second = influence_weight(
            InfluenceType::Custom { scale: 1.0 },
            &shape,
            Vec2::new(0.5, 0.5),
            42,
            7,
            10,
        );

        assert!((first - second).abs() < f32::EPSILON);
    }
}
