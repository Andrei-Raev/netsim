use serde::{Deserialize, Serialize};

/// Конфигурация окна визуализации.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width_px: u32,
    pub height_px: u32,
}
