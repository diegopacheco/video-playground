use ssai::tracking::{AdEvent, Beacon, BeaconSink, MemoryBeaconSink, events_for_segment};

fn all_events(durations: &[f64]) -> Vec<Vec<AdEvent>> {
    (0..durations.len()).map(|i| events_for_segment(durations, i)).collect()
}

#[test]
fn quartiles_fire_on_the_segment_that_covers_each_point() {
    let events = all_events(&[2.0; 5]);
    assert_eq!(events[0], vec![AdEvent::Impression]);
    assert_eq!(events[1], vec![AdEvent::FirstQuartile]);
    assert_eq!(events[2], vec![AdEvent::Midpoint]);
    assert_eq!(events[3], vec![AdEvent::ThirdQuartile]);
    assert_eq!(events[4], vec![AdEvent::Complete]);
}

#[test]
fn every_event_fires_exactly_once_whatever_the_segmentation() {
    for durations in [vec![2.0, 2.0], vec![6.0], vec![2.0, 2.0, 2.0], vec![1.5, 2.5, 2.0, 2.0]] {
        let flat: Vec<AdEvent> = all_events(&durations).into_iter().flatten().collect();
        assert_eq!(
            flat,
            vec![AdEvent::Impression, AdEvent::FirstQuartile, AdEvent::Midpoint, AdEvent::ThirdQuartile, AdEvent::Complete],
            "advertisers bill on these counts, a duplicate or a gap is a billing error: {durations:?}"
        );
    }
}

#[test]
fn out_of_range_segment_fires_nothing() {
    assert!(events_for_segment(&[2.0], 3).is_empty());
}

fn beacon(session: &str, event: AdEvent) -> Beacon {
    Beacon { seq: 0, user: "alice".into(), session: session.into(), break_id: "b1".into(), creative: "acme".into(), event, at_ms: 0 }
}

#[test]
fn player_retries_do_not_double_count_beacons() {
    let sink = MemoryBeaconSink::default();
    assert!(sink.record(beacon("s1", AdEvent::Impression)).unwrap());
    assert!(!sink.record(beacon("s1", AdEvent::Impression)).unwrap());
    assert!(sink.record(beacon("s2", AdEvent::Impression)).unwrap());
    assert_eq!(sink.list(None).unwrap().len(), 2);
    assert_eq!(sink.list(Some("s1")).unwrap().len(), 1);
    sink.clear().unwrap();
    assert!(sink.list(None).unwrap().is_empty());
}
