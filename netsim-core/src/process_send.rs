use crate::statistics::should_drop_by_load;
use crate::{Event, WorldGrid};

/// Результат обработки отправки.
#[derive(Debug, Clone, Copy)]
pub struct ProcessSendResult {
    pub event: Event,
    pub dropped: bool,
}

/// Процессор отправки: модифицирует событие на основе мира,
/// потому что физику нельзя доверять логике агента.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProcessSend;

impl ProcessSend {
    /// Обрабатывает отправку и возвращает обновленное событие.
    pub fn process(
        &self,
        event: Event,
        world: Option<&WorldGrid>,
        world_seed: u64,
        pos_x: f32,
        pos_y: f32,
    ) -> ProcessSendResult {
        let mut updated = event;
        updated.payload.ttl = updated.payload.ttl.saturating_sub(1);

        let mut dropped = false;
        if let Some(grid) = world
            && let Some(cell) = grid.sample(pos_x, pos_y)
        {
            let noise = cell.noise;
            let quality = updated.payload.quality;
            updated.payload.quality = (quality - noise).max(0.0);

            dropped = should_drop_by_load(world_seed, updated.payload.packet_id, cell.load);
        }

        ProcessSendResult {
            event: updated,
            dropped,
        }
    }
}
