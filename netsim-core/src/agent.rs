use crate::memory::MemoryId;

/// Хранилище агентов в формате Structure-of-Arrays.
#[derive(Debug, Clone)]
pub struct AgentSoA {
    /// Стабильные идентификаторы агентов.
    pub agent_id: Vec<u32>,
    /// Флаг активности агента.
    pub alive: Vec<bool>,
    /// Признак статичности (не двигается).
    pub is_static: Vec<bool>,
    /// Счетчик пакетов на агента для детерминизма.
    pub packet_seq: Vec<u32>,
    /// Позиция по X.
    pub pos_x: Vec<f32>,
    /// Позиция по Y.
    pub pos_y: Vec<f32>,
    /// Целевая позиция по X.
    pub target_x: Vec<f32>,
    /// Целевая позиция по Y.
    pub target_y: Vec<f32>,
    /// Текущий уровень энергии.
    pub energy: Vec<f32>,
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
            packet_seq: vec![0; count],
            pos_x: vec![0.0; count],
            pos_y: vec![0.0; count],
            target_x: vec![0.0; count],
            target_y: vec![0.0; count],
            energy: vec![0.0; count],
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

    /// Returns the number of agents stored in the SoA.
    pub fn len(&self) -> usize {
        self.agent_id.len()
    }

    /// Returns true if there are no agents in the SoA.
    pub fn is_empty(&self) -> bool {
        self.agent_id.is_empty()
    }
}
