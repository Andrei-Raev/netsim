use crate::AgentSoA;

/// Хеш‑грид для ускоренного поиска агентов по ячейкам мира.
///
/// CPU‑вариант: хранит списки индексов агентов для каждой ячейки.
#[derive(Debug, Clone)]
pub struct AgentHashGrid {
    width: usize,
    height: usize,
    cell_size: f32,
    buckets: Vec<Vec<usize>>,
}

impl AgentHashGrid {
    /// Создаёт хеш‑грид с заданными размерами и размером ячейки.
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        let bucket_count = width.saturating_mul(height);
        Self {
            width,
            height,
            cell_size,
            buckets: vec![Vec::new(); bucket_count],
        }
    }

    /// Очищает все ячейки.
    pub fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
    }

    /// Возвращает число ячеек.
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    /// Перестраивает сетку по позициям агентов.
    ///
    /// Возвращает число агентов, выпавших за границы сетки.
    pub fn rebuild(&mut self, agents: &AgentSoA) -> usize {
        self.clear();

        let mut skipped = 0usize;
        for (index, alive) in agents.alive.iter().enumerate() {
            if !*alive {
                continue;
            }
            let x = agents.pos_x[index];
            let y = agents.pos_y[index];
            if !self.insert(index, x, y) {
                skipped = skipped.saturating_add(1);
            }
        }

        skipped
    }

    /// Возвращает список агентов в ячейке.
    pub fn bucket(&self, x: usize, y: usize) -> Option<&[usize]> {
        let index = self.cell_index(x, y)?;
        self.buckets.get(index).map(|bucket| bucket.as_slice())
    }

    /// Вставляет агента по координатам.
    ///
    /// Возвращает false, если координаты вне границ.
    pub fn insert(&mut self, agent_index: usize, x: f32, y: f32) -> bool {
        let (cell_x, cell_y) = match self.world_to_cell(x, y) {
            Some(value) => value,
            None => return false,
        };
        let index = match self.cell_index(cell_x, cell_y) {
            Some(value) => value,
            None => return false,
        };
        if let Some(bucket) = self.buckets.get_mut(index) {
            bucket.push(agent_index);
            return true;
        }
        false
    }

    /// Преобразует мировые координаты в координаты ячейки.
    pub fn world_to_cell(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        if self.cell_size <= 0.0 {
            return None;
        }
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let cell_x = (x / self.cell_size).floor() as i64;
        let cell_y = (y / self.cell_size).floor() as i64;
        if cell_x < 0 || cell_y < 0 {
            return None;
        }
        Some((cell_x as usize, cell_y as usize))
    }

    fn cell_index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(y * self.width + x)
    }
}
