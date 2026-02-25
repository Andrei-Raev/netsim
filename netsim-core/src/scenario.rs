use crate::WorldScene;
use crate::agent::AgentSpec;
use crate::world::{FieldSource, WorldConfig};

/// Конфигурация сценария симуляции (внешний файл *.scenario.toml).
#[derive(Debug, Clone)]
pub struct ScenarioConfig {
    /// Настройки мира.
    pub world: WorldConfig,
    /// Seed сценария.
    pub seed: u64,
    /// Количество тиков в прогоне.
    pub ticks: u64,
    /// Размер окна ring‑buffer очереди событий.
    pub event_queue_window: u64,
    /// Порог шума для дропа пакетов.
    pub noise_drop_threshold: f32,
    /// Описание сцены.
    pub scene: SceneSpec,
    /// Конфигурация начальных событий.
    pub initial_events: crate::InitialEventsConfig,
    /// Список событий сценария.
    pub events: Vec<ScenarioEventSpec>,
}

impl ScenarioConfig {
    /// Создает сцену мира из спецификации.
    pub fn build_scene(&self) -> WorldScene {
        match &self.scene {
            SceneSpec::Preset { name } => match name.as_str() {
                "minimal" => crate::world::scenes::minimal_scene(
                    self.world.width,
                    self.world.height,
                    self.world.cell_size,
                    self.seed,
                ),
                _ => crate::world::scenes::minimal_scene(
                    self.world.width,
                    self.world.height,
                    self.world.cell_size,
                    self.seed,
                ),
            },
            SceneSpec::Generated { sources } => {
                crate::world::scenes::generate_scene(self.world, self.seed, *sources)
            }
            SceneSpec::Manual { sources } => {
                WorldScene::new(self.world, sources.clone(), self.seed)
            }
        }
    }

    /// Готовит список событий для заданного тика.
    pub fn events_for_tick(&self, tick: u64) -> Vec<ScenarioEventSpec> {
        let mut events: Vec<ScenarioEventSpec> = self
            .events
            .iter()
            .filter(|event| event.is_due(tick))
            .cloned()
            .collect();

        let generated = self
            .initial_events
            .events_for_tick(tick)
            .into_iter()
            .map(ScenarioEventSpec::Traffic);
        events.extend(generated);

        events
    }
}

/// Описание сцены мира.
#[derive(Debug, Clone)]
pub enum SceneSpec {
    /// Готовый preset.
    Preset { name: String },
    /// Детерминированная генерация.
    Generated { sources: usize },
    /// Ручной список источников.
    Manual { sources: Vec<FieldSource> },
}

/// Событие сценария.
#[derive(Debug, Clone)]
pub enum ScenarioEventSpec {
    /// Спаун агентов.
    SpawnAgents(SpawnAgentsSpec),
    /// Трафик (команда агенту отправить пакет).
    Traffic(TrafficSpec),
    /// Трафик в области (каждый агент в зоне получает команду на отправку).
    TrafficArea(TrafficAreaSpec),
}

impl ScenarioEventSpec {
    /// Проверяет, срабатывает ли событие на тике.
    pub fn is_due(&self, tick: u64) -> bool {
        match self {
            ScenarioEventSpec::SpawnAgents(spec) => spec.tick == tick,
            ScenarioEventSpec::Traffic(spec) => spec.is_due(tick),
            ScenarioEventSpec::TrafficArea(spec) => spec.is_due(tick),
        }
    }
}

/// Типы спауна агентов.
#[derive(Debug, Clone)]
pub enum SpawnShape {
    /// Сетка фиксированных размеров.
    Grid {
        rows: u32,
        cols: u32,
        spacing: f32,
        origin_x: f32,
        origin_y: f32,
    },
    /// Распределение по окружности.
    Circle {
        center_x: f32,
        center_y: f32,
        radius: f32,
    },
}

