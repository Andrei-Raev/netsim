use crate::memory::MemoryId;

/// Structure-of-arrays storage for all agents.
#[derive(Debug, Clone)]
pub struct AgentSoA {
    /// Stable agent ids.
    pub agent_id: Vec<u32>,
    /// Alive flag per agent.
    pub alive: Vec<bool>,
    /// Whether the agent is static and does not move.
    pub is_static: Vec<bool>,
    /// Per-agent packet sequence counter.
    pub packet_seq: Vec<u32>,
    /// Position X coordinate.
    pub pos_x: Vec<f32>,
    /// Position Y coordinate.
    pub pos_y: Vec<f32>,
    /// Target X coordinate.
    pub target_x: Vec<f32>,
    /// Target Y coordinate.
    pub target_y: Vec<f32>,
    /// Current energy level.
    pub energy: Vec<f32>,
    /// Compute power parameter.
    pub compute_power: Vec<f32>,
    /// Bandwidth parameter.
    pub bandwidth: Vec<f32>,
    /// Total packets sent by agent.
    pub packets_sent: Vec<u64>,
    /// Total packets received by agent.
    pub packets_recv: Vec<u64>,
    /// Total packets dropped by agent.
    pub packets_drop: Vec<u64>,
    /// Meta packets sent by agent.
    pub meta_packets_sent: Vec<u64>,
    /// Meta packets received by agent.
    pub meta_packets_recv: Vec<u64>,
    /// Memory block id for agent-owned data.
    pub memory_id: Vec<MemoryId>,
}

impl AgentSoA {
    /// Creates a new SoA with a fixed number of agents.
    pub fn new(count: usize) -> Self {
        let mut agent_id = Vec::with_capacity(count);
        for id in 0..count {
            agent_id.push(id as u32);
        }

        Self {
            agent_id,
            alive: vec![true; count],
            is_static: vec![false; count],
            packet_seq: vec![0; count],
            pos_x: vec![0.0; count],
            pos_y: vec![0.0; count],
            target_x: vec![0.0; count],
            target_y: vec![0.0; count],
            energy: vec![0.0; count],
            compute_power: vec![0.0; count],
            bandwidth: vec![0.0; count],
            packets_sent: vec![0; count],
            packets_recv: vec![0; count],
            packets_drop: vec![0; count],
            meta_packets_sent: vec![0; count],
            meta_packets_recv: vec![0; count],
            memory_id: vec![MemoryId::default(); count],
        }
    }

    /// Returns the number of agents stored in the SoA.
    pub fn len(&self) -> usize {
        self.agent_id.len()
    }

    /// Returns true if there are no agents in the SoA.
    pub fn is_empty(&self) -> bool {
        self.agent_id.is_empty()
    }
}
