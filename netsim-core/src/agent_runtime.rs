use crate::{AgentSoA, Event, MemoryId};

/// Память агента (notes) — заглушка для этапа 0.
#[derive(Debug, Default, Clone)]
pub struct AgentMemory {
    /// Идентификатор блока памяти.
    pub id: MemoryId,
}

/// Симулируемая таблица маршрутизации — заглушка для этапа 0.
#[derive(Debug, Default, Clone)]
pub struct RoutingTable {
    /// Количество записей в таблице.
    pub entries: u32,
}

/// Интерфейс алгоритма обработки событий агентом.
pub trait AgentAlgorithm {
    /// Обрабатывает событие и возвращает новое событие или `None`.
    fn eval_event(
        &self,
        agent_index: usize,
        agents: &AgentSoA,
        memory: &mut AgentMemory,
        routing: &mut RoutingTable,
        event: &Event,
    ) -> Option<Event>;
}

/// Интерфейс валидатора исходящих событий.
pub trait EventValidator {
    /// Возвращает `true`, если событие можно отправить.
    fn validate(&self, agent_index: usize, agents: &AgentSoA, event: &Event) -> bool;
}

/// Валидатор-заглушка: всегда разрешает отправку.
#[derive(Debug, Default, Clone, Copy)]
pub struct AllowAllValidator;

impl EventValidator for AllowAllValidator {
    fn validate(&self, _agent_index: usize, _agents: &AgentSoA, _event: &Event) -> bool {
        true
    }
}

/// Runtime, связывающий алгоритм и валидатор.
pub struct AgentRuntime {
    algorithm: Box<dyn AgentAlgorithm + Send + Sync>,
    validator: Box<dyn EventValidator + Send + Sync>,
}

impl std::fmt::Debug for AgentRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentRuntime").finish()
    }
}

impl AgentRuntime {
    /// Создает runtime с заданными алгоритмом и валидатором.
    pub fn new(
        algorithm: Box<dyn AgentAlgorithm + Send + Sync>,
        validator: Box<dyn EventValidator + Send + Sync>,
    ) -> Self {
        Self {
            algorithm,
            validator,
        }
    }

    /// Обрабатывает событие агентом с учетом валидации.
    pub fn handle_event(
        &self,
        agent_index: usize,
        agents: &AgentSoA,
        memory: &mut AgentMemory,
        routing: &mut RoutingTable,
        event: &Event,
    ) -> Option<Event> {
        let outgoing = self
            .algorithm
            .eval_event(agent_index, agents, memory, routing, event);

        outgoing.filter(|candidate| self.validator.validate(agent_index, agents, candidate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Packet, PacketSpec};

    #[derive(Debug, Default)]
    struct EchoAlgorithm;

    impl AgentAlgorithm for EchoAlgorithm {
        fn eval_event(
            &self,
            agent_index: usize,
            agents: &AgentSoA,
            _memory: &mut AgentMemory,
            _routing: &mut RoutingTable,
            event: &Event,
        ) -> Option<Event> {
            let agent_id = agents.agent_id[agent_index];
            let mut packet = event.payload;
            packet.src_id = agent_id;
            packet.dst_id = agent_id;
            packet.packet_id = packet.packet_id.wrapping_add(1);
            Some(Event::packet(agent_id, event.packet_seq + 1, packet))
        }
    }

    #[derive(Debug, Default)]
    struct DenyValidator;

    impl EventValidator for DenyValidator {
        fn validate(&self, _agent_index: usize, _agents: &AgentSoA, _event: &Event) -> bool {
            false
        }
    }

    fn event_for(agent_id: u32, packet_seq: u32) -> Event {
        let packet = Packet::from_spec(PacketSpec {
            packet_id: 10,
            src_id: agent_id,
            dst_id: agent_id,
            created_tick: 0,
            deliver_tick: 0,
            ttl: 1,
            size_bytes: 1,
            quality: 1.0,
            meta: false,
            route_hint: 0,
        });

        Event::packet(agent_id, packet_seq, packet)
    }

    #[test]
    fn runtime_allows_validated_outgoing_event() {
        let runtime = AgentRuntime::new(
            Box::new(EchoAlgorithm::default()),
            Box::new(AllowAllValidator::default()),
        );
        let agents = AgentSoA::new(1);
        let mut memory = AgentMemory::default();
        let mut routing = RoutingTable::default();
        let event = event_for(0, 1);

        let outgoing = runtime.handle_event(0, &agents, &mut memory, &mut routing, &event);
        assert!(outgoing.is_some());
    }

    #[test]
    fn runtime_blocks_denied_event() {
        let runtime = AgentRuntime::new(
            Box::new(EchoAlgorithm::default()),
            Box::new(DenyValidator::default()),
        );
        let agents = AgentSoA::new(1);
        let mut memory = AgentMemory::default();
        let mut routing = RoutingTable::default();
        let event = event_for(0, 1);

        let outgoing = runtime.handle_event(0, &agents, &mut memory, &mut routing, &event);
        assert!(outgoing.is_none());
    }
}
