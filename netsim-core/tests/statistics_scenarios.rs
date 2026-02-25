use netsim_core::{AgentMemoryArena, StatisticsCollector};

#[test]
fn statistics_collector_updates_descriptor_stats() {
    let mut arena = AgentMemoryArena::new();
    let mut builder = netsim_core::AgentMemoryBuilder::new(&mut arena);
    let spec = netsim_core::AgentMemorySpec::placeholder(0);
    let (id, _) = builder.build(spec);
    let mut memory = netsim_core::AgentMemory::new(&mut arena, id);

    let collector = StatisticsCollector::default();
    collector.on_send(&mut memory, 0.8);
    collector.on_receive(&mut memory, 0.9);
    collector.on_drop(&mut memory);

    let stats = memory.block.stats();

    assert_eq!(stats.sent_count, 1);
    assert_eq!(stats.recv_count, 1);
    assert_eq!(stats.drop_count, 1);
    assert_eq!(stats.quality_samples, 2);
}
