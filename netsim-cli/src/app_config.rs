use netsim_screen;
use config::{Config, File, Enviroment};

#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub window: netsim_screen::WindowConfig,
    pub debuf: bool
}

impl SystemConfig{
    pub fn new() -> Result<Self, config::ConfigError> {
        let builder = Config::builder()
        .set_default("debug", false)?
        .add_source(File::with_name("test_cfg").required(false));

    builder.build()?.try_deserealize()
    }
}