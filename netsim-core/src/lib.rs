pub mod agent;
pub mod agent_runtime;
pub mod config;
pub mod event;
pub mod event_queue;
pub mod initial_events;
pub mod memory;
pub mod packet;
pub mod process_send;
pub mod scenario;
pub mod sim;
pub mod statistics;
pub mod stats;
pub mod world;

pub use agent::{AgentBuilder, AgentSoA, AgentSpec};
pub use agent_runtime::{
    AgentAlgorithm, AgentMemory, AgentRuntime, AllowAllValidator, BasicRoutingAlgorithm,
    EventValidator, RoutingTable,
};
pub use config::{InitialEventSpec, SimConfig};
pub use event::{Event, EventKind};
pub use event_queue::{EventQueue, EventQueueConfig};
pub use initial_events::{InitialEventRule, InitialEventsConfig};
pub use memory::{
    AGENT_MEMORY_MAGIC, AGENT_MEMORY_VERSION, AgentDescriptor, AgentMemoryArena, AgentMemoryBlock,
    AgentMemoryBlockMut, AgentMemoryBuilder, AgentMemoryLayout, AgentMemorySpec, MemoryId,
    ROUTE_FLAG_VALID, RouteEntry, RoutingTableError,
};
pub use packet::{Packet, PacketSpec};
pub use process_send::ProcessSend;
pub use scenario::{
    ScenarioConfig, ScenarioEventSpec, SceneSpec, SpawnAgentsSpec, SpawnShape, TrafficAreaShape,
    TrafficAreaSpec, TrafficSpec, TrafficTargetSpec, TrafficTemplateSpec,
};
pub use sim::{SimPipeline, SimResult};
pub use statistics::{StatisticsCollector, StatsSample};
pub use stats::SimStats;
pub use world::agents_grid::AgentHashGrid;
pub use world::scenes::{WorldScene, generate_scene, minimal_scene};
pub use world::{
    ActiveWindow, FieldShape, FieldSource, InfluenceType, TimeProfile, Vec2, WorldBase, WorldCell,
    WorldConfig, WorldFieldType, WorldGrid, WorldGridGenerator,
};
