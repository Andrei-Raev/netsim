use crate::memory::{AgentMemoryArena, AgentMemoryBuilder, AgentMemorySpec, MemoryId};

/// Спецификация агента, используемая при построении.
#[derive(Debug, Clone, Copy)]
pub struct AgentSpec {
    /// Идентификатор агента.
    pub agent_id: u32,
    /// Идентификатор типа агента.
    pub type_id: u16,
    /// Емкость таблицы маршрутизации.
    pub routing_cap: u32,
    /// Размер scratchpad в байтах.
    pub scratch_cap: u32,
    /// Вычислительная мощность.
    pub compute_power: f32,
    /// Пропускная способность.
    pub bandwidth: f32,
    /// Ограничение скорости движения.
    pub self_speed: f32,
    /// Явно заданная емкость памяти (0 = вычислить автоматически).
    pub memory_cap: u32,
    /// Период сборки метрик по агенту (0 = без сборки).
    pub collect_every: u64,
}

impl AgentSpec {
    /// Создает временный spec без данных генератора мира.
    pub fn placeholder(agent_id: u32) -> Self {
        Self {
            agent_id,
            type_id: 0,
            routing_cap: 8,
            scratch_cap: 64,
            compute_power: 0.0,
            bandwidth: 0.0,
            self_speed: 0.0,
            memory_cap: 0,
            collect_every: 0,
        }
    }
}

/// Фабрика для создания агента и его памяти.
pub struct AgentBuilder<'a> {
    memory_builder: AgentMemoryBuilder<'a>,
}

impl<'a> AgentBuilder<'a> {
    /// Создает билдер агента на базе общего пула памяти.
    pub fn new(arena: &'a mut AgentMemoryArena) -> Self {
        Self {
            memory_builder: AgentMemoryBuilder::new(arena),
        }
    }

    /// Создает память агента, заполняет SoA и возвращает MemoryId.
    pub fn build(&mut self, agents: &mut AgentSoA, index: usize, spec: AgentSpec) -> MemoryId {
        let (memory_id, layout) = self.memory_builder.build(AgentMemorySpec {
            routing_cap: spec.routing_cap,
            scratch_cap: spec.scratch_cap,
            compute_power: spec.compute_power,
            bandwidth: spec.bandwidth,
            self_speed: spec.self_speed,
            agent_id: spec.agent_id,
            type_id: spec.type_id,
            memory_cap: spec.memory_cap,
        });

        {
            let mut block = self.memory_builder.block_mut(memory_id);
            block.update_descriptor_collect_every(spec.collect_every);
        }

        let memory_cap = if spec.memory_cap == 0 {
            layout.total_len
        } else {
            spec.memory_cap
        };

        agents.agent_id[index] = spec.agent_id;
        agents.alive[index] = true;
        agents.is_static[index] = false;
        agents.type_id[index] = spec.type_id;
        agents.packet_seq[index] = 0;
        agents.pos_x[index] = 0.0;
        agents.pos_y[index] = 0.0;
        agents.vel_x[index] = 0.0;
        agents.vel_y[index] = 0.0;
        agents.target_x[index] = 0.0;
        agents.target_y[index] = 0.0;
        agents.self_speed[index] = spec.self_speed;
        agents.energy[index] = 0.0;
        agents.memory_cap[index] = memory_cap;
        agents.mem_used[index] = 0;
        agents.compute_power[index] = spec.compute_power;
        agents.bandwidth[index] = spec.bandwidth;
        agents.packets_sent[index] = 0;
        agents.packets_recv[index] = 0;
        agents.packets_drop[index] = 0;
        agents.meta_packets_sent[index] = 0;
        agents.meta_packets_recv[index] = 0;
        agents.memory_id[index] = memory_id;

        memory_id
    }
}

