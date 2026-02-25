use crate::{
    AgentMemory, AgentMemoryArena, AgentRuntime, AgentSoA, AllowAllValidator, Event, EventQueue,
    EventQueueConfig, Packet, ProcessSend, SimConfig, SimStats, StatisticsCollector, StatsSample,
    WorldGrid, WorldGridGenerator,
};
use crate::{
    ScenarioConfig, ScenarioEventSpec, SpawnShape, TrafficAreaShape, TrafficAreaSpec, TrafficSpec,
    TrafficTargetSpec,
};

/// Спецификация пакета для постановки в очередь.
#[derive(Debug, Clone, Copy)]
struct TrafficPacketSpec {
    /// Тик отправки.
    tick: u64,
    /// Идентификатор пакета.
    packet_id: u64,
    /// Агент-источник.
    src_id: u32,
    /// Агент-получатель.
    dst_id: u32,
    /// TTL пакета.
    ttl: u16,
    /// Размер пакета в байтах.
    size_bytes: u32,
    /// Показатель качества/шума сигнала.
    quality: f32,
    /// Признак служебного пакета.
    meta: bool,
    /// Идентификатор конечного адресата.
    trg_id: u32,
    /// Подсказка следующего хопа.
    route_hint: u32,
}

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
    /// Общий пул памяти агентов.
    pub memory_arena: AgentMemoryArena,
    /// Текущая сетка мира (CPU‑референс).
    pub world_grid: Option<WorldGrid>,
    /// Порог шума для дропа пакетов.
    pub world_noise_drop_threshold: f32,
    /// Обработчик отправки пакетов.
    pub process_send: ProcessSend,
    /// Сборщик статистики.
    pub statistics_collector: StatisticsCollector,
}

impl SimPipeline {
    /// Создает новый пайплайн с заданным количеством агентов.
    pub fn new(agent_count: usize) -> Self {
        let runtime = AgentRuntime::new(Box::new(AllowAllAlgorithm), Box::new(AllowAllValidator));
        let event_queue = EventQueue::new(EventQueueConfig { window_size: 64 });
        let mut memory_arena = AgentMemoryArena::new();
        let mut agents = AgentSoA::new(agent_count);

        let mut builder = crate::AgentBuilder::new(&mut memory_arena);
        for index in 0..agent_count {
            let spec = crate::AgentSpec::placeholder(index as u32);
            builder.build(&mut agents, index, spec);
        }

        Self {
            agents,
            stats: SimStats::default(),
            current_tick: 0,
            event_queue,
            runtime,
            memory_arena,
            world_grid: None,
            world_noise_drop_threshold: 0.0,
            process_send: ProcessSend,
            statistics_collector: StatisticsCollector,
        }
    }

    /// Создает пайплайн по конфигу ядра.
    /// Обновляет runtime пайплайна.
    pub fn set_runtime(&mut self, runtime: AgentRuntime) {
        self.runtime = runtime;
    }

    pub fn from_config(config: SimConfig) -> Self {
        let runtime = AgentRuntime::new(Box::new(AllowAllAlgorithm), Box::new(AllowAllValidator));
        let mut event_queue = EventQueue::new(EventQueueConfig {
            window_size: config.event_queue_window,
        });
        let mut memory_arena = AgentMemoryArena::new();
        let mut agents = AgentSoA::new(config.agents_count as usize);

        let mut builder = crate::AgentBuilder::new(&mut memory_arena);
        for index in 0..config.agents_count as usize {
            let spec = crate::AgentSpec::placeholder(index as u32);
            builder.build(&mut agents, index, spec);
        }

        for initial in &config.initial_events {
            let packet = Packet::from_spec(initial.packet);
            let event = Event::packet(initial.agent_id, initial.packet_seq, packet);
            event_queue.push(event);
        }

        Self {
            agents,
            stats: SimStats::default(),
            current_tick: 0,
            event_queue,
            runtime,
            memory_arena,
            world_grid: None,
            world_noise_drop_threshold: 0.0,
            process_send: ProcessSend,
            statistics_collector: StatisticsCollector,
        }
    }

