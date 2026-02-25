use crate::Event;

/// Конфигурация очереди событий.
#[derive(Debug, Clone, Copy, Default)]
pub struct EventQueueConfig {
    /// Размер окна ring‑buffer в тиках.
    pub window_size: u64,
}

/// Слот ring‑buffer для конкретного тика.
#[derive(Debug, Clone)]
struct Slot {
    tick: u64,
    events: Vec<Event>,
}

impl Slot {
    fn new() -> Self {
        Self {
            tick: u64::MAX,
            events: Vec::new(),
        }
    }

    fn clear_for_tick(&mut self, tick: u64) {
        self.tick = tick;
        self.events.clear();
    }
}

/// Детерминированная очередь событий на основе ring‑buffer.
#[derive(Debug)]
pub struct EventQueue {
    window_size: u64,
    current_tick: u64,
    slots: Vec<Slot>,
    overflow: Vec<Event>,
}

impl EventQueue {
    /// Создает очередь с заданным размером окна.
    pub fn new(config: EventQueueConfig) -> Self {
        let window_size = config.window_size.max(1);
        let mut slots = Vec::with_capacity(window_size as usize);
        for _ in 0..window_size {
            slots.push(Slot::new());
        }

        Self {
            window_size,
            current_tick: 0,
            slots,
            overflow: Vec::new(),
        }
    }

    /// Возвращает текущий тик очереди.
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Возвращает размер окна ring‑buffer.
    pub fn window_size(&self) -> u64 {
        self.window_size
    }

    /// Добавляет одно событие в очередь.
    pub fn push(&mut self, event: Event) {
        self.push_batch(std::iter::once(event));
    }

    /// Добавляет пакет событий (локальный список после merge).
    pub fn push_batch<I>(&mut self, events: I)
    where
        I: IntoIterator<Item = Event>,
    {
        for event in events {
            self.insert_event(event);
        }
    }

    /// Извлекает события текущего тика в детерминированном порядке.
    pub fn pop_current(&mut self) -> Vec<Event> {
        let index = (self.current_tick % self.window_size) as usize;
        let slot = &mut self.slots[index];

        if slot.tick != self.current_tick {
            return Vec::new();
        }

        let mut events = Vec::new();
        std::mem::swap(&mut events, &mut slot.events);
        events
    }

    /// Переходит к следующему тику.
    pub fn advance(&mut self) {
        self.current_tick += 1;
    }

    /// Возвращает количество событий в overflow‑буфере.
    pub fn overflow_len(&self) -> usize {
        self.overflow.len()
    }

    fn insert_event(&mut self, event: Event) {
        let deliver_tick = event.payload.deliver_tick;
        if deliver_tick < self.current_tick || deliver_tick >= self.current_tick + self.window_size
        {
            self.overflow.push(event);
            return;
        }

        let index = (deliver_tick % self.window_size) as usize;
        let slot = &mut self.slots[index];

        if slot.tick != deliver_tick {
            slot.clear_for_tick(deliver_tick);
        }

        slot.events.push(event);
        slot.events
            .sort_unstable_by_key(|item| (item.agent_id, item.packet_seq));
    }
}

#[cfg(test)]
mod tests {
    use super::{EventQueue, EventQueueConfig};
    use crate::{Event, Packet, PacketSpec};

    fn packet_for(agent_id: u32, packet_seq: u32, deliver_tick: u64) -> Event {
        let packet = Packet::from_spec(PacketSpec {
            packet_id: u64::from(agent_id) << 32 | u64::from(packet_seq),
            src_id: agent_id,
            dst_id: agent_id,
            created_tick: 0,
            deliver_tick,
            ttl: 1,
            size_bytes: 1,
            quality: 1.0,
            meta: false,
            trg_id: 0,
            route_hint: 0,
        });

        Event::packet(agent_id, packet_seq, packet)
    }

    #[test]
    fn pop_current_returns_sorted_events() {
        let mut queue = EventQueue::new(EventQueueConfig { window_size: 4 });
        let first = packet_for(2, 5, 0);
        let second = packet_for(1, 7, 0);
        let third = packet_for(1, 3, 0);

        queue.push_batch(vec![first, second, third]);

        let events = queue.pop_current();
        let keys: Vec<(u32, u32)> = events
            .iter()
            .map(|event| (event.agent_id, event.packet_seq))
            .collect();

        assert_eq!(keys, vec![(1, 3), (1, 7), (2, 5)]);
    }

    #[test]
    fn events_outside_window_go_to_overflow() {
        let mut queue = EventQueue::new(EventQueueConfig { window_size: 2 });
        queue.push(packet_for(1, 0, 5));

        assert_eq!(queue.overflow_len(), 1);
        assert!(queue.pop_current().is_empty());
    }

    #[test]
    fn ring_reuses_slots_for_future_ticks() {
        let mut queue = EventQueue::new(EventQueueConfig { window_size: 2 });
        queue.push(packet_for(1, 0, 0));
        assert_eq!(queue.pop_current().len(), 1);

        queue.advance();
        queue.push(packet_for(1, 1, 2));

        assert!(queue.pop_current().is_empty());
        queue.advance();
        let events = queue.pop_current();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].packet_seq, 1);
    }
}
