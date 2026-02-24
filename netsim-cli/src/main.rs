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

    let mut pipeline = netsim_core::SimPipeline::new(0);
    let result = pipeline.run(netsim_core::SimConfig { ticks: 0 });

    info!(
        "Симуляция завершена: ticks={}, sent={}, recv={}, drop={}",
        result.ticks_processed,
        result.stats.packets_sent,
        result.stats.packets_recv,
        result.stats.packets_drop
    );

    Ok(())
}
