/// Полезная нагрузка пакета, доставляемая как событие.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Packet {
    /// Глобально уникальный идентификатор пакета.
    pub packet_id: u64,
    /// Идентификатор исходного агента.
    pub src_id: u32,
    /// Идентификатор целевого агента.
    pub dst_id: u32,
    /// Тик, в который пакет создан.
    pub created_tick: u64,
    /// Тик, в который пакет должен быть доставлен.
    pub deliver_tick: u64,
    /// TTL для пакета.
    pub ttl: u16,
    /// Размер пакета в байтах.
    pub size_bytes: u32,
    /// Показатель качества/шума сигнала.
    pub quality: f32,
    /// Признак служебного пакета.
    pub meta: bool,
    /// Количество пройденных хопов.
    pub hop_count: u16,
    /// Тег полезной нагрузки для маршрутизации.
    pub payload_tag: u16,
    /// Идентификатор конечного адресата.
    pub trg_id: u32,
    /// Подсказка следующего хопа для будущей логики.
    pub route_hint: u32,
}

/// Набор обязательных полей для детерминированной сборки пакета.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PacketSpec {
    /// Глобально уникальный идентификатор пакета.
    pub packet_id: u64,
    /// Идентификатор исходного агента.
    pub src_id: u32,
    /// Идентификатор целевого агента.
    pub dst_id: u32,
    /// Тик, в который пакет создан.
    pub created_tick: u64,
    /// Тик, в который пакет должен быть доставлен.
    pub deliver_tick: u64,
    /// TTL для пакета.
    pub ttl: u16,
    /// Размер пакета в байтах.
    pub size_bytes: u32,
    /// Показатель качества/шума сигнала.
    pub quality: f32,
    /// Признак служебного пакета.
    pub meta: bool,
    /// Идентификатор конечного адресата.
    pub trg_id: u32,
    /// Подсказка следующего хопа для будущей логики.
    pub route_hint: u32,
}

impl Packet {
    /// Собирает пакет из обязательного набора полей.
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
            trg_id: spec.trg_id,
            route_hint: spec.route_hint,
        }
    }
}
