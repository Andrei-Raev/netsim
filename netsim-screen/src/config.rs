use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct WindowConfig {
    pub width_px: u32,
    pub height_px: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width_px: 1280,
            height_px: 720,
        }
    }
}
