//! Простой алгоритм роевого общения.
//!
//! Алгоритм пересылает пакет дальше только если агент не является получателем,
//! потому что для старта важен максимально простой маршрут без оптимизации.

use netsim_core::{AgentAlgorithm, AgentMemory, AgentSoA, Event};

/// Простой алгоритм маршрутизации.
#[derive(Debug, Default)]
pub struct SimpleAlgorithm;

impl AgentAlgorithm for SimpleAlgorithm {
    fn eval_event(
        &self,
        agent_index: usize,
        agents: &AgentSoA,
        _memory: &mut AgentMemory,
        event: &Event,
    ) -> Option<Event> {
        let agent_id = agents.agent_id[agent_index];

        if event.payload.dst_id == agent_id {
            return None;
        }

        let mut packet = event.payload;
        packet.src_id = agent_id;
        let packet_seq = event.packet_seq.saturating_add(1);
        Some(Event::packet(agent_id, packet_seq, packet))
    }
}
