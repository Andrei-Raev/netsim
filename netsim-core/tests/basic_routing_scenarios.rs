use netsim_core::{
    AgentAlgorithm, AgentMemory, AgentMemoryArena, AgentMemoryBuilder, AgentSoA,
    BasicRoutingAlgorithm, Event, Packet, PacketSpec,
};

fn build_memory(routing_cap: u32) -> (AgentMemoryArena, netsim_core::MemoryId) {
    let mut arena = AgentMemoryArena::new();
    let mut builder = AgentMemoryBuilder::new(&mut arena);
    let mut spec = netsim_core::AgentMemorySpec::placeholder(0);
    spec.routing_cap = routing_cap;
    let (id, _) = builder.build(spec);
    (arena, id)
}

fn packet_for(src_id: u32, dst_id: u32, trg_id: u32) -> Packet {
    Packet::from_spec(PacketSpec {
        packet_id: 1,
        src_id,
        dst_id,
        created_tick: 0,
        deliver_tick: 0,
        ttl: 2,
        size_bytes: 1,
        quality: 1.0,
        meta: false,
        trg_id,
        route_hint: 0,
    })
}

#[test]
fn basic_routing_forwards_to_next_hop() {
    let mut agents = AgentSoA::new(1);
    agents.agent_id[0] = 1;

    let (mut arena, id) = build_memory(2);
    let mut memory = AgentMemory::new(&mut arena, id);
    {
        let mut routing = memory.routing_table();
        routing.upsert(5, 2, 1.0, 2, 0).expect("route");
    }

    let event = Event::packet(1, 3, packet_for(10, 1, 5));
    let algorithm = BasicRoutingAlgorithm::default();
    let outgoing = algorithm
        .eval_event(0, &agents, &mut memory, &event)
        .expect("forwarded");

    assert_eq!(outgoing.agent_id, 2);
    assert_eq!(outgoing.payload.dst_id, 2);
    assert_eq!(outgoing.payload.trg_id, 5);
    assert_eq!(outgoing.payload.hop_count, 1);
}

#[test]
fn basic_routing_drops_when_no_route() {
    let mut agents = AgentSoA::new(1);
    agents.agent_id[0] = 1;

    let (mut arena, id) = build_memory(1);
    let mut memory = AgentMemory::new(&mut arena, id);

    let event = Event::packet(1, 3, packet_for(10, 1, 5));
    let algorithm = BasicRoutingAlgorithm::default();

    let outgoing = algorithm.eval_event(0, &agents, &mut memory, &event);
    assert!(outgoing.is_none());
}

#[test]
fn basic_routing_receives_when_target_is_self() {
    let mut agents = AgentSoA::new(1);
    agents.agent_id[0] = 5;

    let (mut arena, id) = build_memory(1);
    let mut memory = AgentMemory::new(&mut arena, id);

    let event = Event::packet(5, 3, packet_for(10, 5, 5));
    let algorithm = BasicRoutingAlgorithm::default();

    let outgoing = algorithm.eval_event(0, &agents, &mut memory, &event);
    assert!(outgoing.is_none());
}

#[test]
fn basic_routing_drops_when_route_ttl_zero() {
    let mut agents = AgentSoA::new(1);
    agents.agent_id[0] = 1;

    let (mut arena, id) = build_memory(2);
    let mut memory = AgentMemory::new(&mut arena, id);
    {
        let mut routing = memory.routing_table();
        routing.upsert(5, 2, 1.0, 1, 0).expect("route");
        routing.decay_ttl();
    }

    let event = Event::packet(1, 3, packet_for(10, 1, 5));
    let algorithm = BasicRoutingAlgorithm::default();

    let outgoing = algorithm.eval_event(0, &agents, &mut memory, &event);
    assert!(outgoing.is_none());
}
