use anyhow::Result;
use clap::Parser;
use tabled::{Table, Tabled};
use tracing::info;

mod app_config;
mod argparser;
mod scenario_config;

/// Строка таблицы для печати загруженной конфигурации.
#[derive(Debug, Tabled)]
struct ConfigRow {
    /// Имя параметра из конфигурации.
    #[tabled(rename = "Параметр")]
    key: String,
    /// Значение параметра в строковом виде.
    #[tabled(rename = "Значение")]
    value: String,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = argparser::Cli::parse();
    let cfg = app_config::SystemConfig::new()?;
    let cfg_snapshot = cfg.snapshot();

    let scenario_path = cli
        .scenario
        .as_deref()
        .unwrap_or("scenarios/default.scenario.toml");
    let scenario = scenario_config::load_scenario(scenario_path)?;

    info!(
        "Netsim CLI: window={}x{}, debug={}",
        cfg.window.width_px, cfg.window.height_px, cfg.debug
    );

    print_config(&cfg_snapshot);

    let visualizer = netsim_screen::SimVisualizer::new(
        netsim_screen::WindowConfig {
            width_px: cfg.window.width_px,
            height_px: cfg.window.height_px,
        },
        scenario,
    )?;

    let result = visualizer.run()?;

    info!(
        "Симуляция завершена: ticks={}, sent={}, recv={}, drop={}",
        result.ticks_processed,
        result.stats.packets_sent,
        result.stats.packets_recv,
        result.stats.packets_drop
    );

    Ok(())
}

/// Печатает загруженный конфиг в виде таблицы.
fn print_config(cfg: &app_config::SystemConfigFile) {
    let mut rows = vec![
        ConfigRow {
            key: "debug".to_string(),
            value: cfg.debug.to_string(),
        },
        ConfigRow {
            key: "window.width_px".to_string(),
            value: cfg.window.width_px.to_string(),
        },
        ConfigRow {
            key: "window.height_px".to_string(),
            value: cfg.window.height_px.to_string(),
        },
        ConfigRow {
            key: "sim.agents_count".to_string(),
            value: cfg.sim.agents_count.to_string(),
        },
        ConfigRow {
            key: "sim.ticks".to_string(),
            value: cfg.sim.ticks.to_string(),
        },
        ConfigRow {
            key: "sim.event_queue_window".to_string(),
            value: cfg.sim.event_queue_window.to_string(),
        },
        ConfigRow {
            key: "sim.initial_events".to_string(),
            value: cfg.sim.initial_events.len().to_string(),
        },
    ];

    for (index, event) in cfg.sim.initial_events.iter().enumerate() {
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].agent_id"),
            value: event.agent_id.to_string(),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].packet_seq"),
            value: event.packet_seq.to_string(),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].packet.packet_id"),
            value: event.packet.packet_id.to_string(),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].packet.src_id"),
            value: event.packet.src_id.to_string(),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_errors[{index}].packet.dst_id"),
            value: event.packet.dst_id.to_string(),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].packet.created_tick"),
            value: event.packet.created_tick.to_string(),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].packet.deliver_tick"),
            value: event.packet.deliver_tick.to_string(),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].packet.ttl"),
            value: event.packet.ttl.to_string(),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].packet.size_bytes"),
            value: event.packet.size_bytes.to_string(),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].packet.quality"),
            value: format!("{:.3}", event.packet.quality),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].packet.meta"),
            value: event.packet.meta.to_string(),
        });
        rows.push(ConfigRow {
            key: format!("sim.initial_events[{index}].packet.route_hint"),
            value: event.packet.route_hint.to_string(),
        });
    }

    let mut table = Table::new(rows);
    table.with(tabled::settings::Style::rounded());
    println!("{table}");
}
