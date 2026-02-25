/// Конфигурация детерминированного генератора начальных событий.
#[derive(Debug, Clone, Default)]
pub struct InitialEventsConfig {
    /// Seed генерации.
    pub seed: u64,
    /// Правила генерации.
    pub rules: Vec<InitialEventRule>,
}

impl InitialEventsConfig {
    /// Генерирует список событий для заданного тика.
    pub fn events_for_tick(&self, tick: u64) -> Vec<crate::TrafficSpec> {
        let mut events = Vec::new();

        for (index, rule) in self.rules.iter().enumerate() {
            if rule.tick != tick {
                continue;
            }

            let generated = rule.generate(self.seed, index as u64, tick);
            events.extend(generated);
        }

        events
    }
}

/// Правило генерации начальных событий.
#[derive(Debug, Clone)]
pub struct InitialEventRule {
    /// Тик генерации события.
    pub tick: u64,
    /// Сколько событий сгенерировать.
    pub count: u32,
    /// Базовый идентификатор пакета.
    pub packet_id_base: u64,
    /// Диапазон источников (включительно).
    pub src_range: (u32, u32),
    /// Диапазон получателей (включительно).
    pub dst_range: (u32, u32),
    /// TTL пакета.
    pub ttl: u16,
    /// Размер пакета в байтах.
    pub size_bytes: u32,
    /// Показатель качества/шума сигнала.
    pub quality: f32,
    /// Признак служебного пакета.
    pub meta: bool,
    /// Идентификатор конечного адресата.
    pub trg_id: u32,
    /// Подсказка следующего хопа.
    pub route_hint: u32,
}

impl InitialEventRule {
    /// Генерирует события по правилу для указанного тика.
    pub fn generate(&self, seed: u64, rule_index: u64, tick: u64) -> Vec<crate::TrafficSpec> {
        let mut events = Vec::new();

        if self.count == 0 {
            return events;
        }

        let src_span = span_inclusive(self.src_range.0, self.src_range.1);
        let dst_span = span_inclusive(self.dst_range.0, self.dst_range.1);
        let (src_span, dst_span) = match (src_span, dst_span) {
            (Some(src_span), Some(dst_span)) => (src_span, dst_span),
            _ => return events,
        };

        for offset in 0..self.count {
            let salt = mix_salt(seed, rule_index, tick, offset as u64);
            let src = self.src_range.0 + (salt % src_span) as u32;
            let dst = self.dst_range.0 + (mix64(salt) % dst_span) as u32;
            let packet_id = self.packet_id_base.saturating_add(offset as u64);

            events.push(crate::TrafficSpec {
                tick: self.tick,
                packet_id,
                src_id: src,
                dst_id: dst,
                ttl: self.ttl,
                size_bytes: self.size_bytes,
                quality: self.quality,
                meta: self.meta,
                trg_id: self.trg_id,
                route_hint: self.route_hint,
                repeat_every: 0,
            });
        }

        events
    }
}

fn span_inclusive(min: u32, max: u32) -> Option<u64> {
    if min > max {
        return None;
    }
    let span = (max - min) as u64 + 1;
    if span == 0 {
        return None;
    }
    Some(span)
}

fn mix_salt(seed: u64, rule_index: u64, tick: u64, offset: u64) -> u64 {
    let mut value = seed ^ rule_index.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value ^= tick.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= offset.wrapping_mul(0x94D0_49BB_1331_11EB);
    mix64(value)
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    value
}
