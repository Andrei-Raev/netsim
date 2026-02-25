use netsim_core::world::cpu::CpuWorldGenerator;
use netsim_core::{
    AgentRuntime, AllowAllValidator, Event, Packet, PacketSpec, SimPipeline, minimal_scene,
};

#[derive(Debug, Default)]
struct EmitOnce;

impl netsim_core::AgentAlgorithm for EmitOnce {
    fn eval_event(
        &self,
        agent_index: usize,
        agents: &netsim_core::AgentSoA,
        _memory: &mut netsim_core::AgentMemory,
        event: &Event,
    ) -> Option<Event> {
        let agent_id = agents.agent_id[agent_index];
        let mut packet = event.payload;
        packet.src_id = agent_id;
        packet.dst_id = agent_id;
        packet.packet_id = packet.packet_id.wrapping_add(1);
        Some(Event::packet(agent_id, event.packet_seq + 1, packet))
    }
}

#[test]
fn pipeline_step_with_world_drops_by_noise() {
    let scene = minimal_scene(4, 4, 1.0, 1);
    let generator = CpuWorldGenerator::new(scene.config, scene.sources, scene.seed);

    let mut pipeline = SimPipeline::new(1);
    pipeline.runtime =
        AgentRuntime::new(Box::new(EmitOnce::default()), Box::new(AllowAllValidator));
    pipeline.agents.pos_x[0] = 2.0;
    pipeline.agents.pos_y[0] = 2.0;
    pipeline.set_world_noise_drop_threshold(0.2);

    let packet = Packet::from_spec(PacketSpec {
        packet_id: 1,
        src_id: 0,
        dst_id: 0,
        created_tick: 0,
        deliver_tick: pipeline.event_queue.current_tick(),
        ttl: 2,
        size_bytes: 1,
        quality: 1.0,
        meta: false,
        route_hint: 0,
    });
    pipeline.event_queue.push(Event::packet(0, 1, packet));

    pipeline.step_with_world(&generator);

    assert_eq!(pipeline.stats.packets_drop, 1);
    assert_eq!(pipeline.stats.packets_sent, 0);
    assert_eq!(pipeline.stats.packets_recv, 0);
}

#[test]
fn pipeline_step_with_world_allows_without_threshold() {
    let scene = minimal_scene(4, 4, 1.0, 1);
    let generator = CpuWorldGenerator::new(scene.config, scene.sources, scene.seed);

    let mut pipeline = SimPipeline::new(1);
    pipeline.runtime =
        AgentRuntime::new(Box::new(EmitOnce::default()), Box::new(AllowAllValidator));
    pipeline.agents.pos_x[0] = 2.0;
    pipeline.agents.pos_y[0] = 2.0;

    let packet = Packet::from_spec(PacketSpec {
        packet_id: 1,
        src_id: 0,
        dst_id: 0,
        created_tick: 0,
        deliver_tick: pipeline.event_queue.current_tick(),
        ttl: 2,
        size_bytes: 1,
        quality: 1.0,
        meta: false,
        route_hint: 0,
    });
    pipeline.event_queue.push(Event::packet(0, 1, packet));

    pipeline.step_with_world(&generator);

    assert_eq!(pipeline.stats.packets_sent, 1);
}
