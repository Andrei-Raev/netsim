use netsim_core::{
    AgentAlgorithm, AgentRuntime, AgentSoA, AllowAllValidator, Event, InitialEventSpec, Packet,
    PacketSpec, SimConfig, SimPipeline,
};

fn event_for(agent_id: u32, packet_seq: u32, deliver_tick: u64) -> Event {
    let packet = Packet::from_spec(PacketSpec {
        packet_id: 1000 + u64::from(packet_seq),
        src_id: agent_id,
        dst_id: agent_id,
        created_tick: 0,
        deliver_tick,
        ttl: 3,
        size_bytes: 1,
        quality: 1.0,
        meta: false,
        trg_id: agent_id,
        route_hint: 0,
    });

    Event::packet(agent_id, packet_seq, packet)
}

#[derive(Debug, Default)]
struct EmitThenStop {
    remaining: std::sync::atomic::AtomicU8,
}

impl AgentAlgorithm for EmitThenStop {
    fn eval_event(
        &self,
        agent_index: usize,
        agents: &AgentSoA,
        _memory: &mut netsim_core::AgentMemory,
        event: &Event,
    ) -> Option<Event> {
        let left = self.remaining.load(std::sync::atomic::Ordering::Relaxed);
        if left == 0 {
            return None;
        }
        self.remaining
            .store(left.saturating_sub(1), std::sync::atomic::Ordering::Relaxed);

        let agent_id = agents.agent_id[agent_index];
        let mut packet = event.payload;
        packet.src_id = agent_id;
        packet.dst_id = agent_id;
        packet.packet_id = packet.packet_id.wrapping_add(1);
        packet.deliver_tick = event.payload.deliver_tick + 1;
        Some(Event::packet(agent_id, event.packet_seq + 1, packet))
    }
}

#[test]
fn pipeline_processes_multiple_ticks_and_updates_stats() {
    let mut pipeline = SimPipeline::new(1);
    let algorithm = EmitThenStop {
        remaining: std::sync::atomic::AtomicU8::new(2),
    };
    pipeline.runtime = AgentRuntime::new(Box::new(algorithm), Box::new(AllowAllValidator));

    let tick = pipeline.event_queue.current_tick();
    pipeline.event_queue.push(event_for(0, 1, tick));

    pipeline.step();
    pipeline.step();
    pipeline.step();

    assert_eq!(pipeline.stats.packets_sent, 2);
    assert_eq!(pipeline.stats.packets_recv, 1);
    assert_eq!(pipeline.stats.packets_drop, 0);
}

#[test]
fn pipeline_drops_events_for_missing_agent() {
    let mut pipeline = SimPipeline::new(0);
    let tick = pipeline.event_queue.current_tick();
    pipeline.event_queue.push(event_for(10, 1, tick));

    pipeline.process_current_events();

    assert_eq!(pipeline.stats.packets_drop, 1);
}

#[test]
fn pipeline_accepts_initial_events_from_config() {
    let config = SimConfig {
        agents_count: 1,
        ticks: 1,
        event_queue_window: 2,
        initial_events: vec![InitialEventSpec {
            agent_id: 0,
            packet_seq: 9,
            packet: PacketSpec {
                packet_id: 55,
                src_id: 0,
                dst_id: 0,
                created_tick: 0,
                deliver_tick: 0,
                ttl: 1,
                size_bytes: 1,
                quality: 1.0,
                meta: false,
                trg_id: 0,
                route_hint: 0,
            },
        }],
    };

    let mut pipeline = SimPipeline::from_config(config);
    pipeline.process_current_events();

    assert_eq!(pipeline.stats.packets_recv, 1);
    assert_eq!(pipeline.stats.packets_sent, 0);
}

#[test]
fn pipeline_drops_events_with_zero_ttl() {
    let config = SimConfig {
        agents_count: 1,
        ticks: 1,
        event_queue_window: 2,
        initial_events: vec![InitialEventSpec {
            agent_id: 0,
            packet_seq: 1,
            packet: PacketSpec {
                packet_id: 10,
                src_id: 0,
                dst_id: 0,
                created_tick: 0,
                deliver_tick: 0,
                ttl: 0,
                size_bytes: 1,
                quality: 1.0,
                meta: false,
                trg_id: 0,
                route_hint: 0,
            },
        }],
    };

    let mut pipeline = SimPipeline::from_config(config);
    pipeline.process_current_events();

    assert_eq!(pipeline.stats.packets_drop, 1);
    assert_eq!(pipeline.stats.packets_recv, 0);
}
