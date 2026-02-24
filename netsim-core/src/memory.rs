use std::mem::{align_of, size_of};

/// Магическое значение заголовка памяти агента (`NSIM`).
pub const AGENT_MEMORY_MAGIC: u32 = 0x4E_53_49_4D;
/// Версия layout памяти агента.
pub const AGENT_MEMORY_VERSION: u16 = 1;

/// Идентификатор блока памяти агента в общем пуле.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryId {
    /// Базовый оффсет в пуле.
    pub base: u32,
    /// Длина блока в байтах.
    pub len: u32,
}

impl MemoryId {
    /// Создает новый идентификатор блока в пуле.
    pub fn new(base: u32, len: u32) -> Self {
        Self { base, len }
    }
}

/// Заголовок фиксированного блока памяти агента.
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy)]
pub struct AgentMemoryHeader {
    /// Магическое значение `NSIM`.
    pub magic: u32,
    /// Версия layout.
    pub version: u16,
    /// Резервные флаги.
    pub flags: u16,
    /// Полный размер блока в байтах.
    pub total_len: u32,
    /// Длина секции дескриптора.
    pub desc_len: u32,
    /// Длина секции таблицы маршрутизации.
    pub routing_len: u32,
    /// Длина секции scratchpad.
    pub scratch_len: u32,
    /// Емкость таблицы маршрутизации (число записей).
    pub routing_cap: u32,
    /// Активные записи в таблице маршрутизации.
    pub mem_used: u32,
    /// Смещения секций: descriptor, routing, scratch.
    pub offsets: [u32; 3],
}

/// Дескриптор агента в блоке памяти.
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, Default)]
pub struct AgentDescriptor {
    /// Идентификатор агента.
    pub agent_id: u32,
    /// Тип агента.
    pub type_id: u16,
    /// Выравнивание.
    pub _pad: u16,
    /// Емкость памяти (общий лимит в байтах).
    pub memory_cap: u32,
    /// Емкость таблицы маршрутизации (число записей).
    pub routing_cap: u32,
    /// Размер scratchpad в байтах.
    pub scratch_cap: u32,
    /// Вычислительная мощность.
    pub compute_power: f32,
    /// Пропускная способность.
    pub bandwidth: f32,
    /// Ограничение скорости движения.
    pub self_speed: f32,
}

/// Запись таблицы маршрутизации.
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, Default)]
pub struct RouteEntry {
    /// Идентификатор целевого агента.
    pub dst_id: u32,
    /// Следующий хоп.
    pub next_hop: u32,
    /// Стоимость маршрута.
    pub cost: f32,
    /// Последний тик, когда маршрут видели.
    pub last_seen_tick: u64,
    /// TTL маршрута.
    pub ttl: u16,
    /// Флаги состояния.
    pub flags: u8,
    /// Выравнивание.
    pub _pad: u8,
}

/// Флаг валидной записи маршрута.
pub const ROUTE_FLAG_VALID: u8 = 0b0000_0001;

/// Ошибки при работе с таблицей маршрутизации.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingTableError {
    /// Таблица заполнена — нет свободных слотов.
    Full,
    /// Некорректный TTL (0).
    InvalidTtl,
}

/// Описание параметров памяти агента, задаваемых фабрикой.
#[derive(Debug, Clone, Copy)]
pub struct AgentMemorySpec {
    /// Емкость таблицы маршрутизации (число записей).
    pub routing_cap: u32,
    /// Размер scratchpad в байтах.
    pub scratch_cap: u32,
    /// Вычислительная мощность.
    pub compute_power: f32,
    /// Пропускная способность.
    pub bandwidth: f32,
    /// Ограничение скорости движения.
    pub self_speed: f32,
    /// Идентификатор агента.
    pub agent_id: u32,
    /// Идентификатор типа агента.
    pub type_id: u16,
    /// Явно заданная емкость памяти (0 = вычислить автоматически).
    pub memory_cap: u32,
}

impl AgentMemorySpec {
    /// Создает базовый spec без зависимости от генератора мира.
    pub fn placeholder(agent_id: u32) -> Self {
        Self {
            routing_cap: 8,
            scratch_cap: 64,
            compute_power: 0.0,
            bandwidth: 0.0,
            self_speed: 0.0,
            agent_id,
            type_id: 0,
            memory_cap: 0,
        }
    }
}

