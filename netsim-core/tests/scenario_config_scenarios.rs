use netsim_core::{ScenarioConfig, ScenarioEventSpec, SceneSpec, SpawnAgentsSpec, SpawnShape};

#[test]
fn scenario_builds_minimal_scene() {
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
        noise_drop_threshold: 0.5,
        scene: SceneSpec::Preset {
            name: "minimal".to_string(),
        },
        events: vec![ScenarioEventSpec::SpawnAgents(SpawnAgentsSpec {
            tick: 0,
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
        })],
    };

    let scene = config.build_scene();
    assert!(!scene.sources.is_empty());
}
