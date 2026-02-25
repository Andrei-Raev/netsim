use netsim_core::{
    ScenarioConfig, ScenarioEventSpec, SceneSpec, TrafficAreaShape, TrafficAreaSpec,
    TrafficTargetSpec, TrafficTemplateSpec, WorldBase, WorldConfig,
};

#[test]
fn traffic_area_queues_events_in_current_tick() {
    let scenario = ScenarioConfig {
        world: WorldConfig {
            width: 2,
            height: 2,
            cell_size: 1.0,
            base: WorldBase {
                load: 0.0,
                noise: 0.0,
                bandwidth: 0.0,
                cost: 0.0,
            },
        },
        seed: 1,
        ticks: 1,
        event_queue_window: 4,
        noise_drop_threshold: 0.0,
        scene: SceneSpec::Preset {
            name: "minimal".to_string(),
        },
        initial_events: netsim_core::InitialEventsConfig::default(),
        events: vec![ScenarioEventSpec::TrafficArea(TrafficAreaSpec {
            tick: 0,
            repeat_every: 0,
            area: TrafficAreaShape::Rect {
                min_x: -1.0,
                min_y: -1.0,
                max_x: 1.0,
                max_y: 1.0,
            },
            template: TrafficTemplateSpec {
                packet_id_base: 1,
                ttl: 1,
                size_bytes: 1,
                quality: 1.0,
                meta: false,
                route_hint: 0,
            },
            target: TrafficTargetSpec::SelfTarget,
        })],
    };

    let mut pipeline = netsim_core::SimPipeline::from_scenario(&scenario);
    pipeline.agents.extend(1);
    let mut builder = netsim_core::AgentBuilder::new(&mut pipeline.memory_arena);
    let mut spec = netsim_core::AgentSpec::placeholder(0);
    spec.collect_every = 2;
    builder.build(&mut pipeline.agents, 0, spec);

    pipeline.agents.pos_x[0] = 0.0;
    pipeline.agents.pos_y[0] = 0.0;

    let generator =
        netsim_core::world::cpu::CpuWorldGenerator::new(scenario.world, Vec::new(), scenario.seed);

    pipeline.step_with_scenario(&scenario, &generator);

    assert_eq!(pipeline.stats.packets_recv, 1);
}
