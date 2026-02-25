//! Простой алгоритм роевого общения.
//!
//! Алгоритм пересылает пакет дальше только если агент не является получателем,
//! потому что для старта важен максимально простой маршрут без оптимизации.
//! Дополнительно ведется кольцевой буфер хэшей в scratchpad,
//! потому что это предотвращает циклы пересылки без глобальной памяти.

use netsim_core::{AgentAlgorithm, AgentMemory, AgentSoA, Event};

/// Простой алгоритм маршрутизации.
#[derive(Debug, Default)]
pub struct SimpleAlgorithm;

impl AgentAlgorithm for SimpleAlgorithm {
    fn eval_event(
        &self,
        agent_index: usize,
        agents: &AgentSoA,
        memory: &mut AgentMemory,
        event: &Event,
    ) -> Option<Event> {
        let agent_id = agents.agent_id[agent_index];

        if event.payload.dst_id == agent_id {
            return None;
        }

        let hash = message_hash(event.payload.packet_id, event.payload.dst_id);
        let scratch = memory.block.scratchpad_mut();
        let mut ring = ScratchpadRing::new(scratch);
        if ring.contains(hash) {
            return None;
        }
        ring.push(hash);

        let mut packet = event.payload;
        packet.src_id = agent_id;
        let packet_seq = event.packet_seq.saturating_add(1);
        Some(Event::packet(agent_id, packet_seq, packet))
    }
}

fn message_hash(packet_id: u64, dst_id: u32) -> u64 {
    let mut value = packet_id ^ (dst_id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

struct ScratchpadRing<'a> {
    data: &'a mut [u8],
    capacity: usize,
}

impl<'a> ScratchpadRing<'a> {
    fn new(data: &'a mut [u8]) -> Self {
        let capacity = hash_capacity(data);
        let mut ring = Self { data, capacity };
        ring.ensure_header();
        ring
    }

    fn contains(&self, hash: u64) -> bool {
        if self.capacity == 0 {
            return false;
        }
        let len = self.len();
        let count = len.min(self.capacity);
        for index in 0..count {
            if self.read_hash(index) == hash {
                return true;
            }
        }
        false
    }

    fn push(&mut self, hash: u64) {
        if self.capacity == 0 {
            return;
        }
        let head = self.head();
        self.write_hash(head, hash);
        let next_head = (head + 1) % self.capacity;
        self.write_head(next_head);
        let len = self.len();
        if len < self.capacity {
            self.write_len(len + 1);
        }
    }

    fn ensure_header(&mut self) {
        if self.data.len() < header_size() {
            return;
        }
        let capacity = self.capacity;
        let len = self.len();
        let head = self.head();
        if len > capacity || head >= capacity {
            self.write_head(0);
            self.write_len(0);
        }
    }

    fn head(&self) -> usize {
        read_u32(self.data, 0).unwrap_or(0) as usize
    }

    fn len(&self) -> usize {
        read_u32(self.data, 4).unwrap_or(0) as usize
    }

    fn write_head(&mut self, value: usize) {
        write_u32(self.data, 0, value as u32);
    }

    fn write_len(&mut self, value: usize) {
        write_u32(self.data, 4, value as u32);
    }

    fn read_hash(&self, index: usize) -> u64 {
        let offset = header_size() + index * 8;
        read_u64(self.data, offset).unwrap_or(0)
    }

    fn write_hash(&mut self, index: usize, value: u64) {
        let offset = header_size() + index * 8;
        write_u64(self.data, offset, value);
    }
}

fn header_size() -> usize {
    8
}

fn hash_capacity(data: &[u8]) -> usize {
    data.len().saturating_sub(header_size()) / 8
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    let end = offset.saturating_add(4);
    if end > data.len() {
        return None;
    }
    let bytes: [u8; 4] = data[offset..end].try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) {
    let end = offset.saturating_add(4);
    if end > data.len() {
        return;
    }
    data[offset..end].copy_from_slice(&value.to_le_bytes());
}

fn read_u64(data: &[u8], offset: usize) -> Option<u64> {
    let end = offset.saturating_add(8);
    if end > data.len() {
        return None;
    }
    let bytes: [u8; 8] = data[offset..end].try_into().ok()?;
    Some(u64::from_le_bytes(bytes))
}

fn write_u64(data: &mut [u8], offset: usize, value: u64) {
    let end = offset.saturating_add(8);
    if end > data.len() {
        return;
    }
    data[offset..end].copy_from_slice(&value.to_le_bytes());
}
