use netsim_algorithm_simple::SimpleAlgorithm;
use netsim_core::{AgentRuntime, AllowAllValidator, Event, Packet, PacketSpec, SimPipeline};

fn packet_for(agent_id: u32, dst_id: u32, packet_id: u64) -> Event {
    let packet = Packet::from_spec(PacketSpec {
        packet_id,
        src_id: agent_id,
        dst_id,
        created_tick: 0,
        deliver_tick: 0,
        ttl: 2,
        size_bytes: 1,
        quality: 1.0,
        meta: false,
        route_hint: 0,
    });

    Event::packet(agent_id, 1, packet)
}

#[test]
fn pipeline_uses_simple_algorithm_with_scratchpad() {
    let mut pipeline = SimPipeline::new(1);
    let runtime = AgentRuntime::new(
        Box::new(SimpleAlgorithm::default()),
        Box::new(AllowAllValidator),
    );
    pipeline.set_runtime(runtime);

    pipeline.event_queue.push(packet_for(0, 1, 10));
    pipeline.process_current_events();

    assert_eq!(pipeline.stats.packets_sent, 1);
    assert_eq!(pipeline.stats.packets_drop, 0);
    assert_eq!(pipeline.stats.packets_recv, 0);

    pipeline.event_queue.push(packet_for(0, 1, 10));
    pipeline.process_current_events();

    assert_eq!(pipeline.stats.packets_sent, 1);
    assert_eq!(pipeline.stats.packets_drop, 0);
    assert_eq!(pipeline.stats.packets_recv, 2);
}
