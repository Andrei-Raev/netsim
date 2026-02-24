# Конфигурации Netsim

Этот документ описывает формат конфигов, используемых CLI и ядром (core). CLI отвечает за загрузку, ядро получает готовые структуры.

## Общий формат

Конфиги читаются через `config` crate в `netsim-cli`:
- файл: `netsim-cli/src/app_config.rs`
- точка входа: `SystemConfig::new()`
- источники: `test_cfg` (необязательный), переменные окружения с префиксом `NETSIM__`

Схема:

- `debug` — глобальный флаг отладочного режима.
- `window.*` — настройки окна (см. `netsim-screen`).
- `sim.*` — настройки симуляции ядра.

## Поля

### debug
- **Путь:** `debug`
- **Тип:** `bool`
- **Где задано:** `netsim-cli/src/app_config.rs::SystemConfig`
- **Описание:** включает/выключает отладочный режим для CLI.

### window.width_px
- **Путь:** `window.width_px`
- **Тип:** `u32`
- **Где задано:** `netsim-screen/src/config.rs::WindowConfig`
- **Описание:** ширина окна визуализации в пикселях.

### window.height_px
- **Путь:** `window.height_px`
- **Тип:** `u32`
- **Где задано:** `netsim-screen/src/config.rs::WindowConfig`
- **Описание:** высота окна визуализации в пикселях.

### sim.agents_count
- **Путь:** `sim.agents_count`
- **Тип:** `u32`
- **Где задано:** `netsim-cli/src/app_config.rs::SimConfigFile`
- **Описание:** количество агентов в симуляции.

### sim.ticks
- **Путь:** `sim.ticks`
- **Тип:** `u64`
- **Где задано:** `netsim-cli/src/app_config.rs::SimConfigFile`
- **Описание:** число тиков, которое выполняет симуляция.

### sim.event_queue_window
- **Путь:** `sim.event_queue_window`
- **Тип:** `u64`
- **Где задано:** `netsim-cli/src/app_config.rs::SimConfigFile`
- **Описание:** размер окна ring‑buffer очереди событий.

### sim.initial_events
- **Путь:** `sim.initial_events`
- **Тип:** `array<InitialEventFile>`
- **Где задано:** `netsim-cli/src/app_config.rs::InitialEventFile`
- **Описание:** список событий, которые добавляются в очередь до первого тика.

#### sim.initial_events[].agent_id
- **Путь:** `sim.initial_events[].agent_id`
- **Тип:** `u32`
- **Где задано:** `netsim-cli/src/app_config.rs::InitialEventFile`
- **Описание:** агент‑получатель события.

#### sim.initial_events[].packet_seq
- **Путь:** `sim.initial_events[].packet_seq`
- **Тип:** `u32`
- **Где задано:** `netsim-cli/src/app_config.rs::InitialEventFile`
- **Описание:** локальный счетчик пакетов агента для детерминизма.

#### sim.initial_events[].packet
- **Путь:** `sim.initial_events[].packet`
- **Тип:** `PacketSpecFile`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** обязательный набор полей пакета для детерминированной сборки.

##### sim.initial_events[].packet.packet_id
- **Путь:** `sim.initial_events[].packet.packet_id`
- **Тип:** `u64`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** глобально уникальный идентификатор пакета.

##### sim.initial_events[].packet.src_id
- **Путь:** `sim.initial_events[].packet.src_id`
- **Тип:** `u32`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** идентификатор исходного агента.

##### sim.initial_events[].packet.dst_id
- **Путь:** `sim.initial_events[].packet.dst_id`
- **Тип:** `u32`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** идентификатор целевого агента.

##### sim.initial_events[].packet.created_tick
- **Путь:** `sim.initial_events[].packet.created_tick`
- **Тип:** `u64`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** тик, в который пакет создан.

##### sim.initial_events[].packet.deliver_tick
- **Путь:** `sim.initial_events[].packet.deliver_tick`
- **Тип:** `u64`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** тик, в который пакет должен быть доставлен.

##### sim.initial_events[].packet.ttl
- **Путь:** `sim.initial_events[].packet.ttl`
- **Тип:** `u16`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** TTL пакета.

##### sim.initial_events[].packet.size_bytes
- **Путь:** `sim.initial_events[].packet.size_bytes`
- **Тип:** `u32`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** размер пакета в байтах.

##### sim.initial_events[].packet.quality
- **Путь:** `sim.initial_events[].packet.quality`
- **Тип:** `f32`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** показатель качества/шума сигнала.

##### sim.initial_events[].packet.meta
- **Путь:** `sim.initial_events[].packet.meta`
- **Тип:** `bool`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** признак служебного пакета.

##### sim.initial_events[].packet.route_hint
- **Путь:** `sim.initial_events[].packet.route_hint`
- **Тип:** `u32`
- **Где задано:** `netsim-cli/src/app_config.rs::PacketSpecFile`
- **Описание:** подсказка следующего хопа для будущей логики.

## Связь с ядром

CLI переводит `InitialEventFile` → `InitialEventSpec` и `PacketSpecFile` → `PacketSpec` (ядро) в `netsim-cli/src/app_config.rs::InitialEventFile::to_core()`.

Ядро принимает итоговую структуру `SimConfig` в `netsim-core/src/config.rs` и использует её в `SimPipeline::from_config` для наполнения очереди событий.
