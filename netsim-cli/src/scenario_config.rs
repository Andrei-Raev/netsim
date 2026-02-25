use std::fs;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use netsim_core::{
    FieldShape, FieldSource, InfluenceType, ScenarioConfig, ScenarioEventSpec, SceneSpec,
    SpawnAgentsSpec, SpawnShape, TimeProfile, TrafficSpec, Vec2, WorldBase, WorldConfig,
    WorldFieldType,
};

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioFile {
    pub world: WorldConfigFile,
    pub simulation: SimulationFile,
    pub scene: SceneFile,
    #[serde(default)]
    pub events: Vec<EventFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorldConfigFile {
    pub seed: u64,
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SimulationFile {
    pub ticks: u64,
    pub event_queue_window: u64,
    #[serde(default)]
    pub noise_drop_threshold: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SceneFile {
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub generate: Option<GenerateSceneFile>,
    #[serde(default)]
    pub sources: Vec<SourceFile>,
}

impl SceneFile {
    pub fn validate(&self) -> Result<()> {
        let mut choices = 0;
        if self.preset.is_some() {
            choices += 1;
        }
        if self.generate.is_some() {
            choices += 1;
        }
        if !self.sources.is_empty() {
            choices += 1;
        }

        if choices == 0 {
            return Err(anyhow!("Не задано описание сцены: preset/generate/sources"));
        }
        if choices > 1 {
            return Err(anyhow!(
                "Описание сцены должно быть только одного типа: preset/generate/sources"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateSceneFile {
    pub sources: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventFile {
    SpawnAgents(SpawnAgentsFile),
    Traffic(TrafficFile),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpawnAgentsFile {
    pub tick: u64,
    pub agent_id_start: u32,
    pub count: u32,
    pub agent: AgentSpecFile,
    pub shape: SpawnShapeFile,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpawnShapeFile {
    Grid {
        rows: u32,
        cols: u32,
        spacing: f32,
        origin: [f32; 2],
    },
    Circle {
        center: [f32; 2],
        radius: f32,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentSpecFile {
    pub type_id: u16,
    pub routing_cap: u32,
    pub scratch_cap: u32,
    pub compute_power: f32,
    pub bandwidth: f32,
    pub self_speed: f32,
    pub memory_cap: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrafficFile {
    pub tick: u64,
    pub packet_id: u64,
    pub src_id: u32,
    pub dst_id: u32,
    pub ttl: u16,
    pub size_bytes: u32,
    pub quality: f32,
    pub meta: bool,
    pub route_hint: u32,
    #[serde(default)]
    pub repeat_every: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourceFile {
    pub id: u64,
    pub field_type: String,
    pub strength: f32,
    pub shape: ShapeFile,
    pub influence: InfluenceFile,
    pub time: TimeProfileFile,
    pub active: ActiveWindowFile,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShapeFile {
    Circle {
        center: [f32; 2],
        radius: f32,
    },
    Rect {
        center: [f32; 2],
        half_extents: [f32; 2],
    },
    Line {
        from: [f32; 2],
        to: [f32; 2],
        width: f32,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InfluenceFile {
    Hard,
    Linear,
    Gaussian,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TimeProfileFile {
    Static,
    Pulse {
        period_ticks: u64,
        duty: f32,
    },
    Wave {
        period_ticks: u64,
        amplitude: f32,
        phase: f32,
    },
    Curve {
        points: Vec<(u64, f32)>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActiveWindowFile {
    pub start: u64,
    pub end: u64,
}

impl ScenarioFile {
    pub fn to_core(&self) -> ScenarioConfig {
        let world = WorldConfig {
            width: self.world.width,
            height: self.world.height,
            cell_size: self.world.cell_size,
            base: WorldBase {
                load: 0.0,
                noise: 0.0,
                bandwidth: 0.0,
                cost: 0.0,
            },
        };

        let scene = if let Some(preset) = &self.scene.preset {
            SceneSpec::Preset {
                name: preset.clone(),
            }
        } else if let Some(generate) = &self.scene.generate {
            SceneSpec::Generated {
                sources: generate.sources,
            }
        } else {
            let sources = self
                .scene
                .sources
                .iter()
                .map(|source| source.to_core())
                .collect();
            SceneSpec::Manual { sources }
        };

        ScenarioConfig {
            world,
            seed: self.world.seed,
            ticks: self.simulation.ticks,
            event_queue_window: self.simulation.event_queue_window,
            noise_drop_threshold: self.simulation.noise_drop_threshold,
            scene,
            events: self.events.iter().map(|event| event.to_core()).collect(),
        }
    }
}

impl EventFile {
    pub fn to_core(&self) -> ScenarioEventSpec {
        match self {
            EventFile::SpawnAgents(spec) => ScenarioEventSpec::SpawnAgents(spec.to_core()),
            EventFile::Traffic(spec) => ScenarioEventSpec::Traffic(spec.to_core()),
        }
    }
}

impl SpawnAgentsFile {
    pub fn to_core(&self) -> SpawnAgentsSpec {
        let agent_spec = netsim_core::AgentSpec {
            agent_id: self.agent_id_start,
            type_id: self.agent.type_id,
            routing_cap: self.agent.routing_cap,
            scratch_cap: self.agent.scratch_cap,
            compute_power: self.agent.compute_power,
            bandwidth: self.agent.bandwidth,
            self_speed: self.agent.self_speed,
            memory_cap: self.agent.memory_cap,
        };

        SpawnAgentsSpec {
            tick: self.tick,
            agent_id_start: self.agent_id_start,
            count: self.count,
            agent_spec,
            shape: self.shape.to_core(),
        }
    }
}

impl SpawnShapeFile {
    pub fn to_core(&self) -> SpawnShape {
        match self {
            SpawnShapeFile::Grid {
                rows,
                cols,
                spacing,
                origin,
            } => SpawnShape::Grid {
                rows: *rows,
                cols: *cols,
                spacing: *spacing,
                origin_x: origin[0],
                origin_y: origin[1],
            },
            SpawnShapeFile::Circle { center, radius } => SpawnShape::Circle {
                center_x: center[0],
                center_y: center[1],
                radius: *radius,
            },
        }
    }
}

impl TrafficFile {
    pub fn to_core(&self) -> TrafficSpec {
        TrafficSpec {
            tick: self.tick,
            packet_id: self.packet_id,
            src_id: self.src_id,
            dst_id: self.dst_id,
            ttl: self.ttl,
            size_bytes: self.size_bytes,
            quality: self.quality,
            meta: self.meta,
            route_hint: self.route_hint,
            repeat_every: self.repeat_every,
        }
    }
}

impl SourceFile {
    pub fn to_core(&self) -> FieldSource {
        FieldSource {
            id: self.id,
            field_type: parse_field_type(&self.field_type),
            shape: self.shape.to_core(),
            influence: self.influence.to_core(),
            strength: self.strength,
            time_profile: self.time.to_core(),
            active_window: self.active.to_core(),
        }
    }
}

impl ShapeFile {
    pub fn to_core(&self) -> FieldShape {
        match self {
            ShapeFile::Circle { center, radius } => FieldShape::Circle {
                center: Vec2::new(center[0], center[1]),
                radius: *radius,
            },
            ShapeFile::Rect {
                center,
                half_extents,
            } => FieldShape::Rect {
                center: Vec2::new(center[0], center[1]),
                half_extents: Vec2::new(half_extents[0], half_extents[1]),
            },
            ShapeFile::Line { from, to, width } => FieldShape::Line {
                from: Vec2::new(from[0], from[1]),
                to: Vec2::new(to[0], to[1]),
                width: *width,
            },
        }
    }
}

impl InfluenceFile {
    pub fn to_core(&self) -> InfluenceType {
        match self {
            InfluenceFile::Hard => InfluenceType::Hard,
            InfluenceFile::Linear => InfluenceType::Linear,
            InfluenceFile::Gaussian => InfluenceType::Gaussian,
        }
    }
}

impl TimeProfileFile {
    pub fn to_core(&self) -> TimeProfile {
        match self {
            TimeProfileFile::Static => TimeProfile::Static,
            TimeProfileFile::Pulse { period_ticks, duty } => TimeProfile::Pulse {
                period_ticks: *period_ticks,
                duty: *duty,
            },
            TimeProfileFile::Wave {
                period_ticks,
                amplitude,
                phase,
            } => TimeProfile::Wave {
                period_ticks: *period_ticks,
                amplitude: *amplitude,
                phase: *phase,
            },
            TimeProfileFile::Curve { points } => TimeProfile::Curve {
                points: points.clone(),
            },
        }
    }
}

impl ActiveWindowFile {
    pub fn to_core(&self) -> netsim_core::ActiveWindow {
        netsim_core::ActiveWindow {
            start: self.start,
            end: self.end,
        }
    }
}

fn parse_field_type(value: &str) -> WorldFieldType {
    match value {
        "load" => WorldFieldType::Load,
        "noise" => WorldFieldType::Noise,
        "bandwidth" => WorldFieldType::Bandwidth,
        "cost" => WorldFieldType::Cost,
        _ => WorldFieldType::Noise,
    }
}

pub fn load_scenario(path: &str) -> Result<ScenarioConfig> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("Не удалось прочитать {path}"))?;
    let parsed: ScenarioFile = toml::from_str(&contents)
        .map_err(|error| anyhow!("Не удалось разобрать сценарий {path}: {error}"))?;
    parsed.scene.validate()?;
    Ok(parsed.to_core())
}
