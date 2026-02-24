/// Core packet payload that is delivered as an event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Packet {
    /// Globally unique packet id.
    pub packet_id: u64,
    /// Source agent id.
    pub src_id: u32,
    /// Destination agent id.
    pub dst_id: u32,
    /// Tick at which the packet was created.
    pub created_tick: u64,
    /// Tick at which the packet must be delivered.
    pub deliver_tick: u64,
    /// Time-to-live value for the packet.
    pub ttl: u16,
    /// Packet size in bytes.
    pub size_bytes: u32,
    /// Signal quality or noise indicator.
    pub quality: f32,
    /// Whether the packet is service/meta traffic.
    pub meta: bool,
    /// Hop count accumulated so far.
    pub hop_count: u16,
    /// Payload tag for higher-level routing.
    pub payload_tag: u16,
    /// Next hop hint reserved for future routing logic.
    pub route_hint: u32,
}

/// Required fields used to build a packet deterministically.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PacketSpec {
    /// Globally unique packet id.
    pub packet_id: u64,
    /// Source agent id.
    pub src_id: u32,
    /// Destination agent id.
    pub dst_id: u32,
    /// Tick at which the packet was created.
    pub created_tick: u64,
    /// Tick at which the packet must be delivered.
    pub deliver_tick: u64,
    /// Time-to-live value for the packet.
    pub ttl: u16,
    /// Packet size in bytes.
    pub size_bytes: u32,
    /// Signal quality or noise indicator.
    pub quality: f32,
    /// Whether the packet is service/meta traffic.
    pub meta: bool,
    /// Next hop hint reserved for future routing logic.
    pub route_hint: u32,
}

impl Packet {
    /// Builds a packet from a required field set.
    pub fn from_spec(spec: PacketSpec) -> Self {
        Self {
            packet_id: spec.packet_id,
            src_id: spec.src_id,
            dst_id: spec.dst_id,
            created_tick: spec.created_tick,
            deliver_tick: spec.deliver_tick,
            ttl: spec.ttl,
            size_bytes: spec.size_bytes,
            quality: spec.quality,
            meta: spec.meta,
            hop_count: 0,
            payload_tag: 0,
            route_hint: spec.route_hint,
        }
    }
}
