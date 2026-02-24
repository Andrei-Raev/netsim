use crate::packet::Packet;

/// Event kinds supported by the simulator core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// Packet delivery event.
    Packet,
    /// System-level maintenance event.
    System,
    /// Control or orchestration event.
    Control,
}

/// Deterministic event envelope stored in the queue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    /// Event category.
    pub kind: EventKind,
    /// Agent that receives or owns this event.
    pub agent_id: u32,
    /// Per-agent packet sequence used for deterministic ordering.
    pub packet_seq: u32,
    /// Packet payload for packet events.
    pub payload: Packet,
}

impl Event {
    /// Creates a packet event bound to an agent and sequence.
    pub fn packet(agent_id: u32, packet_seq: u32, payload: Packet) -> Self {
        Self {
            kind: EventKind::Packet,
            agent_id,
            packet_seq,
            payload,
        }
    }

    /// Returns the deterministic ordering key for the event.
    pub fn sort_key(&self) -> (u64, u32, u32) {
        (self.payload.deliver_tick, self.agent_id, self.packet_seq)
    }
}