/// Информация о layout памяти агента.
#[derive(Debug, Clone, Copy)]
pub struct AgentMemoryLayout {
    /// Полный размер блока.
    pub total_len: u32,
    /// Длина секции дескриптора.
    pub desc_len: u32,
    /// Длина секции таблицы маршрутизации.
    pub routing_len: u32,
    /// Длина секции scratchpad.
    pub scratch_len: u32,
    /// Смещение дескриптора.
    pub desc_offset: u32,
    /// Смещение таблицы маршрутизации.
    pub routing_offset: u32,
    /// Смещение scratchpad.
    pub scratch_offset: u32,
}

impl AgentMemoryLayout {
    /// Возвращает layout для заданных параметров памяти.
    pub fn new(routing_cap: u32, scratch_cap: u32) -> Self {
        let header_len = size_of::<AgentMemoryHeader>() as u32;
        let desc_len = size_of::<AgentDescriptor>() as u32;
        let routing_len = routing_cap.saturating_mul(size_of::<RouteEntry>() as u32);
        let scratch_len = scratch_cap;

        let desc_offset = align_up(header_len, 8);
        let routing_offset = align_up(desc_offset + desc_len, 8);
        let scratch_offset = align_up(routing_offset + routing_len, 8);
        let total_len = align_up(scratch_offset + scratch_len, 8);

        Self {
            total_len,
            desc_len,
            routing_len,
            scratch_len,
            desc_offset,
            routing_offset,
            scratch_offset,
        }
    }
}

/// Общий пул памяти агентов (arena).
#[derive(Debug, Default)]
pub struct AgentMemoryArena {
    data: Vec<u8>,
}

impl AgentMemoryArena {
    /// Создает пустой пул памяти агентов.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Выделяет блок памяти с заданным размером и выравниванием.
    pub fn allocate(&mut self, len: u32, align: usize) -> MemoryId {
        let base = align_up(self.data.len() as u32, align as u32) as usize;
        if base > self.data.len() {
            self.data.resize(base, 0);
        }
        let end = base + len as usize;
        if end > self.data.len() {
            self.data.resize(end, 0);
        }
        MemoryId::new(base as u32, len)
    }

    /// Возвращает immutable view на блок памяти.
    pub fn block(&self, id: MemoryId) -> AgentMemoryBlock<'_> {
        let base = id.base as usize;
        let end = base + id.len as usize;
        AgentMemoryBlock {
            data: &self.data[base..end],
        }
    }

    /// Возвращает mutable view на блок памяти.
    pub fn block_mut(&mut self, id: MemoryId) -> AgentMemoryBlockMut<'_> {
        let base = id.base as usize;
        let end = base + id.len as usize;
        AgentMemoryBlockMut {
            data: &mut self.data[base..end],
        }
    }
}

/// Builder для создания блоков памяти агентов.
pub struct AgentMemoryBuilder<'a> {
    arena: &'a mut AgentMemoryArena,
}

impl<'a> AgentMemoryBuilder<'a> {
    /// Создает builder для выделения блоков в заданном пуле.
    pub fn new(arena: &'a mut AgentMemoryArena) -> Self {
        Self { arena }
    }

    /// Создает блок памяти агента и возвращает его идентификатор и layout.
    pub fn build(&mut self, spec: AgentMemorySpec) -> (MemoryId, AgentMemoryLayout) {
        let layout = AgentMemoryLayout::new(spec.routing_cap, spec.scratch_cap);
        let id = self.arena.allocate(layout.total_len, 8);

        let mut block = self.arena.block_mut(id);
        let header = AgentMemoryHeader {
            magic: AGENT_MEMORY_MAGIC,
            version: AGENT_MEMORY_VERSION,
            flags: 0,
            total_len: layout.total_len,
            desc_len: layout.desc_len,
            routing_len: layout.routing_len,
            scratch_len: layout.scratch_len,
            routing_cap: spec.routing_cap,
            mem_used: 0,
            offsets: [
                layout.desc_offset,
                layout.routing_offset,
                layout.scratch_offset,
            ],
        };
        block.write_header(header);

        let memory_cap = if spec.memory_cap == 0 {
            layout.total_len
        } else {
            spec.memory_cap
        };
        let descriptor = AgentDescriptor {
            agent_id: spec.agent_id,
            type_id: spec.type_id,
            _pad: 0,
            memory_cap,
            routing_cap: spec.routing_cap,
            scratch_cap: spec.scratch_cap,
            compute_power: spec.compute_power,
            bandwidth: spec.bandwidth,
            self_speed: spec.self_speed,
        };
        block.write_descriptor(descriptor);

        (id, layout)
    }
}

