use crate::packet::Packet;

/// Типы событий, поддерживаемые ядром симулятора.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// Событие доставки пакета.
    Packet,
    /// Системное сервисное событие.
    System,
    /// Управляющее/оркестрационное событие.
    Control,
}

/// Детерминированный контейнер события для очереди.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    /// Категория события.
    pub kind: EventKind,
    /// Агент, который получает или владеет событием.
    pub agent_id: u32,
    /// Счетчик пакетов на агента для детерминизма.
    pub packet_seq: u32,
    /// Полезная нагрузка для пакетных событий.
    pub payload: Packet,
}

impl Event {
    /// Создает пакетное событие для агента и номера последовательности.
    pub fn packet(agent_id: u32, packet_seq: u32, payload: Packet) -> Self {
        Self {
            kind: EventKind::Packet,
            agent_id,
            packet_seq,
            payload,
        }
    }

    /// Возвращает ключ детерминированного порядка событий.
    pub fn sort_key(&self) -> (u64, u32, u32) {
        (self.payload.deliver_tick, self.agent_id, self.packet_seq)
    }
}
