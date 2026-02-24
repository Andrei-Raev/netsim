use crate::{
    AgentMemory, AgentRuntime, AgentSoA, AllowAllValidator, Event, EventQueue, EventQueueConfig,
    RoutingTable, SimConfig, SimStats,
};

/// Результат прогона симуляции.
#[derive(Debug, Clone)]
pub struct SimResult {
    /// Число обработанных тиков.
    pub ticks_processed: u64,
    /// Итоговая статистика.
    pub stats: SimStats,
}

/// Минимальный CPU‑пайплайн симуляции (каркас).
#[derive(Debug)]
pub struct SimPipeline {
    /// Хранилище агентов.
    pub agents: AgentSoA,
    /// Статистика симуляции.
    pub stats: SimStats,
    /// Текущий тик.
    pub current_tick: u64,
    /// Очередь событий.
    pub event_queue: EventQueue,
    /// Runtime агентов.
    pub runtime: AgentRuntime,
    /// Память агентов.
    pub agent_memory: Vec<AgentMemory>,
    /// Таблицы маршрутизации агентов.
    pub routing_tables: Vec<RoutingTable>,
}

impl SimPipeline {
    /// Создает новый пайплайн с заданным количеством агентов.
    pub fn new(agent_count: usize) -> Self {
        let runtime = AgentRuntime::new(Box::new(AllowAllAlgorithm), Box::new(AllowAllValidator));
        let event_queue = EventQueue::new(EventQueueConfig { window_size: 64 });

        Self {
            agents: AgentSoA::new(agent_count),
            stats: SimStats::default(),
            current_tick: 0,
            event_queue,
            runtime,
            agent_memory: vec![AgentMemory::default(); agent_count],
            routing_tables: vec![RoutingTable::default(); agent_count],
        }
    }

    /// Создает пайплайн по конфигу ядра.
    pub fn from_config(config: SimConfig) -> Self {
        let runtime = AgentRuntime::new(Box::new(AllowAllAlgorithm), Box::new(AllowAllValidator));
        let event_queue = EventQueue::new(EventQueueConfig {
            window_size: config.event_queue_window,
        });

        Self {
            agents: AgentSoA::new(config.agents_count as usize),
            stats: SimStats::default(),
            current_tick: 0,
            event_queue,
            runtime,
            agent_memory: vec![AgentMemory::default(); config.agents_count as usize],
            routing_tables: vec![RoutingTable::default(); config.agents_count as usize],
        }
    }

    /// Выполняет один тик симуляции.
    pub fn step(&mut self) {
        self.process_current_events();
        self.current_tick = self.event_queue.current_tick();
        self.event_queue.advance();
    }

    /// Запускает симуляцию на заданное число тиков.
    pub fn run(&mut self, config: SimConfig) -> SimResult {
        for _ in 0..config.ticks {
            self.step();
        }

        SimResult {
            ticks_processed: config.ticks,
            stats: self.stats.clone(),
        }
    }

    /// Обрабатывает события текущего тика и обновляет статистику.
    pub fn process_current_events(&mut self) {
        let events = self.event_queue.pop_current();
        for event in events {
            let agent_index = event.agent_id as usize;
            if agent_index >= self.agents.len() {
                self.stats.packets_drop += 1;
                continue;
            }

            let memory = &mut self.agent_memory[agent_index];
            let routing = &mut self.routing_tables[agent_index];
            let outgoing =
                self.runtime
                    .handle_event(agent_index, &self.agents, memory, routing, &event);

            if let Some(outgoing_event) = outgoing {
                self.stats.packets_sent += 1;
                self.event_queue.push(outgoing_event);
            } else {
                self.stats.packets_recv += 1;
            }
        }
    }
}

/// Алгоритм-заглушка: не генерирует новых событий.
#[derive(Debug, Default, Clone, Copy)]
struct AllowAllAlgorithm;

impl crate::AgentAlgorithm for AllowAllAlgorithm {
    fn eval_event(
        &self,
        _agent_index: usize,
        _agents: &AgentSoA,
        _memory: &mut AgentMemory,
        _routing: &mut RoutingTable,
        _event: &Event,
    ) -> Option<Event> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, Packet, PacketSpec};

    fn event_for(agent_id: u32, packet_seq: u32, deliver_tick: u64) -> Event {
        let packet = Packet::from_spec(PacketSpec {
            packet_id: 1,
            src_id: agent_id,
            dst_id: agent_id,
            created_tick: 0,
            deliver_tick,
            ttl: 1,
            size_bytes: 1,
            quality: 1.0,
            meta: false,
            route_hint: 0,
        });

        Event::packet(agent_id, packet_seq, packet)
    }

    #[derive(Debug, Default)]
    struct EmitOnce;

    impl crate::AgentAlgorithm for EmitOnce {
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
            packet.deliver_tick = event.payload.deliver_tick + 1;
            Some(Event::packet(agent_id, event.packet_seq + 1, packet))
        }
    }

    #[test]
    fn pipeline_routes_event_and_emits_outgoing() {
        let mut pipeline = SimPipeline::new(1);
        pipeline.runtime =
            AgentRuntime::new(Box::new(EmitOnce::default()), Box::new(AllowAllValidator));

        pipeline
            .event_queue
            .push(event_for(0, 1, pipeline.event_queue.current_tick()));

        pipeline.process_current_events();

        assert_eq!(pipeline.stats.packets_sent, 1);
        assert_eq!(pipeline.stats.packets_recv, 0);
    }

    #[test]
    fn pipeline_drops_events_for_unknown_agent() {
        let mut pipeline = SimPipeline::new(0);
        pipeline
            .event_queue
            .push(event_for(1, 1, pipeline.event_queue.current_tick()));

        pipeline.process_current_events();

        assert_eq!(pipeline.stats.packets_drop, 1);
    }

    #[test]
    fn pipeline_respects_configured_event_queue_window() {
        let config = SimConfig {
            agents_count: 1,
            ticks: 1,
            event_queue_window: 3,
        };
        let pipeline = SimPipeline::from_config(config);

        assert_eq!(pipeline.event_queue.window_size(), 3);
    }
}
