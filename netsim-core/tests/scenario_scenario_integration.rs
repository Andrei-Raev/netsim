use netsim_core::world::cpu::CpuWorldGenerator;
use netsim_core::{
    ScenarioConfig, ScenarioEventSpec, SceneSpec, SpawnAgentsSpec, SpawnShape, TrafficSpec,
};

#[test]
fn pipeline_spawns_agents_from_scenario() {
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
        ticks: 2,
        event_queue_window: 8,
        noise_drop_threshold: 0.0,
        scene: SceneSpec::Preset {
            name: "minimal".to_string(),
        },
        initial_events: netsim_core::InitialEventsConfig::default(),
        events: vec![ScenarioEventSpec::SpawnAgents(SpawnAgentsSpec {
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
        })],
    };

    let mut pipeline = netsim_core::SimPipeline::from_scenario(&scenario);
    let scene = scenario.build_scene();
    let generator = CpuWorldGenerator::new(scene.config, scene.sources, scene.seed);

    pipeline.step_with_scenario(&scenario, &generator);

    assert_eq!(pipeline.agents.len(), 4);
    assert!(pipeline.agents.pos_x.iter().any(|x| *x > 0.0));
}

#[test]
fn pipeline_enqueues_traffic_events() {
    let scenario = ScenarioConfig {
        world: netsim_core::WorldConfig {
            width: 2,
            height: 2,
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
                count: 1,
                agent_spec: {
                    let mut spec = netsim_core::AgentSpec::placeholder(0);
                    spec.collect_every = 2;
                    spec
                },
                shape: SpawnShape::Grid {
                    rows: 1,
                    cols: 1,
                    spacing: 1.0,
                    origin_x: 0.0,
                    origin_y: 0.0,
                },
            }),
            ScenarioEventSpec::Traffic(TrafficSpec {
                tick: 0,
                packet_id: 1,
                src_id: 0,
                dst_id: 0,
                ttl: 2,
                size_bytes: 32,
                quality: 1.0,
                meta: false,
                trg_id: 0,
                route_hint: 0,
                repeat_every: 0,
            }),
        ],
    };

    let mut pipeline = netsim_core::SimPipeline::from_scenario(&scenario);
    let scene = scenario.build_scene();
    let generator = CpuWorldGenerator::new(scene.config, scene.sources, scene.seed);

    pipeline.run_with_scenario(&scenario, &generator);

    assert!(pipeline.stats.packets_sent + pipeline.stats.packets_recv >= 1);
}
