use crate::WorldScene;
use crate::world::cpu::CpuWorldGenerator;

impl WorldScene {
    /// Создаёт CPU‑генератор для сцены.
    pub fn into_generator(&self) -> CpuWorldGenerator {
        CpuWorldGenerator::new(self.config, self.sources.clone(), self.seed)
    }
}
