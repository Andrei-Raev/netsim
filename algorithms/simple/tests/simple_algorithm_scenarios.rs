use netsim_algorithm_simple::SimpleAlgorithm;
use netsim_core::{AgentAlgorithm, AgentMemory, AgentMemoryArena, AgentMemoryBuilder, AgentSoA};
use netsim_core::{Event, Packet, PacketSpec};

fn build_memory() -> (AgentMemoryArena, netsim_core::MemoryId) {
    let mut arena = AgentMemoryArena::new();
    let mut builder = AgentMemoryBuilder::new(&mut arena);
    let spec = netsim_core::AgentMemorySpec::placeholder(0);
    let (id, _) = builder.build(spec);
    (arena, id)
}

#[test]
fn simple_algorithm_drops_when_dst_is_self() {
    let mut agents = AgentSoA::new(1);
    agents.agent_id[0] = 10;

    let (mut arena, id) = build_memory();
    let mut memory = AgentMemory::new(&mut arena, id);

    let packet = Packet::from_spec(PacketSpec {
        packet_id: 1,
        src_id: 5,
        dst_id: 10,
        created_tick: 0,
        deliver_tick: 0,
        ttl: 2,
        size_bytes: 1,
        quality: 1.0,
        meta: false,
        route_hint: 0,
    });
    let event = Event::packet(10, 1, packet);

    let algorithm = SimpleAlgorithm::default();
    let outgoing = algorithm.eval_event(0, &agents, &mut memory, &event);

    assert!(outgoing.is_none());
}

#[test]
fn simple_algorithm_forwards_when_dst_differs() {
    let mut agents = AgentSoA::new(1);
    agents.agent_id[0] = 10;

    let (mut arena, id) = build_memory();
    let mut memory = AgentMemory::new(&mut arena, id);

    let packet = Packet::from_spec(PacketSpec {
        packet_id: 1,
        src_id: 5,
        dst_id: 20,
        created_tick: 0,
        deliver_tick: 0,
        ttl: 2,
        size_bytes: 1,
        quality: 1.0,
        meta: false,
        route_hint: 0,
    });
    let event = Event::packet(10, 1, packet);

    let algorithm = SimpleAlgorithm::default();
    let outgoing = algorithm
        .eval_event(0, &agents, &mut memory, &event)
        .expect("forwarded");

    assert_eq!(outgoing.payload.src_id, 10);
    assert_eq!(outgoing.payload.dst_id, 20);
    assert_eq!(outgoing.packet_seq, 2);
}