    /// Создает пайплайн по сценарию симуляции.
    pub fn from_scenario(config: &ScenarioConfig) -> Self {
        let runtime = AgentRuntime::new(Box::new(AllowAllAlgorithm), Box::new(AllowAllValidator));
        let event_queue = EventQueue::new(EventQueueConfig {
            window_size: config.event_queue_window,
        });
        let memory_arena = AgentMemoryArena::new();
        let agents = AgentSoA::new(0);

        Self {
            agents,
            stats: SimStats::default(),
            current_tick: 0,
            event_queue,
            runtime,
            memory_arena,
            world_grid: None,
            world_noise_drop_threshold: config.noise_drop_threshold,
            process_send: ProcessSend,
            statistics_collector: StatisticsCollector,
        }
    }

    /// Выполняет один тик симуляции.
    pub fn step(&mut self) {
        self.process_current_events();
        self.flush_stats_for_tick(self.event_queue.current_tick());
        self.current_tick = self.event_queue.current_tick();
        self.event_queue.advance();
    }

    /// Выполняет один тик симуляции вместе с генерацией мира.
    pub fn step_with_world<G>(&mut self, generator: &G)
    where
        G: WorldGridGenerator,
    {
        let tick = self.event_queue.current_tick();
        let grid = generator.build_grid(tick);
        self.set_world_grid(grid);
        self.process_current_events();
        self.flush_stats_for_tick(tick);
        self.current_tick = tick;
        self.event_queue.advance();
    }

    /// Выполняет один тик сценария: применяет события и обрабатывает очередь.
    pub fn step_with_scenario<G>(&mut self, scenario: &ScenarioConfig, generator: &G)
    where
        G: WorldGridGenerator,
    {
        let tick = self.event_queue.current_tick();
        self.apply_scenario_tick(scenario, tick);
        let grid = generator.build_grid(tick);
        self.set_world_grid(grid);
        self.process_current_events();
        self.flush_stats_for_tick(tick);
        self.current_tick = tick;
        self.event_queue.advance();
    }

    /// Устанавливает сетку мира для текущего тика.
    pub fn set_world_grid(&mut self, grid: WorldGrid) {
        self.world_grid = Some(grid);
    }

    /// Сбрасывает сетку мира (симуляция без влияния мира).
    pub fn clear_world_grid(&mut self) {
        self.world_grid = None;
    }

