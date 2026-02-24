use netsim_core::{
    AgentMemory, AgentMemoryArena, AgentMemoryBuilder, AgentMemorySpec, ROUTE_FLAG_VALID,
    RoutingTableError,
};

fn setup_memory(capacity: u32) -> (AgentMemoryArena, netsim_core::MemoryId) {
    let mut arena = AgentMemoryArena::new();
    let mut builder = AgentMemoryBuilder::new(&mut arena);
    let mut spec = AgentMemorySpec::placeholder(0);
    spec.routing_cap = capacity;
    let (id, _) = builder.build(spec);
    (arena, id)
}

#[test]
fn routing_table_upsert_inserts_and_updates() {
    let (mut arena, id) = setup_memory(4);
    let mut memory = AgentMemory::new(&mut arena, id);
    let mut table = memory.routing_table();

    assert!(table.upsert(10, 1, 1.0, 3, 100).is_ok());
    assert_eq!(table.mem_used(), 1);

    assert!(table.upsert(10, 2, 2.0, 5, 120).is_ok());
    assert_eq!(table.mem_used(), 1);

    let entry = table.find(10).expect("entry should exist");
    assert_eq!(entry.next_hop, 2);
    assert_eq!(entry.cost, 2.0);
    assert_eq!(entry.ttl, 5);
    assert_eq!(entry.last_seen_tick, 120);
    assert_eq!(entry.flags & ROUTE_FLAG_VALID, ROUTE_FLAG_VALID);
}

#[test]
fn routing_table_rejects_zero_ttl() {
    let (mut arena, id) = setup_memory(2);
    let mut memory = AgentMemory::new(&mut arena, id);
    let mut table = memory.routing_table();

    let result = table.upsert(1, 1, 1.0, 0, 0);
    assert_eq!(result, Err(RoutingTableError::InvalidTtl));
    assert_eq!(table.mem_used(), 0);
}

#[test]
fn routing_table_reports_full_capacity() {
    let (mut arena, id) = setup_memory(1);
    let mut memory = AgentMemory::new(&mut arena, id);
    let mut table = memory.routing_table();

    assert!(table.upsert(1, 1, 1.0, 2, 0).is_ok());
    let result = table.upsert(2, 2, 1.0, 2, 0);
    assert_eq!(result, Err(RoutingTableError::Full));
    assert_eq!(table.mem_used(), 1);
}

#[test]
fn routing_table_remove_clears_entry_and_mem_used() {
    let (mut arena, id) = setup_memory(2);
    let mut memory = AgentMemory::new(&mut arena, id);
    let mut table = memory.routing_table();

    assert!(table.upsert(1, 1, 1.0, 2, 0).is_ok());
    assert!(table.remove(1));
    assert_eq!(table.mem_used(), 0);
    assert!(table.find(1).is_none());
}

#[test]
fn routing_table_decay_and_cleanup_expired() {
    let (mut arena, id) = setup_memory(3);
    let mut memory = AgentMemory::new(&mut arena, id);
    let mut table = memory.routing_table();

    assert!(table.upsert(1, 1, 1.0, 1, 0).is_ok());
    assert!(table.upsert(2, 2, 1.0, 2, 0).is_ok());
    assert!(table.upsert(3, 3, 1.0, 3, 0).is_ok());

    table.decay_ttl();
    table.cleanup_expired();
    assert_eq!(table.mem_used(), 2);

    table.decay_ttl();
    table.cleanup_expired();
    assert_eq!(table.mem_used(), 1);

    table.decay_ttl();
    table.cleanup_expired();
    assert_eq!(table.mem_used(), 0);
}
