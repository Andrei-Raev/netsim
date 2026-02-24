use crate::{AgentMemoryArena, AgentMemoryBlockMut, AgentSoA, Event, MemoryId, RouteEntry};
use std::marker::PhantomData;

/// Память агента: доступ к его блоку в общем пуле.
#[derive(Debug)]
pub struct AgentMemory<'a> {
    /// Идентификатор блока памяти.
    pub id: MemoryId,
    /// Mutable view на блок памяти агента.
    pub block: AgentMemoryBlockMut<'a>,
}

/// Таблица маршрутизации с доступом через raw‑указатели.
#[derive(Debug)]
pub struct RoutingTable<'a> {
    entries_ptr: *mut RouteEntry,
    entries_len: usize,
    mem_used_ptr: *mut u32,
    _marker: PhantomData<&'a mut [RouteEntry]>,
}

impl<'a> RoutingTable<'a> {
    /// Возвращает емкость таблицы.
    pub fn capacity(&self) -> u32 {
        self.entries_len as u32
    }

    /// Возвращает число активных записей.
    pub fn mem_used(&self) -> u32 {
        unsafe { *self.mem_used_ptr }
    }

    /// Устанавливает число активных записей.
    pub fn set_mem_used(&mut self, value: u32) {
        unsafe {
            *self.mem_used_ptr = value;
        }
    }

    /// Возвращает mutable-доступ к записям таблицы.
    pub fn entries_mut(&mut self) -> &mut [RouteEntry] {
        unsafe { std::slice::from_raw_parts_mut(self.entries_ptr, self.entries_len) }
    }
}

impl AgentMemory<'_> {
    /// Создает доступ к памяти агента по его идентификатору.
    pub fn new(arena: &mut AgentMemoryArena, id: MemoryId) -> AgentMemory<'_> {
        AgentMemory {
            id,
            block: arena.block_mut(id),
        }
    }

    /// Возвращает routing table view, привязанный к блоку памяти.
    pub fn routing_table(&mut self) -> RoutingTable<'_> {
        let (entries_ptr, entries_len, mem_used_ptr) = self.block.routing_table_ptrs();
        RoutingTable {
            entries_ptr,
            entries_len,
            mem_used_ptr,
            _marker: PhantomData,
        }
    }
}

/// Интерфейс алгоритма обработки событий агентом.
pub trait AgentAlgorithm {
    /// Обрабатывает событие и возвращает новое событие или `None`.
    fn eval_event(
        &self,
        agent_index: usize,
        agents: &AgentSoA,
        memory: &mut AgentMemory,
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
        event: &Event,
    ) -> Option<Event> {
        let outgoing = self
            .algorithm
            .eval_event(agent_index, agents, memory, event);

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
        let mut arena = AgentMemoryArena::new();
        let mut builder = crate::AgentMemoryBuilder::new(&mut arena);
        let spec = crate::AgentMemorySpec::placeholder(0);
        let (id, _) = builder.build(spec);
        let mut memory = AgentMemory::new(&mut arena, id);
        let _routing = memory.routing_table();
        let event = event_for(0, 1);

        let outgoing = runtime.handle_event(0, &agents, &mut memory, &event);
        assert!(outgoing.is_some());
    }

    #[test]
    fn runtime_blocks_denied_event() {
        let runtime = AgentRuntime::new(
            Box::new(EchoAlgorithm::default()),
            Box::new(DenyValidator::default()),
        );
        let agents = AgentSoA::new(1);
        let mut arena = AgentMemoryArena::new();
        let mut builder = crate::AgentMemoryBuilder::new(&mut arena);
        let spec = crate::AgentMemorySpec::placeholder(0);
        let (id, _) = builder.build(spec);
        let mut memory = AgentMemory::new(&mut arena, id);
        let event = event_for(0, 1);

        let outgoing = runtime.handle_event(0, &agents, &mut memory, &event);
        assert!(outgoing.is_none());
    }
}
