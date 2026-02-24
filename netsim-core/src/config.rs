// Тут хранятся конфиги ядра симулятора (без механизма загрузки).

use crate::PacketSpec;

/// Описание начального события, загружаемого из конфига.
#[derive(Debug, Clone)]
pub struct InitialEventSpec {
    /// Агент-получатель события.
    pub agent_id: u32,
    /// Счетчик пакетов для детерминизма.
    pub packet_seq: u32,
    /// Полезная нагрузка пакета.
    pub packet: PacketSpec,
}

/// Базовая конфигурация симуляции для ядра.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// Количество агентов.
    pub agents_count: u32,
    /// Количество тиков для прогона.
    pub ticks: u64,
    /// Размер окна ring‑buffer для очереди событий.
    pub event_queue_window: u64,
    /// Начальные события, добавляемые до первого тика.
    pub initial_events: Vec<InitialEventSpec>,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            agents_count: 0,
            ticks: 0,
            event_queue_window: 64,
            initial_events: Vec::new(),
        }
    }
}
