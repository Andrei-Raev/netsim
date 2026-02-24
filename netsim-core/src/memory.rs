/// Identifier for an agent-owned memory block in the global memory pool.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryId {
    /// Base offset in the pool.
    pub base: u32,
    /// Length of the block in bytes.
    pub len: u32,
}

impl MemoryId {
    /// Creates a new memory id pointing to a block in the pool.
    pub fn new(base: u32, len: u32) -> Self {
        Self { base, len }
    }
}
