use crate::memory::AgentStats;
use crate::{AgentMemory, AgentMemoryArena, Event, MemoryId, WorldCell};

/// Контейнер для локальной выборки метрик.
#[derive(Debug, Clone, Copy, Default)]
pub struct StatsSample {
    /// Количество отправленных пакетов.
    pub sent_count: u64,
    /// Количество полученных пакетов.
    pub recv_count: u64,
    /// Количество дропов.
    pub drop_count: u64,
    /// Суммарная стоимость.
    pub cost_accum: f32,
    /// Суммарная пропускная нагрузка.
    pub bandwidth_accum: f32,
    /// Суммарная нагрузка по миру.
    pub load_accum: f32,
    /// Сумма качества сигналов.
    pub quality_sum: f32,
    /// Количество сэмплов качества.
    pub quality_samples: u32,
}

impl StatsSample {
    /// Преобразует в сохранённую структуру дескриптора.
    pub fn into_descriptor(self) -> AgentStats {
        AgentStats {
            sent_count: self.sent_count,
            recv_count: self.recv_count,
            drop_count: self.drop_count,
            cost_accum: self.cost_accum,
            bandwidth_accum: self.bandwidth_accum,
            load_accum: self.load_accum,
            quality_sum: self.quality_sum,
            quality_samples: self.quality_samples,
            _pad: 0,
        }
    }
}

/// Коллектор метрик, который пишет данные в дескрипторы,
/// потому что держать всё в общей памяти дороже и сложнее.
#[derive(Debug, Default)]
pub struct StatisticsCollector;

impl StatisticsCollector {
    /// Учитывает входящее событие у агента.
    pub fn on_receive(&self, memory: &mut AgentMemory, quality: f32) {
        let mut stats = self.read_stats(memory);
        stats.recv_count = stats.recv_count.saturating_add(1);
        stats.quality_sum += quality;
        stats.quality_samples = stats.quality_samples.saturating_add(1);
        self.write_stats(memory, stats);
    }

    /// Учитывает исходящее событие у агента.
    pub fn on_send(&self, memory: &mut AgentMemory, quality: f32) {
        let mut stats = self.read_stats(memory);
        stats.sent_count = stats.sent_count.saturating_add(1);
        stats.quality_sum += quality;
        stats.quality_samples = stats.quality_samples.saturating_add(1);
        self.write_stats(memory, stats);
    }

    /// Учитывает дроп события у агента.
    pub fn on_drop(&self, memory: &mut AgentMemory) {
        let mut stats = self.read_stats(memory);
        stats.drop_count = stats.drop_count.saturating_add(1);
        self.write_stats(memory, stats);
    }

    /// Учитывает влияние ячейки мира.
    pub fn on_world_cell(&self, memory: &mut AgentMemory, cell: Option<&WorldCell>) {
        let Some(cell) = cell else {
            return;
        };
        let mut stats = self.read_stats(memory);
        stats.cost_accum += cell.cost;
        stats.bandwidth_accum += cell.bandwidth;
        stats.load_accum += cell.load;
        self.write_stats(memory, stats);
    }

    /// Пишет накопленные метрики в дескриптор (раз в collect_every).
    pub fn flush_agent(&self, arena: &mut AgentMemoryArena, id: MemoryId, sample: StatsSample) {
        let mut memory = AgentMemory::new(arena, id);
        memory.block.write_stats(sample.into_descriptor());
    }

    fn read_stats(&self, memory: &mut AgentMemory) -> StatsSample {
        let stats = memory.block.stats();
        StatsSample {
            sent_count: stats.sent_count,
            recv_count: stats.recv_count,
            drop_count: stats.drop_count,
            cost_accum: stats.cost_accum,
            bandwidth_accum: stats.bandwidth_accum,
            load_accum: stats.load_accum,
            quality_sum: stats.quality_sum,
            quality_samples: stats.quality_samples,
        }
    }

    fn write_stats(&self, memory: &mut AgentMemory, sample: StatsSample) {
        memory.block.write_stats(sample.into_descriptor());
    }
}

/// Рассчитывает, нужно ли собирать метрики для агента.
pub fn should_collect(collect_every: u64, tick: u64) -> bool {
    collect_every > 0 && tick.is_multiple_of(collect_every)
}

/// Учитывает событие как полученное, потому что оно обработано агентом.
pub fn apply_receive(event: &Event, memory: &mut AgentMemory, collector: &StatisticsCollector) {
    collector.on_receive(memory, event.payload.quality);
}

impl From<AgentStats> for StatsSample {
    fn from(stats: AgentStats) -> Self {
        Self {
            sent_count: stats.sent_count,
            recv_count: stats.recv_count,
            drop_count: stats.drop_count,
            cost_accum: stats.cost_accum,
            bandwidth_accum: stats.bandwidth_accum,
            load_accum: stats.load_accum,
            quality_sum: stats.quality_sum,
            quality_samples: stats.quality_samples,
        }
    }
}