    /// Настраивает порог шума, при котором пакет дропается.
    pub fn set_world_noise_drop_threshold(&mut self, threshold: f32) {
        self.world_noise_drop_threshold = threshold.max(0.0);
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

    /// Запускает симуляцию на заданное число тиков с генерацией мира.
    pub fn run_with_world<G>(&mut self, ticks: u64, generator: &G) -> SimResult
    where
        G: WorldGridGenerator,
    {
        for _ in 0..ticks {
            self.step_with_world(generator);
        }

        SimResult {
            ticks_processed: ticks,
            stats: self.stats.clone(),
        }
    }

    /// Запускает сценарий на заданное число тиков.
    pub fn run_with_scenario<G>(&mut self, scenario: &ScenarioConfig, generator: &G) -> SimResult
    where
        G: WorldGridGenerator,
    {
        for _ in 0..scenario.ticks {
            self.step_with_scenario(scenario, generator);
        }

        SimResult {
            ticks_processed: scenario.ticks,
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

            let id = self.agents.memory_id[agent_index];
            let should_drop_world = self.should_drop_by_world(&event);
            let mut memory = AgentMemory::new(&mut self.memory_arena, id);

            if event.payload.ttl == 0 {
                self.stats.packets_drop += 1;
                self.statistics_collector.on_drop(&mut memory);
                continue;
            }

            if should_drop_world {
                self.stats.packets_drop += 1;
                self.statistics_collector.on_drop(&mut memory);
                continue;
            }

            let pos_x = self.agents.pos_x[agent_index];
            let pos_y = self.agents.pos_y[agent_index];
            let event = self
                .process_send
                .process(event, self.world_grid.as_ref(), pos_x, pos_y);

            if let Some(grid) = self.world_grid.as_ref() {
                let cell = grid.sample(pos_x, pos_y);
                self.statistics_collector.on_world_cell(&mut memory, cell);
            }

            let outgoing =
                self.runtime
                    .handle_event(agent_index, &self.agents, &mut memory, &event);

            if let Some(outgoing_event) = outgoing {
                self.stats.packets_sent += 1;
                self.statistics_collector
                    .on_send(&mut memory, outgoing_event.payload.quality);
                self.event_queue.push(outgoing_event);
            } else {
                self.stats.packets_recv += 1;
                self.statistics_collector
                    .on_receive(&mut memory, event.payload.quality);
            }
        }
    }

    fn flush_stats_for_tick(&mut self, tick: u64) {
        for index in 0..self.agents.len() {
            if !self.agents.alive[index] {
                continue;
            }
            let id = self.agents.memory_id[index];
            let memory = AgentMemory::new(&mut self.memory_arena, id);
            let collect_every = memory.block.descriptor().collect_every;
            if collect_every == 0 || !tick.is_multiple_of(collect_every) {
                continue;
            }
            let stats = memory.block.stats();
            self.statistics_collector.flush_agent(
                &mut self.memory_arena,
                id,
                StatsSample::from(stats),
            );
        }
    }

    fn apply_scenario_tick(&mut self, scenario: &ScenarioConfig, tick: u64) {
        for event in scenario.events_for_tick(tick) {
            match event {
                ScenarioEventSpec::SpawnAgents(spec) => {
                    self.apply_spawn_event(spec);
                }
                ScenarioEventSpec::Traffic(spec) => {
                    self.apply_traffic_event(tick, spec);
                }
                ScenarioEventSpec::TrafficArea(spec) => {
                    self.apply_traffic_area_event(tick, spec);
                }
            }
        }
    }

    fn apply_spawn_event(&mut self, spec: crate::SpawnAgentsSpec) {
        if spec.count == 0 {
            return;
        }
        let start_index = self.agents.len();
        let count = spec.count as usize;
        self.agents.extend(count);

        let mut builder = crate::AgentBuilder::new(&mut self.memory_arena);
        for index in 0..count {
            let agent_id = spec.agent_id_start.saturating_add(index as u32);
            let spec = spec.spec_for_index(agent_id);
            builder.build(&mut self.agents, start_index + index, spec);
        }

        self.apply_spawn_positions(spec, start_index, count);
    }

    fn apply_spawn_positions(
        &mut self,
        spec: crate::SpawnAgentsSpec,
        start_index: usize,
        count: usize,
    ) {
        match spec.shape {
            SpawnShape::Grid {
                rows,
                cols,
                spacing,
                origin_x,
                origin_y,
            } => {
                let _rows = rows.max(1) as usize;
                let cols = cols.max(1) as usize;
                for index in 0..count {
                    let row = index / cols;
                    let col = index % cols;
                    let x = origin_x + col as f32 * spacing;
                    let y = origin_y + row as f32 * spacing;
                    let agent_index = start_index + index;
                    if agent_index < self.agents.len() {
                        self.agents.pos_x[agent_index] = x;
                        self.agents.pos_y[agent_index] = y;
                    }
                }
            }
            SpawnShape::Circle {
                center_x,
                center_y,
                radius,
            } => {
                let total = count.max(1) as f32;
                for index in 0..count {
                    let t = index as f32 / total;
                    let angle = std::f32::consts::TAU * t;
                    let x = center_x + radius * angle.cos();
                    let y = center_y + radius * angle.sin();
                    let agent_index = start_index + index;
                    if agent_index < self.agents.len() {
                        self.agents.pos_x[agent_index] = x;
                        self.agents.pos_y[agent_index] = y;
                    }
                }
            }
        }
    }

    fn apply_traffic_event(&mut self, tick: u64, spec: TrafficSpec) {
        if self.agents.is_empty() {
            self.stats.packets_drop += 1;
            return;
        }

        self.enqueue_packet(TrafficPacketSpec {
            tick,
            packet_id: spec.packet_id,
            src_id: spec.src_id,
            dst_id: spec.dst_id,
            ttl: spec.ttl,
            size_bytes: spec.size_bytes,
            quality: spec.quality,
            meta: spec.meta,
            trg_id: spec.trg_id,
            route_hint: spec.route_hint,
        });
    }

    fn apply_traffic_area_event(&mut self, tick: u64, spec: TrafficAreaSpec) {
        if self.agents.is_empty() {
            self.stats.packets_drop += 1;
            return;
        }

        let mut queued = 0usize;
        let mut packet_id = spec.template.packet_id_base;

        for agent_index in 0..self.agents.len() {
            if !self.agents.alive[agent_index] {
                continue;
            }
            let pos_x = self.agents.pos_x[agent_index];
            let pos_y = self.agents.pos_y[agent_index];
            if !self.is_inside_traffic_area(&spec.area, pos_x, pos_y) {
                continue;
            }

            let src_id = self.agents.agent_id[agent_index];
            let dst_id = match spec.target {
                TrafficTargetSpec::Fixed { dst_id } => dst_id,
                TrafficTargetSpec::SelfTarget => src_id,
            };

            self.enqueue_packet(TrafficPacketSpec {
                tick,
                packet_id,
                src_id,
                dst_id,
                ttl: spec.template.ttl,
                size_bytes: spec.template.size_bytes,
                quality: spec.template.quality,
                meta: spec.template.meta,
                trg_id: spec.template.trg_id,
                route_hint: spec.template.route_hint,
            });

            packet_id = packet_id.saturating_add(1);
            queued += 1;
        }

        if queued == 0 {
            self.stats.packets_drop += 1;
        }
    }

    fn enqueue_packet(&mut self, spec: TrafficPacketSpec) {
        let agent_index = spec.src_id as usize;
        if agent_index >= self.agents.len() {
            self.stats.packets_drop += 1;
            return;
        }

        let packet = Packet::from_spec(crate::PacketSpec {
            packet_id: spec.packet_id,
            src_id: spec.src_id,
            dst_id: spec.dst_id,
            created_tick: spec.tick,
            deliver_tick: spec.tick,
            ttl: spec.ttl,
            size_bytes: spec.size_bytes,
            quality: spec.quality,
            meta: spec.meta,
            trg_id: spec.trg_id,
            route_hint: spec.route_hint,
        });

        let packet_seq = self.agents.packet_seq[agent_index];
        self.agents.packet_seq[agent_index] = packet_seq.saturating_add(1);
        let event = Event::packet(spec.src_id, packet_seq, packet);
        self.event_queue.push(event);
    }

    fn is_inside_traffic_area(&self, area: &TrafficAreaShape, pos_x: f32, pos_y: f32) -> bool {
        match *area {
            TrafficAreaShape::Grid { min, max } => match &self.world_grid {
                Some(grid) => match grid.world_to_cell(pos_x, pos_y) {
                    Some((cx, cy)) => cx >= min.0 && cy >= min.1 && cx <= max.0 && cy <= max.1,
                    None => false,
                },
                None => {
                    let cx = pos_x.floor() as i64;
                    let cy = pos_y.floor() as i64;
                    if cx < 0 || cy < 0 {
                        return false;
                    }
                    let cx = cx as usize;
                    let cy = cy as usize;
                    cx >= min.0 && cy >= min.1 && cx <= max.0 && cy <= max.1
                }
            },
            TrafficAreaShape::Circle {
                center_x,
                center_y,
                radius,
            } => {
                let dx = pos_x - center_x;
                let dy = pos_y - center_y;
                dx * dx + dy * dy <= radius * radius
            }
            TrafficAreaShape::Rect {
                min_x,
                min_y,
                max_x,
                max_y,
            } => pos_x >= min_x && pos_y >= min_y && pos_x <= max_x && pos_y <= max_y,
        }
    }

    fn should_drop_by_world(&self, event: &Event) -> bool {
        if self.world_noise_drop_threshold <= 0.0 {
            return false;
        }
        let grid = match &self.world_grid {
            Some(grid) => grid,
            None => return false,
        };

        let agent_index = event.agent_id as usize;
        if agent_index >= self.agents.len() {
            return false;
        }
        let pos_x = self.agents.pos_x[agent_index];
        let pos_y = self.agents.pos_y[agent_index];
        let cell = match grid.sample(pos_x, pos_y) {
            Some(cell) => cell,
            None => return false,
        };

        cell.noise >= self.world_noise_drop_threshold
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
        _event: &Event,
    ) -> Option<Event> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, Packet, PacketSpec, WorldGrid};

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
            trg_id: 0,
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
    fn pipeline_drops_events_with_zero_ttl() {
        let mut pipeline = SimPipeline::new(1);
        let packet = Packet::from_spec(PacketSpec {
            packet_id: 1,
            src_id: 0,
            dst_id: 0,
            created_tick: 0,
            deliver_tick: pipeline.event_queue.current_tick(),
            ttl: 0,
            size_bytes: 1,
            quality: 1.0,
            meta: false,
            trg_id: 0,
            route_hint: 0,
        });
        pipeline.event_queue.push(Event::packet(0, 1, packet));

        pipeline.process_current_events();

        assert_eq!(pipeline.stats.packets_drop, 1);
        assert_eq!(pipeline.stats.packets_recv, 0);
        assert_eq!(pipeline.stats.packets_sent, 0);
    }

