use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SystemConfig {
    pub window: netsim_screen::WindowConfig,
    pub debug: bool,
    pub sim: SimConfigFile,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SimConfigFile {
    pub agents_count: u32,
    pub ticks: u64,
    pub event_queue_window: u64,
}

impl SystemConfig {
    pub fn new() -> Result<Self, config::ConfigError> {
        let builder = Config::builder()
            .set_default("debug", false)?
            .set_default("window.width_px", 1280)?
            .set_default("window.height_px", 720)?
            .set_default("sim.agents_count", 0)?
            .set_default("sim.ticks", 0)?
            .set_default("sim.event_queue_window", 64)?
            .add_source(File::with_name("test_cfg").required(false))
            .add_source(Environment::with_prefix("NETSIM").separator("__"));

        builder.build()?.try_deserialize()
    }
}
