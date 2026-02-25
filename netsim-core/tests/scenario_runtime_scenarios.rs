use netsim_core::{
    ScenarioConfig, ScenarioEventSpec, SceneSpec, SpawnAgentsSpec, SpawnShape, TrafficSpec,
};

#[test]
fn scenario_events_for_tick_respects_repeat() {
    let config = ScenarioConfig {
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
        ticks: 10,
        event_queue_window: 4,
        noise_drop_threshold: 0.0,
        scene: SceneSpec::Preset {
            name: "minimal".to_string(),
        },
        events: vec![
            ScenarioEventSpec::Traffic(TrafficSpec {
                tick: 2,
                packet_id: 1,
                src_id: 0,
                dst_id: 1,
                ttl: 2,
                size_bytes: 1,
                quality: 1.0,
                meta: false,
                route_hint: 0,
                repeat_every: 2,
            }),
            ScenarioEventSpec::SpawnAgents(SpawnAgentsSpec {
                tick: 1,
                agent_id_start: 0,
                count: 1,
                agent_spec: netsim_core::AgentSpec::placeholder(0),
                shape: SpawnShape::Grid {
                    rows: 1,
                    cols: 1,
                    spacing: 1.0,
                    origin_x: 0.0,
                    origin_y: 0.0,
                },
            }),
        ],
    };

    let events_t1 = config.events_for_tick(1);
    let events_t2 = config.events_for_tick(2);
    let events_t4 = config.events_for_tick(4);

    assert_eq!(events_t1.len(), 1);
    assert_eq!(events_t2.len(), 1);
    assert_eq!(events_t4.len(), 1);
}
