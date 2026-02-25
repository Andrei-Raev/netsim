use netsim_core::world::cpu::CpuWorldGenerator;
use netsim_core::{
    ScenarioConfig, ScenarioEventSpec, SceneSpec, SpawnAgentsSpec, SpawnShape, TrafficAreaShape,
    TrafficAreaSpec, TrafficTargetSpec, TrafficTemplateSpec,
};

#[test]
fn traffic_area_enqueues_packets_for_agents_in_rect() {
    let scenario = ScenarioConfig {
        world: netsim_core::WorldConfig {
            width: 4,
            height: 4,
            cell_size: 1.0,
            base: netsim_core::WorldBase {
                load: 0.0,
                noise: 0.0,
                bandwidth: 0.0,
                cost: 0.0,
            },
        },
        seed: 1,
        ticks: 1,
        event_queue_window: 8,
        noise_drop_threshold: 0.0,
        scene: SceneSpec::Preset {
            name: "minimal".to_string(),
        },
        initial_events: netsim_core::InitialEventsConfig::default(),
        events: vec![
            ScenarioEventSpec::SpawnAgents(SpawnAgentsSpec {
                tick: 0,
                agent_id_start: 0,
                count: 4,
                agent_spec: {
                    let mut spec = netsim_core::AgentSpec::placeholder(0);
                    spec.collect_every = 2;
                    spec
                },
                shape: SpawnShape::Grid {
                    rows: 2,
                    cols: 2,
                    spacing: 1.0,
                    origin_x: 0.0,
                    origin_y: 0.0,
                },
            }),
            ScenarioEventSpec::TrafficArea(TrafficAreaSpec {
                tick: 0,
                repeat_every: 0,
                area: TrafficAreaShape::Rect {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 1.5,
                    max_y: 1.5,
                },
                template: TrafficTemplateSpec {
                    packet_id_base: 100,
                    ttl: 2,
                    size_bytes: 16,
                    quality: 1.0,
                    meta: false,
                    trg_id: 0,
                    route_hint: 0,
                },
                target: TrafficTargetSpec::SelfTarget,
            }),
        ],
    };

    let mut pipeline = netsim_core::SimPipeline::from_scenario(&scenario);
    let scene = scenario.build_scene();
    let generator = CpuWorldGenerator::new(scene.config, scene.sources, scene.seed);

    pipeline.step_with_scenario(&scenario, &generator);

    let id = pipeline.agents.memory_id[0];
    let memory = netsim_core::AgentMemory::new(&mut pipeline.memory_arena, id);
    assert_eq!(memory.block.descriptor().collect_every, 2);

    assert_eq!(pipeline.stats.packets_sent, 0);
    assert!(pipeline.stats.packets_recv >= 1);
}

#[test]
fn traffic_area_ignores_agents_outside_circle() {
    let scenario = ScenarioConfig {
        world: netsim_core::WorldConfig {
            width: 4,
            height: 4,
            cell_size: 1.0,
            base: netsim_core::WorldBase {
                load: 0.0,
                noise: 0.0,
                bandwidth: 0.0,
                cost: 0.0,
            },
        },
        seed: 1,
        ticks: 1,
        event_queue_window: 8,
        noise_drop_threshold: 0.0,
        scene: SceneSpec::Preset {
            name: "minimal".to_string(),
        },
        initial_events: netsim_core::InitialEventsConfig::default(),
        events: vec![
            ScenarioEventSpec::SpawnAgents(SpawnAgentsSpec {
                tick: 0,
                agent_id_start: 0,
                count: 2,
                agent_spec: {
                    let mut spec = netsim_core::AgentSpec::placeholder(0);
                    spec.collect_every = 2;
                    spec
                },
                shape: SpawnShape::Grid {
                    rows: 1,
                    cols: 2,
                    spacing: 5.0,
                    origin_x: 0.0,
                    origin_y: 0.0,
                },
            }),
            ScenarioEventSpec::TrafficArea(TrafficAreaSpec {
                tick: 0,
                repeat_every: 0,
                area: TrafficAreaShape::Circle {
                    center_x: 0.0,
                    center_y: 0.0,
                    radius: 1.0,
                },
                template: TrafficTemplateSpec {
                    packet_id_base: 10,
                    ttl: 2,
                    size_bytes: 1,
                    quality: 1.0,
                    meta: false,
                    trg_id: 0,
                    route_hint: 0,
                },
                target: TrafficTargetSpec::SelfTarget,
            }),
        ],
    };

    let mut pipeline = netsim_core::SimPipeline::from_scenario(&scenario);
    let scene = scenario.build_scene();
    let generator = CpuWorldGenerator::new(scene.config, scene.sources, scene.seed);

    pipeline.step_with_scenario(&scenario, &generator);

    let id = pipeline.agents.memory_id[0];
    let memory = netsim_core::AgentMemory::new(&mut pipeline.memory_arena, id);
    assert_eq!(memory.block.descriptor().collect_every, 2);

    assert_eq!(pipeline.stats.packets_recv, 1);
}