/// Хранилище агентов в формате Structure-of-Arrays (SoA).
#[derive(Debug, Clone)]
pub struct AgentSoA {
    /// Стабильные идентификаторы агентов.
    pub agent_id: Vec<u32>,
    /// Флаг активности агента.
    pub alive: Vec<bool>,
    /// Признак статичности (не двигается).
    pub is_static: Vec<bool>,
    /// Идентификатор типа агента.
    pub type_id: Vec<u16>,
    /// Счетчик пакетов на агента для детерминизма.
    pub packet_seq: Vec<u32>,
    /// Позиция по X.
    pub pos_x: Vec<f32>,
    /// Позиция по Y.
    pub pos_y: Vec<f32>,
    /// Скорость по X.
    pub vel_x: Vec<f32>,
    /// Скорость по Y.
    pub vel_y: Vec<f32>,
    /// Целевая позиция по X.
    pub target_x: Vec<f32>,
    /// Целевая позиция по Y.
    pub target_y: Vec<f32>,
    /// Ограничение скорости движения по целям.
    pub self_speed: Vec<f32>,
    /// Текущий уровень энергии.
    pub energy: Vec<f32>,
    /// Емкость памяти агента.
    pub memory_cap: Vec<u32>,
    /// Использованная память (кешированное значение).
    pub mem_used: Vec<u32>,
    /// Параметр вычислительной мощности.
    pub compute_power: Vec<f32>,
    /// Параметр пропускной способности.
    pub bandwidth: Vec<f32>,
    /// Количество отправленных пакетов.
    pub packets_sent: Vec<u64>,
    /// Количество полученных пакетов.
    pub packets_recv: Vec<u64>,
    /// Количество дропнутых пакетов.
    pub packets_drop: Vec<u64>,
    /// Количество служебных отправленных пакетов.
    pub meta_packets_sent: Vec<u64>,
    /// Количество служебных полученных пакетов.
    pub meta_packets_recv: Vec<u64>,
    /// Идентификатор блока памяти агента.
    pub memory_id: Vec<MemoryId>,
}

impl AgentSoA {
    /// Создает SoA с фиксированным числом агентов.
    pub fn new(count: usize) -> Self {
        let mut agent_id = Vec::with_capacity(count);
        for id in 0..count {
            agent_id.push(id as u32);
        }

        Self {
            agent_id,
            alive: vec![true; count],
            is_static: vec![false; count],
            type_id: vec![0; count],
            packet_seq: vec![0; count],
            pos_x: vec![0.0; count],
            pos_y: vec![0.0; count],
            vel_x: vec![0.0; count],
            vel_y: vec![0.0; count],
            target_x: vec![0.0; count],
            target_y: vec![0.0; count],
            self_speed: vec![0.0; count],
            energy: vec![0.0; count],
            memory_cap: vec![0; count],
            mem_used: vec![0; count],
            compute_power: vec![0.0; count],
            bandwidth: vec![0.0; count],
            packets_sent: vec![0; count],
            packets_recv: vec![0; count],
            packets_drop: vec![0; count],
            meta_packets_sent: vec![0; count],
            meta_packets_recv: vec![0; count],
            memory_id: vec![MemoryId::default(); count],
        }
    }

    /// Возвращает количество агентов в SoA.
    pub fn len(&self) -> usize {
        self.agent_id.len()
    }

    /// Возвращает true, если агентов нет.
    pub fn is_empty(&self) -> bool {
        self.agent_id.is_empty()
    }

    /// Увеличивает размер SoA на заданное число агентов.
    pub fn extend(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }
        let start = self.agent_id.len();
        let new_len = start.saturating_add(additional);

        self.agent_id.reserve(additional);
        self.alive.reserve(additional);
        self.is_static.reserve(additional);
        self.type_id.reserve(additional);
        self.packet_seq.reserve(additional);
        self.pos_x.reserve(additional);
        self.pos_y.reserve(additional);
        self.vel_x.reserve(additional);
        self.vel_y.reserve(additional);
        self.target_x.reserve(additional);
        self.target_y.reserve(additional);
        self.self_speed.reserve(additional);
        self.energy.reserve(additional);
        self.memory_cap.reserve(additional);
        self.mem_used.reserve(additional);
        self.compute_power.reserve(additional);
        self.bandwidth.reserve(additional);
        self.packets_sent.reserve(additional);
        self.packets_recv.reserve(additional);
        self.packets_drop.reserve(additional);
        self.meta_packets_sent.reserve(additional);
        self.meta_packets_recv.reserve(additional);
        self.memory_id.reserve(additional);

        for id in start..new_len {
            self.agent_id.push(id as u32);
            self.alive.push(true);
            self.is_static.push(false);
            self.type_id.push(0);
            self.packet_seq.push(0);
            self.pos_x.push(0.0);
            self.pos_y.push(0.0);
            self.vel_x.push(0.0);
            self.vel_y.push(0.0);
            self.target_x.push(0.0);
            self.target_y.push(0.0);
            self.self_speed.push(0.0);
            self.energy.push(0.0);
            self.memory_cap.push(0);
            self.mem_used.push(0);
            self.compute_power.push(0.0);
            self.bandwidth.push(0.0);
            self.packets_sent.push(0);
            self.packets_recv.push(0);
            self.packets_drop.push(0);
            self.meta_packets_sent.push(0);
            self.meta_packets_recv.push(0);
            self.memory_id.push(MemoryId::default());
        }
    }
}
