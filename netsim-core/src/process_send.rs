use crate::{Event, WorldGrid};

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
        pos_x: f32,
        pos_y: f32,
    ) -> Event {
        let mut updated = event;
        updated.payload.ttl = updated.payload.ttl.saturating_sub(1);

        if let Some(grid) = world
            && let Some(cell) = grid.sample(pos_x, pos_y)
        {
            let noise = cell.noise;
            let quality = updated.payload.quality;
            updated.payload.quality = (quality - noise).max(0.0);
        }

        updated
    }
}
