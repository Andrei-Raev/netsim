use config::{Config, Environment, File};
use serde::Deserialize;

use netsim_core::{InitialEventSpec, PacketSpec};

#[derive(Debug, Clone, Deserialize)]
pub struct SystemConfig {
    pub window: netsim_screen::WindowConfig,
    pub debug: bool,
    pub sim: SimConfigFile,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InitialEventFile {
    pub agent_id: u32,
    pub packet_seq: u32,
    pub packet: PacketSpecFile,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PacketSpecFile {
    pub packet_id: u64,
    pub src_id: u32,
    pub dst_id: u32,
    pub created_tick: u64,
    pub deliver_tick: u64,
    pub ttl: u16,
    pub size_bytes: u32,
    pub quality: f32,
    pub meta: bool,
    pub route_hint: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemConfigFile {
    pub debug: bool,
    pub window: netsim_screen::WindowConfig,
    pub sim: SimConfigFile,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SimConfigFile {
    pub agents_count: u32,
    pub ticks: u64,
    pub event_queue_window: u64,
    #[serde(default)]
    pub initial_events: Vec<InitialEventFile>,
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
            .add_source(File::with_name("netsim.toml").required(false))
            .add_source(Environment::with_prefix("NETSIM").separator("__"));

        builder.build()?.try_deserialize()
    }

    pub fn snapshot(&self) -> SystemConfigFile {
        SystemConfigFile {
            debug: self.debug,
            window: self.window,
            sim: self.sim.clone(),
        }
    }
}

impl InitialEventFile {
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
