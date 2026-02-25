use netsim_core::{InitialEventRule, InitialEventsConfig};

#[test]
fn initial_events_are_deterministic_for_same_seed() {
    let config = InitialEventsConfig {
        seed: 42,
        rules: vec![InitialEventRule {
            tick: 0,
            count: 4,
            packet_id_base: 10,
            src_range: (0, 3),
            dst_range: (4, 7),
            ttl: 2,
            size_bytes: 8,
            quality: 1.0,
            meta: false,
            trg_id: 0,
            route_hint: 0,
        }],
    };

    let left = config.events_for_tick(0);
    let right = config.events_for_tick(0);

    assert_eq!(left, right);
}

#[test]
fn initial_events_change_with_seed() {
    let config_a = InitialEventsConfig {
        seed: 1,
        rules: vec![InitialEventRule {
            tick: 0,
            count: 4,
            packet_id_base: 10,
            src_range: (0, 3),
            dst_range: (4, 7),
            ttl: 2,
            size_bytes: 8,
            quality: 1.0,
            meta: false,
            trg_id: 0,
            route_hint: 0,
        }],
    };

    let config_b = InitialEventsConfig {
        seed: 2,
        rules: vec![InitialEventRule {
            tick: 0,
            count: 4,
            packet_id_base: 10,
            src_range: (0, 3),
            dst_range: (4, 7),
            ttl: 2,
            size_bytes: 8,
            quality: 1.0,
            meta: false,
            trg_id: 0,
            route_hint: 0,
        }],
    };

    let left = config_a.events_for_tick(0);
    let right = config_b.events_for_tick(0);

    assert_ne!(left, right);
}

#[test]
fn initial_events_respect_bounds_and_count() {
    let config = InitialEventsConfig {
        seed: 5,
        rules: vec![InitialEventRule {
            tick: 0,
            count: 5,
            packet_id_base: 100,
            src_range: (2, 4),
            dst_range: (10, 12),
            ttl: 1,
            size_bytes: 4,
            quality: 0.5,
            meta: true,
            trg_id: 0,
            route_hint: 1,
        }],
    };

    let events = config.events_for_tick(0);

    assert_eq!(events.len(), 5);
    for (index, event) in events.iter().enumerate() {
        assert!(event.src_id >= 2 && event.src_id <= 4);
        assert!(event.dst_id >= 10 && event.dst_id <= 12);
        assert_eq!(event.packet_id, 100 + index as u64);
    }
}

#[test]
fn initial_events_ignore_invalid_ranges() {
    let config = InitialEventsConfig {
        seed: 1,
        rules: vec![InitialEventRule {
            tick: 0,
            count: 3,
            packet_id_base: 1,
            src_range: (5, 2),
            dst_range: (1, 0),
            ttl: 1,
            size_bytes: 1,
            quality: 1.0,
            meta: false,
            trg_id: 0,
            route_hint: 0,
        }],
    };

    let events = config.events_for_tick(0);

    assert!(events.is_empty());
}
