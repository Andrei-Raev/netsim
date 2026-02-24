use anyhow::Result;
use clap::Parser;
use tracing::info;

mod app_config;
mod argparser;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let _cli = argparser::Cli::parse();
    let cfg = app_config::SystemConfig::new()?;

    info!(
        "Netsim CLI: window={}x{}, debug={}",
        cfg.window.width_px, cfg.window.height_px, cfg.debug
    );

    let sim_config = netsim_core::SimConfig {
        agents_count: cfg.sim.agents_count,
        ticks: cfg.sim.ticks,
        event_queue_window: cfg.sim.event_queue_window,
        initial_events: cfg
            .sim
            .initial_events
            .iter()
            .map(|event| event.to_core())
            .collect(),
    };
    let mut pipeline = netsim_core::SimPipeline::from_config(sim_config.clone());
    let result = pipeline.run(sim_config);

    info!(
        "Симуляция завершена: ticks={}, sent={}, recv={}, drop={}",
        result.ticks_processed,
        result.stats.packets_sent,
        result.stats.packets_recv,
        result.stats.packets_drop
    );

    Ok(())
}
