/// Статистика обработки пакетов за время симуляции.
#[derive(Debug, Clone, Default)]
pub struct SimStats {
    /// Всего отправлено пакетов.
    pub packets_sent: u64,
    /// Всего получено пакетов.
    pub packets_recv: u64,
    /// Всего дропнуто пакетов.
    pub packets_drop: u64,
}

impl SimStats {
    /// Обнуляет счетчики статистики.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
