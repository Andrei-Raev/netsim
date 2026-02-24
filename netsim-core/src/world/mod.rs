pub mod cpu;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorldFieldType {
    Load,
    Noise,
    Bandwidth,
    Cost,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InfluenceType {
    Hard,
    Linear,
    Gaussian,
    Custom { scale: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldShape {
    Circle { center: Vec2, radius: f32 },
    Rect { center: Vec2, half_extents: Vec2 },
    Line { from: Vec2, to: Vec2, width: f32 },
    Spline { points: Vec<Vec2>, width: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeProfile {
    Static,
    Pulse {
        period_ticks: u64,
        duty: f32,
    },
    Wave {
        period_ticks: u64,
        amplitude: f32,
        phase: f32,
    },
    Curve {
        points: Vec<(u64, f32)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActiveWindow {
    pub start: u64,
    pub end: u64,
}

impl ActiveWindow {
    pub fn is_active(&self, tick: u64) -> bool {
        tick >= self.start && tick <= self.end
    }
}

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
    pub fn is_active(&self, tick: u64) -> bool {
        self.active_window.is_active(tick)
    }

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldBase {
    pub load: f32,
    pub noise: f32,
    pub bandwidth: f32,
    pub cost: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldConfig {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub base: WorldBase,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldCell {
    pub load: f32,
    pub noise: f32,
    pub bandwidth: f32,
    pub cost: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldGrid {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub cells: Vec<WorldCell>,
}

impl WorldGrid {
    pub fn cell(&self, x: usize, y: usize) -> Option<&WorldCell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y * self.width + x;
        self.cells.get(index)
    }

    pub fn cell_mut(&mut self, x: usize, y: usize) -> Option<&mut WorldCell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y * self.width + x;
        self.cells.get_mut(index)
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

fn dist_to_segment(point: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = Vec2::new(b.x - a.x, b.y - a.y);
    let ap = Vec2::new(point.x - a.x, point.y - a.y);
    let ab_len_sq = ab.x * ab.x + ab.y * ab.y;
    if ab_len_sq == 0.0 {
        return ((point.x - a.x).powi(2) + (point.y - a.y).powi(2)).sqrt();
    }
    let t = (ap.x * ab.x + ap.y * ab.y) / ab_len_sq;
    let t = t.clamp(0.0, 1.0);
    let proj = Vec2::new(a.x + ab.x * t, a.y + ab.y * t);
    ((point.x - proj.x).powi(2) + (point.y - proj.y).powi(2)).sqrt()
}

fn shape_radius(shape: &FieldShape) -> f32 {
    match shape {
        FieldShape::Circle { radius, .. } => *radius,
        FieldShape::Rect { half_extents, .. } => half_extents.x.max(half_extents.y),
        FieldShape::Line { width, .. } => *width,
        FieldShape::Spline { width, .. } => *width,
    }
}

fn shape_distance(shape: &FieldShape, point: Vec2) -> f32 {
    match shape {
        FieldShape::Circle { center, radius: _ } => {
            ((point.x - center.x).powi(2) + (point.y - center.y).powi(2)).sqrt()
        }
        FieldShape::Rect {
            center,
            half_extents,
        } => {
            let dx = (point.x - center.x).abs() - half_extents.x;
            let dy = (point.y - center.y).abs() - half_extents.y;
            let cx = dx.max(0.0);
            let cy = dy.max(0.0);
            (cx * cx + cy * cy).sqrt()
        }
        FieldShape::Line { from, to, .. } => dist_to_segment(point, *from, *to),
        FieldShape::Spline { points, .. } => {
            if points.len() < 2 {
                return 0.0;
            }
            let mut best = f32::MAX;
            for window in points.windows(2) {
                if let [a, b] = window {
                    let d = dist_to_segment(point, *a, *b);
                    if d < best {
                        best = d;
                    }
                }
            }
            best
        }
    }
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
            let value = deterministic_unit(seed, source_id, tick, point.x, point.y);
            value * scale
        }
        InfluenceType::Hard => {
            let radius = shape_radius(shape);
            if radius <= 0.0 {
                return 0.0;
            }
            let dist = shape_distance(shape, point);
            if dist <= radius { 1.0 } else { 0.0 }
        }
        InfluenceType::Linear => {
            let radius = shape_radius(shape);
            if radius <= 0.0 {
                return 0.0;
            }
            let dist = shape_distance(shape, point);
            (1.0 - dist / radius).max(0.0)
        }
        InfluenceType::Gaussian => {
            let radius = shape_radius(shape);
            if radius <= 0.0 {
                return 0.0;
            }
            let sigma = radius / 2.0;
            if sigma <= 0.0 {
                return 0.0;
            }
            let dist = shape_distance(shape, point);
            let exponent = -(dist * dist) / (2.0 * sigma * sigma);
            exponent.exp()
        }
    }
}

fn deterministic_unit(seed: u64, id: u64, tick: u64, x: f32, y: f32) -> f32 {
    let mut state = seed ^ id.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    state ^= tick.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state ^= (x.to_bits() as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^= (y.to_bits() as u64).wrapping_mul(0xD6E8_FF3E_1AFB_38D9);
    let hashed = mix_u64(state);
    let upper = (hashed >> 40) as u32;
    upper as f32 / u32::MAX as f32
}

fn mix_u64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    value ^= value >> 33;
    value = value.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    value ^= value >> 33;
    value
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
