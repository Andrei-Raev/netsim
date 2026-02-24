use crate::{AgentSoA, SimConfig, SimStats};

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
}

impl SimPipeline {
    /// Создает новый пайплайн с заданным количеством агентов.
    pub fn new(agent_count: usize) -> Self {
        Self {
            agents: AgentSoA::new(agent_count),
            stats: SimStats::default(),
            current_tick: 0,
        }
    }

    /// Создает пайплайн по конфигу ядра.
    pub fn from_config(config: SimConfig) -> Self {
        Self::new(config.agents_count as usize)
    }

    /// Выполняет один тик симуляции.
    pub fn step(&mut self) {
        self.current_tick += 1;
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
}