    #[test]
    fn pipeline_decrements_ttl_for_processed_events() {
        let mut pipeline = SimPipeline::new(1);
        let packet = Packet::from_spec(PacketSpec {
            packet_id: 1,
            src_id: 0,
            dst_id: 0,
            created_tick: 0,
            deliver_tick: pipeline.event_queue.current_tick(),
            ttl: 2,
            size_bytes: 1,
            quality: 1.0,
            meta: false,
            trg_id: 0,
            route_hint: 0,
        });
        pipeline.event_queue.push(Event::packet(0, 1, packet));

        pipeline.process_current_events();

        assert_eq!(pipeline.stats.packets_recv, 1);
    }

    #[test]
    fn pipeline_respects_configured_event_queue_window() {
        let config = SimConfig {
            agents_count: 1,
            ticks: 1,
            event_queue_window: 3,
            initial_events: Vec::new(),
        };
        let pipeline = SimPipeline::from_config(config);

        assert_eq!(pipeline.event_queue.window_size(), 3);
    }

    #[test]
    fn pipeline_seeds_initial_events_from_config() {
        let packet = Packet::from_spec(PacketSpec {
            packet_id: 7,
            src_id: 0,
            dst_id: 0,
            created_tick: 0,
            deliver_tick: 0,
            ttl: 1,
            size_bytes: 1,
            quality: 1.0,
            meta: false,
            trg_id: 0,
            route_hint: 0,
        });
        let config = SimConfig {
            agents_count: 1,
            ticks: 1,
            event_queue_window: 4,
            initial_events: vec![crate::InitialEventSpec {
                agent_id: 0,
                packet_seq: 5,
                packet: PacketSpec {
                    packet_id: 7,
                    src_id: 0,
                    dst_id: 0,
                    created_tick: 0,
                    deliver_tick: 0,
                    ttl: 1,
                    size_bytes: 1,
                    quality: 1.0,
                    meta: false,
                    trg_id: 0,
                    route_hint: 0,
                },
            }],
        };

        let mut pipeline = SimPipeline::from_config(config);
        let events = pipeline.event_queue.pop_current();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].agent_id, 0);
        assert_eq!(events[0].packet_seq, 5);
        assert_eq!(events[0].payload, packet);
    }

    #[test]
    fn pipeline_drops_events_when_world_noise_exceeds_threshold() {
        let mut pipeline = SimPipeline::new(1);
        pipeline.agents.pos_x[0] = 0.5;
        pipeline.agents.pos_y[0] = 0.5;
        pipeline.set_world_noise_drop_threshold(0.5);

        let grid = WorldGrid {
            width: 1,
            height: 1,
            cell_size: 1.0,
            cells: vec![crate::WorldCell {
                load: 0.0,
                noise: 1.0,
                bandwidth: 1.0,
                cost: 1.0,
            }],
        };
        pipeline.set_world_grid(grid);

        let packet = Packet::from_spec(PacketSpec {
            packet_id: 1,
            src_id: 0,
            dst_id: 0,
            created_tick: 0,
            deliver_tick: pipeline.event_queue.current_tick(),
            ttl: 2,
            size_bytes: 1,
            quality: 1.0,
            meta: false,
            trg_id: 0,
            route_hint: 0,
        });
        pipeline.event_queue.push(Event::packet(0, 1, packet));

        pipeline.process_current_events();

        assert_eq!(pipeline.stats.packets_drop, 1);
        assert_eq!(pipeline.stats.packets_sent, 0);
        assert_eq!(pipeline.stats.packets_recv, 0);
    }

    #[test]
    fn pipeline_keeps_events_without_world_grid() {
        let mut pipeline = SimPipeline::new(1);
        pipeline.set_world_noise_drop_threshold(0.5);

        let packet = Packet::from_spec(PacketSpec {
            packet_id: 1,
            src_id: 0,
            dst_id: 0,
            created_tick: 0,
            deliver_tick: pipeline.event_queue.current_tick(),
            ttl: 2,
            size_bytes: 1,
            quality: 1.0,
            meta: false,
            trg_id: 0,
            route_hint: 0,
        });
        pipeline.event_queue.push(Event::packet(0, 1, packet));

        pipeline.process_current_events();

        assert_eq!(pipeline.stats.packets_drop, 0);
        assert_eq!(pipeline.stats.packets_recv, 1);
    }
}
