use netsim_algorithm_simple::SimpleAlgorithm;
use netsim_core::{AgentAlgorithm, AgentMemory, AgentMemoryArena, AgentMemoryBuilder, AgentSoA};
use netsim_core::{Event, Packet, PacketSpec};

fn build_memory(scratch_cap: u32) -> (AgentMemoryArena, netsim_core::MemoryId) {
    let mut arena = AgentMemoryArena::new();
    let mut builder = AgentMemoryBuilder::new(&mut arena);
    let mut spec = netsim_core::AgentMemorySpec::placeholder(0);
    spec.scratch_cap = scratch_cap;
    let (id, _) = builder.build(spec);
    (arena, id)
}

fn packet_for(dst_id: u32, packet_id: u64) -> Packet {
    Packet::from_spec(PacketSpec {
        packet_id,
        src_id: 5,
        dst_id,
        created_tick: 0,
        deliver_tick: 0,
        ttl: 2,
        size_bytes: 1,
        quality: 1.0,
        meta: false,
        route_hint: 0,
    })
}

#[test]
fn simple_algorithm_drops_when_dst_is_self() {
    let mut agents = AgentSoA::new(1);
    agents.agent_id[0] = 10;

    let (mut arena, id) = build_memory(64);
    let mut memory = AgentMemory::new(&mut arena, id);

    let event = Event::packet(10, 1, packet_for(10, 1));

    let algorithm = SimpleAlgorithm::default();
    let outgoing = algorithm.eval_event(0, &agents, &mut memory, &event);

    assert!(outgoing.is_none());
}

#[test]
fn simple_algorithm_forwards_when_dst_differs() {
    let mut agents = AgentSoA::new(1);
    agents.agent_id[0] = 10;

    let (mut arena, id) = build_memory(64);
    let mut memory = AgentMemory::new(&mut arena, id);

    let event = Event::packet(10, 1, packet_for(20, 1));

    let algorithm = SimpleAlgorithm::default();
    let outgoing = algorithm
        .eval_event(0, &agents, &mut memory, &event)
        .expect("forwarded");

    assert_eq!(outgoing.payload.src_id, 10);
    assert_eq!(outgoing.payload.dst_id, 20);
    assert_eq!(outgoing.packet_seq, 2);
}

#[test]
fn simple_algorithm_blocks_duplicate_hashes() {
    let mut agents = AgentSoA::new(1);
    agents.agent_id[0] = 10;

    let (mut arena, id) = build_memory(64);
    let mut memory = AgentMemory::new(&mut arena, id);

    let event = Event::packet(10, 1, packet_for(20, 9));

    let algorithm = SimpleAlgorithm::default();
    let first = algorithm.eval_event(0, &agents, &mut memory, &event);
    let second = algorithm.eval_event(0, &agents, &mut memory, &event);

    assert!(first.is_some());
    assert!(second.is_none());
}

#[test]
fn simple_algorithm_allows_overwrite_after_capacity() {
    let mut agents = AgentSoA::new(1);
    agents.agent_id[0] = 10;

    let (mut arena, id) = build_memory(24);
    let mut memory = AgentMemory::new(&mut arena, id);

    let algorithm = SimpleAlgorithm::default();

    for packet_id in 0..4 {
        let event = Event::packet(10, 1, packet_for(20, packet_id));
        let _ = algorithm.eval_event(0, &agents, &mut memory, &event);
    }

    let event = Event::packet(10, 1, packet_for(20, 0));
    let outgoing = algorithm.eval_event(0, &agents, &mut memory, &event);

    assert!(outgoing.is_some());
}