/// Immutable view на блок памяти агента.
#[derive(Debug)]
pub struct AgentMemoryBlock<'a> {
    data: &'a [u8],
}

impl<'a> AgentMemoryBlock<'a> {
    /// Возвращает заголовок блока.
    pub fn header(&self) -> &AgentMemoryHeader {
        unsafe { cast_ref(self.data, 0) }
    }

    /// Возвращает дескриптор агента.
    pub fn descriptor(&self) -> &AgentDescriptor {
        let header = self.header();
        unsafe { cast_ref(self.data, header.offsets[0] as usize) }
    }

    /// Возвращает таблицу маршрутизации.
    pub fn routing_table(&self) -> &[RouteEntry] {
        let header = self.header();
        let offset = header.offsets[1] as usize;
        let entries = header.routing_cap as usize;
        unsafe { cast_slice(self.data, offset, entries) }
    }

    /// Возвращает scratchpad-буфер.
    pub fn scratchpad(&self) -> &[u8] {
        let header = self.header();
        let offset = header.offsets[2] as usize;
        let end = offset + header.scratch_len as usize;
        &self.data[offset..end]
    }
}

/// Mutable view на блок памяти агента.
#[derive(Debug)]
pub struct AgentMemoryBlockMut<'a> {
    data: &'a mut [u8],
}

impl<'a> AgentMemoryBlockMut<'a> {
    /// Возвращает заголовок блока (immutable).
    pub fn header(&self) -> &AgentMemoryHeader {
        unsafe { cast_ref(self.data, 0) }
    }

    /// Возвращает заголовок блока (mutable).
    pub fn header_mut(&mut self) -> &mut AgentMemoryHeader {
        unsafe { cast_mut(self.data, 0) }
    }

    /// Записывает заголовок блока.
    pub fn write_header(&mut self, header: AgentMemoryHeader) {
        let target = self.header_mut();
        *target = header;
    }

    /// Возвращает дескриптор агента (immutable).
    pub fn descriptor(&self) -> &AgentDescriptor {
        let header = self.header();
        unsafe { cast_ref(self.data, header.offsets[0] as usize) }
    }

    /// Возвращает дескриптор агента (mutable).
    pub fn descriptor_mut(&mut self) -> &mut AgentDescriptor {
        let offset = self.header().offsets[0] as usize;
        unsafe { cast_mut(self.data, offset) }
    }

    /// Записывает дескриптор агента.
    pub fn write_descriptor(&mut self, descriptor: AgentDescriptor) {
        let target = self.descriptor_mut();
        *target = descriptor;
    }

    /// Обновляет настройки производительности агента в дескрипторе.
    pub fn update_descriptor_params(
        &mut self,
        compute_power: f32,
        bandwidth: f32,
        self_speed: f32,
    ) {
        let descriptor = self.descriptor_mut();
        descriptor.compute_power = compute_power;
        descriptor.bandwidth = bandwidth;
        descriptor.self_speed = self_speed;
    }

    /// Возвращает таблицу маршрутизации (mutable).
    pub fn routing_table_mut(&mut self) -> RoutingTableViewMut<'_> {
        let header = self.header();
        let offset = header.offsets[1] as usize;
        let entries = header.routing_cap as usize;
        let mem_used = &mut self.header_mut().mem_used as *mut u32;
        let entries_ptr = unsafe { self.data.as_mut_ptr().add(offset) as *mut RouteEntry };
        RoutingTableViewMut {
            entries: unsafe { std::slice::from_raw_parts_mut(entries_ptr, entries) },
            mem_used,
        }
    }

    /// Возвращает raw‑указатели к таблице маршрутизации и счетчику mem_used.
    pub fn routing_table_ptrs(&mut self) -> (*mut RouteEntry, usize, *mut u32) {
        let header = self.header();
        let offset = header.offsets[1] as usize;
        let entries = header.routing_cap as usize;
        let mem_used = &mut self.header_mut().mem_used as *mut u32;
        let entries_ptr = unsafe { self.data.as_mut_ptr().add(offset) as *mut RouteEntry };
        (entries_ptr, entries, mem_used)
    }

    /// Возвращает scratchpad-буфер (mutable).
    pub fn scratchpad_mut(&mut self) -> &mut [u8] {
        let header = self.header();
        let offset = header.offsets[2] as usize;
        let end = offset + header.scratch_len as usize;
        &mut self.data[offset..end]
    }

    /// Очищает scratchpad (заполняет нулями).
    pub fn clear_scratchpad(&mut self) {
        let scratch = self.scratchpad_mut();
        scratch.fill(0);
    }
}