/// Спецификация события спауна агентов.
#[derive(Debug, Clone)]
pub struct SpawnAgentsSpec {
    /// Тик срабатывания события.
    pub tick: u64,
    /// Первый агент в диапазоне ID.
    pub agent_id_start: u32,
    /// Количество агентов.
    pub count: u32,
    /// Полное описание агента.
    pub agent_spec: AgentSpec,
    /// Геометрия спауна.
    pub shape: SpawnShape,
}

impl SpawnAgentsSpec {
    /// Строит AgentSpec для конкретного индекса спауна.
    pub fn spec_for_index(&self, agent_id: u32) -> AgentSpec {
        AgentSpec {
            agent_id,
            ..self.agent_spec
        }
    }
}

/// Спецификация события трафика.
#[derive(Debug, Clone, PartialEq)]
pub struct TrafficSpec {
    /// Тик срабатывания события.
    pub tick: u64,
    /// Идентификатор пакета.
    pub packet_id: u64,
    /// Агент-источник.
    pub src_id: u32,
    /// Агент-получатель.
    pub dst_id: u32,
    /// TTL пакета.
    pub ttl: u16,
    /// Размер пакета в байтах.
    pub size_bytes: u32,
    /// Показатель качества/шума сигнала.
    pub quality: f32,
    /// Признак служебного пакета.
    pub meta: bool,
    /// Идентификатор конечного адресата.
    pub trg_id: u32,
    /// Подсказка следующего хопа.
    pub route_hint: u32,
    /// Период повторения (0 = одноразово).
    pub repeat_every: u64,
}

impl TrafficSpec {
    /// Проверяет, срабатывает ли событие на текущем тике.
    pub fn is_due(&self, tick: u64) -> bool {
        if self.repeat_every == 0 {
            return tick == self.tick;
        }
        tick >= self.tick && (tick - self.tick).is_multiple_of(self.repeat_every)
    }
}

/// Спецификация трафика в области.
#[derive(Debug, Clone)]
pub struct TrafficAreaSpec {
    /// Тик первого срабатывания.
    pub tick: u64,
    /// Период повторения (0 = одноразово).
    pub repeat_every: u64,
    /// Геометрия области.
    pub area: TrafficAreaShape,
    /// Шаблон пакета для генерации трафика.
    pub template: TrafficTemplateSpec,
    /// Параметры выбора адресата.
    pub target: TrafficTargetSpec,
}

impl TrafficAreaSpec {
    /// Проверяет, срабатывает ли событие на текущем тике.
    pub fn is_due(&self, tick: u64) -> bool {
        if self.repeat_every == 0 {
            return tick == self.tick;
        }
        tick >= self.tick && (tick - self.tick).is_multiple_of(self.repeat_every)
    }
}

/// Геометрия области трафика.
#[derive(Debug, Clone)]
pub enum TrafficAreaShape {
    /// Сетка вокруг центра по индексам ячеек.
    Grid {
        min: (usize, usize),
        max: (usize, usize),
    },
    /// Окружность в координатах мира.
    Circle {
        center_x: f32,
        center_y: f32,
        radius: f32,
    },
    /// Прямоугольник в координатах мира.
    Rect {
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
    },
}

/// Шаблон пакета для трафика области.
#[derive(Debug, Clone)]
pub struct TrafficTemplateSpec {
    /// Базовый идентификатор пакета (инкрементируется).
    pub packet_id_base: u64,
    /// TTL пакета.
    pub ttl: u16,
    /// Размер пакета в байтах.
    pub size_bytes: u32,
    /// Показатель качества/шума сигнала.
    pub quality: f32,
    /// Признак служебного пакета.
    pub meta: bool,
    /// Идентификатор конечного адресата.
    pub trg_id: u32,
    /// Подсказка следующего хопа.
    pub route_hint: u32,
}

/// Варианты выбора целевого агента.
#[derive(Debug, Clone)]
pub enum TrafficTargetSpec {
    /// Отправка на конкретный агент.
    Fixed { dst_id: u32 },
    /// Отправка на самого себя.
    SelfTarget,
}
