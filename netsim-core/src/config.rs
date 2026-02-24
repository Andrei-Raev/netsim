// Тут хранятся конфиги ядра симулятора (без механизма загрузки).

/// Базовая конфигурация симуляции для ядра.
#[derive(Debug, Clone, Copy)]
pub struct SimConfig {
    /// Количество агентов.
    pub agents_count: u32,
    /// Количество тиков для прогона.
    pub ticks: u64,
    /// Размер окна ring‑buffer для очереди событий.
    pub event_queue_window: u64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            agents_count: 0,
            ticks: 0,
            event_queue_window: 64,
        }
    }
}