/// Mutable view таблицы маршрутизации.
#[derive(Debug)]
pub struct RoutingTableViewMut<'a> {
    entries: &'a mut [RouteEntry],
    mem_used: *mut u32,
}

impl<'a> RoutingTableViewMut<'a> {
    /// Возвращает емкость таблицы.
    pub fn capacity(&self) -> u32 {
        self.entries.len() as u32
    }

    /// Возвращает число активных записей.
    pub fn mem_used(&self) -> u32 {
        unsafe { *self.mem_used }
    }

    /// Устанавливает число активных записей.
    pub fn set_mem_used(&mut self, value: u32) {
        unsafe {
            *self.mem_used = value;
        }
    }

    /// Возвращает mutable-доступ к записям таблицы.
    pub fn entries_mut(&mut self) -> &mut [RouteEntry] {
        self.entries
    }

    /// Раскладывает view на ссылки к записям и счетчику.
    pub fn into_parts(self) -> (&'a mut [RouteEntry], *mut u32) {
        (self.entries, self.mem_used)
    }
}

fn align_up(value: u32, align: u32) -> u32 {
    if align == 0 {
        return value;
    }
    let mask = align - 1;
    (value + mask) & !mask
}

unsafe fn cast_ref<T>(data: &[u8], offset: usize) -> &T {
    debug_assert!(offset.is_multiple_of(align_of::<T>()));
    debug_assert!(offset + size_of::<T>() <= data.len());
    unsafe { &*(data.as_ptr().add(offset) as *const T) }
}

unsafe fn cast_mut<T>(data: &mut [u8], offset: usize) -> &mut T {
    debug_assert!(offset.is_multiple_of(align_of::<T>()));
    debug_assert!(offset + size_of::<T>() <= data.len());
    unsafe { &mut *(data.as_mut_ptr().add(offset) as *mut T) }
}

unsafe fn cast_slice<T>(data: &[u8], offset: usize, len: usize) -> &[T] {
    let bytes = len.saturating_mul(size_of::<T>());
    debug_assert!(offset.is_multiple_of(align_of::<T>()));
    debug_assert!(offset + bytes <= data.len());
    unsafe { std::slice::from_raw_parts(data.as_ptr().add(offset) as *const T, len) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_aligns_sections_to_8_bytes() {
        let layout = AgentMemoryLayout::new(3, 17);
        assert_eq!(layout.desc_offset % 8, 0);
        assert_eq!(layout.routing_offset % 8, 0);
        assert_eq!(layout.scratch_offset % 8, 0);
        assert_eq!(layout.total_len % 8, 0);
    }

    #[test]
    fn builder_writes_header_and_descriptor() {
        let mut arena = AgentMemoryArena::new();
        let mut builder = AgentMemoryBuilder::new(&mut arena);
        let mut spec = AgentMemorySpec::placeholder(7);
        spec.routing_cap = 4;
        spec.scratch_cap = 32;
        spec.compute_power = 1.0;
        spec.bandwidth = 2.0;
        spec.self_speed = 3.0;
        spec.type_id = 1;

        let (id, layout) = builder.build(spec);
        let block = arena.block(id);

        assert_eq!(block.header().magic, AGENT_MEMORY_MAGIC);
        assert_eq!(block.header().version, AGENT_MEMORY_VERSION);
        assert_eq!(block.header().total_len, layout.total_len);
        assert_eq!(block.descriptor().agent_id, 7);
        assert_eq!(block.descriptor().type_id, 1);
        assert_eq!(block.descriptor().memory_cap, layout.total_len);
        assert_eq!(block.routing_table().len(), 4);
        assert_eq!(block.scratchpad().len() as u32, layout.scratch_len);
    }

    #[test]
    fn routing_table_view_mut_updates_mem_used() {
        let mut arena = AgentMemoryArena::new();
        let mut builder = AgentMemoryBuilder::new(&mut arena);
        let mut spec = AgentMemorySpec::placeholder(1);
        spec.routing_cap = 2;
        spec.scratch_cap = 8;
        let (id, _) = builder.build(spec);

        let mut block = arena.block_mut(id);
        let mut table = block.routing_table_mut();
        table.set_mem_used(2);
        assert_eq!(table.mem_used(), 2);
    }
}
