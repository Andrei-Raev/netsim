use super::{FieldShape, Vec2};

pub fn dist_to_segment(point: Vec2, a: Vec2, b: Vec2) -> f32 {
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

pub fn shape_radius(shape: &FieldShape) -> f32 {
    match shape {
        FieldShape::Circle { radius, .. } => *radius,
        FieldShape::Rect { half_extents, .. } => half_extents.x.max(half_extents.y),
        FieldShape::Line { width, .. } => *width,
        FieldShape::Spline { width, .. } => *width,
    }
}

pub fn shape_distance(shape: &FieldShape, point: Vec2) -> f32 {
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

pub fn deterministic_unit(seed: u64, id: u64, tick: u64, x: f32, y: f32) -> f32 {
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
