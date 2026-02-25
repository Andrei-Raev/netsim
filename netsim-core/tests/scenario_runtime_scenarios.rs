use netsim_core::{
    InitialEventRule, InitialEventsConfig, ScenarioConfig, ScenarioEventSpec, SceneSpec,
    SpawnAgentsSpec, SpawnShape, TrafficAreaShape, TrafficAreaSpec, TrafficSpec, TrafficTargetSpec,
    TrafficTemplateSpec,
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
        initial_events: InitialEventsConfig {
            seed: 10,
            rules: vec![InitialEventRule {
                tick: 4,
                count: 1,
                packet_id_base: 99,
                src_range: (0, 0),
                dst_range: (1, 1),
                ttl: 1,
                size_bytes: 1,
                quality: 1.0,
                meta: false,
                trg_id: 0,
                route_hint: 0,
            }],
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
                trg_id: 0,
                route_hint: 0,
                repeat_every: 2,
            }),
            ScenarioEventSpec::SpawnAgents(SpawnAgentsSpec {
                tick: 1,
                agent_id_start: 0,
                count: 1,
                agent_spec: {
                    let mut spec = netsim_core::AgentSpec::placeholder(0);
                    spec.collect_every = 1;
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
            ScenarioEventSpec::TrafficArea(TrafficAreaSpec {
                tick: 3,
                repeat_every: 0,
                area: TrafficAreaShape::Rect {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 1.0,
                    max_y: 1.0,
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

    let events_t1 = config.events_for_tick(1);
    let events_t2 = config.events_for_tick(2);
    let events_t3 = config.events_for_tick(3);
    let events_t4 = config.events_for_tick(4);

    assert_eq!(events_t1.len(), 1);
    assert_eq!(events_t2.len(), 1);
    assert_eq!(events_t3.len(), 1);
    assert_eq!(events_t4.len(), 2);
}
