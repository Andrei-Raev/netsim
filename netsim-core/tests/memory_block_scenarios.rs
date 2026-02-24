use netsim_core::{
    AGENT_MEMORY_MAGIC, AGENT_MEMORY_VERSION, AgentMemoryArena, AgentMemoryBuilder, AgentMemorySpec,
};

#[test]
fn descriptor_contains_routing_and_scratch_caps() {
    let mut arena = AgentMemoryArena::new();
    let mut builder = AgentMemoryBuilder::new(&mut arena);
    let mut spec = AgentMemorySpec::placeholder(42);
    spec.routing_cap = 12;
    spec.scratch_cap = 256;
    let (id, _) = builder.build(spec);

    let block = arena.block(id);
    let descriptor = block.descriptor();

    assert_eq!(descriptor.routing_cap, 12);
    assert_eq!(descriptor.scratch_cap, 256);
}

#[test]
fn scratchpad_clear_resets_bytes() {
    let mut arena = AgentMemoryArena::new();
    let mut builder = AgentMemoryBuilder::new(&mut arena);
    let (id, _) = builder.build(AgentMemorySpec::placeholder(0));

    let mut block = arena.block_mut(id);
    let scratch = block.scratchpad_mut();
    if !scratch.is_empty() {
        scratch[0] = 0xFF;
        scratch[scratch.len() - 1] = 0xAA;
    }

    block.clear_scratchpad();
    let scratch = block.scratchpad_mut();
    assert!(scratch.iter().all(|byte| *byte == 0));
}

#[test]
fn header_and_descriptor_consistent_with_layout() {
    let mut arena = AgentMemoryArena::new();
    let mut builder = AgentMemoryBuilder::new(&mut arena);
    let mut spec = AgentMemorySpec::placeholder(7);
    spec.routing_cap = 3;
    spec.scratch_cap = 10;
    let (id, layout) = builder.build(spec);

    let block = arena.block(id);
    let header = block.header();

    assert_eq!(header.magic, AGENT_MEMORY_MAGIC);
    assert_eq!(header.version, AGENT_MEMORY_VERSION);
    assert_eq!(header.total_len, layout.total_len);
    assert_eq!(header.desc_len, layout.desc_len);
    assert_eq!(header.routing_len, layout.routing_len);
    assert_eq!(header.scratch_len, layout.scratch_len);
}
