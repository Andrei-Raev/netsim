use netsim_core::{ProcessSend, WorldCell, WorldGrid};

#[test]
fn process_send_decrements_ttl_and_quality() {
    let processor = ProcessSend::default();
    let grid = WorldGrid {
        width: 1,
        height: 1,
        cell_size: 1.0,
        cells: vec![WorldCell {
            load: 0.0,
            noise: 0.4,
            bandwidth: 0.0,
            cost: 0.0,
        }],
    };
    let event = netsim_core::Event::packet(
        0,
        1,
        netsim_core::Packet::from_spec(netsim_core::PacketSpec {
            packet_id: 1,
            src_id: 0,
            dst_id: 1,
            created_tick: 0,
            deliver_tick: 0,
            ttl: 3,
            size_bytes: 1,
            quality: 1.0,
            meta: false,
            trg_id: 1,
            route_hint: 0,
        }),
    );

    let updated = processor.process(event, Some(&grid), 0, 0.5, 0.5);

    assert_eq!(updated.event.payload.ttl, 2);
    assert!((updated.event.payload.quality - 0.6).abs() < 1e-6);
}
