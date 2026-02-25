use config::{Config, Environment, File};
use serde::Deserialize;

use netsim_core::{InitialEventSpec, PacketSpec};

/// Полный набор настроек CLI (включая окно и конфигурацию симуляции).
#[derive(Debug, Clone, Deserialize)]
pub struct SystemConfig {
    /// Настройки окна (модуль визуализации).
    pub window: netsim_screen::WindowConfig,
    /// Флаг включения отладочного режима.
    pub debug: bool,
    /// Настройки симуляции ядра.
    pub sim: SimConfigFile,
}

/// Событие, загружаемое из конфигурационного файла.
#[derive(Debug, Clone, Deserialize)]
pub struct InitialEventFile {
    /// Агент-получатель события.
    pub agent_id: u32,
    /// Локальный номер пакета для детерминизма.
    pub packet_seq: u32,
    /// Полезная нагрузка пакета.
    pub packet: PacketSpecFile,
}

/// Обязательные поля пакета из конфига.
#[derive(Debug, Clone, Deserialize)]
pub struct PacketSpecFile {
    /// Глобально уникальный идентификатор пакета.
    pub packet_id: u64,
    /// Идентификатор исходного агента.
    pub src_id: u32,
    /// Идентификатор целевого агента.
    pub dst_id: u32,
    /// Тик, в который пакет создан.
    pub created_tick: u64,
    /// Тик, в который пакет должен быть доставлен.
    pub deliver_tick: u64,
    /// TTL пакета.
    pub ttl: u16,
    /// Размер пакета в байтах.
    pub size_bytes: u32,
    /// Показатель качества/шума сигнала.
    pub quality: f32,
    /// Признак служебного пакета.
    pub meta: bool,
    /// Подсказка следующего хопа для будущей логики.
    pub route_hint: u32,
}

/// Снимок загруженной конфигурации для печати.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemConfigFile {
    /// Флаг включения отладочного режима.
    pub debug: bool,
    /// Настройки окна (модуль визуализации).
    pub window: netsim_screen::WindowConfig,
    /// Настройки симуляции ядра.
    pub sim: SimConfigFile,
}

/// Конфигурация симуляции, загружаемая через CLI.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SimConfigFile {
    /// Количество агентов.
    pub agents_count: u32,
    /// Число тиков для прогона симуляции.
    pub ticks: u64,
    /// Размер окна ring-buffer очереди событий.
    pub event_queue_window: u64,
    /// Начальные события, добавляемые до первого тика.
    #[serde(default)]
    pub initial_events: Vec<InitialEventFile>,
}

impl SystemConfig {
    /// Загружает конфигурацию из `netsim.toml` и переменных окружения.
    pub fn new() -> Result<Self, config::ConfigError> {
        let builder = Config::builder()
            .set_default("debug", false)?
            .set_default("window.width_px", 1280)?
            .set_default("window.height_px", 720)?
            .set_default("sim.agents_count", 0)?
            .set_default("sim.ticks", 0)?
            .set_default("sim.event_queue_window", 64)?
            .add_source(File::with_name("netsim.toml").required(false))
            .add_source(Environment::with_prefix("NETSIM").separator("__"));

        builder.build()?.try_deserialize()
    }

    /// Возвращает копию конфига для отображения в CLI.
    pub fn snapshot(&self) -> SystemConfigFile {
        SystemConfigFile {
            debug: self.debug,
            window: self.window,
            sim: self.sim.clone(),
        }
    }
}

impl InitialEventFile {
    /// Преобразует CLI-конфиг в типы ядра симуляции.
    #[allow(dead_code)]
    pub fn to_core(&self) -> InitialEventSpec {
        InitialEventSpec {
            agent_id: self.agent_id,
            packet_seq: self.packet_seq,
            packet: PacketSpec {
                packet_id: self.packet.packet_id,
                src_id: self.packet.src_id,
                dst_id: self.packet.dst_id,
                created_tick: self.packet.created_tick,
                deliver_tick: self.packet.deliver_tick,
                ttl: self.packet.ttl,
                size_bytes: self.packet.size_bytes,
                quality: self.packet.quality,
                meta: self.packet.meta,
                route_hint: self.packet.route_hint,
            },
        }
    }
}
