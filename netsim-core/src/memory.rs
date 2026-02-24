/// Идентификатор блока памяти агента в общем пуле.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryId {
    /// Базовый оффсет в пуле.
    pub base: u32,
    /// Длина блока в байтах.
    pub len: u32,
}

impl MemoryId {
    /// Создает новый идентификатор блока в пуле.
    pub fn new(base: u32, len: u32) -> Self {
        Self { base, len }
    }
}
