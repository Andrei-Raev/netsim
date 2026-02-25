use netsim_core::{Event, EventQueue, EventQueueConfig, Packet, PacketSpec};
use proptest::prelude::*;

fn event_for(agent_id: u32, packet_seq: u32, deliver_tick: u64) -> Event {
    let packet = Packet::from_spec(PacketSpec {
        packet_id: (u64::from(agent_id) << 32) | u64::from(packet_seq),
        src_id: agent_id,
        dst_id: agent_id,
        created_tick: 0,
        deliver_tick,
        ttl: 1,
        size_bytes: 1,
        quality: 1.0,
        meta: false,
        trg_id: agent_id,
        route_hint: 0,
    });

    Event::packet(agent_id, packet_seq, packet)
}

proptest! {
    #[test]
    fn events_for_current_tick_are_sorted(
        agent_ids in prop::collection::vec(0u32..50, 1..50),
        packet_seqs in prop::collection::vec(0u32..50, 1..50),
    ) {
        let mut queue = EventQueue::new(EventQueueConfig { window_size: 8 });
        let tick = queue.current_tick();

        for (&agent_id, &packet_seq) in agent_ids.iter().zip(packet_seqs.iter()) {
            queue.push(event_for(agent_id, packet_seq, tick));
        }

        let events = queue.pop_current();
        let mut last = (0u32, 0u32);
        for (index, event) in events.iter().enumerate() {
            let key = (event.agent_id, event.packet_seq);
            if index > 0 {
                prop_assert!(key >= last);
            }
            last = key;
        }
    }

    #[test]
    fn overflow_is_used_for_out_of_window_events(
        deliver_offset in 9u64..32,
        agent_id in 0u32..10,
        packet_seq in 0u32..10,
    ) {
        let mut queue = EventQueue::new(EventQueueConfig { window_size: 8 });
        let deliver_tick = queue.current_tick() + deliver_offset;

        queue.push(event_for(agent_id, packet_seq, deliver_tick));

        prop_assert!(queue.overflow_len() >= 1);
        prop_assert!(queue.pop_current().is_empty());
    }
}
