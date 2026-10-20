use minimap_core::structs::ui::{UiEvent, UiEventQueue};

#[test]
fn the_queue_drops_the_oldest_past_its_capacity_and_counts_them() {
    let mut queue = UiEventQueue::new(4);

    for c in ['a', 'b', 'c', 'd', 'e', 'f'] {
        queue.push(UiEvent::Text(c));
    }
    let mut out = Vec::new();
    queue.drain_into(&mut out);

    let expected: Vec<UiEvent> = ['c', 'd', 'e', 'f'].into_iter().map(UiEvent::Text).collect();
    assert_eq!(out, expected);
    assert_eq!(queue.dropped(), 2);
    assert!(queue.is_empty());
}

#[test]
fn a_zero_capacity_still_holds_the_newest_event() {
    let mut queue = UiEventQueue::new(0);

    queue.push(UiEvent::Wheel(120));
    queue.push(UiEvent::Wheel(-120));

    assert_eq!(queue.len(), 1);
    let mut out = Vec::new();
    queue.drain_into(&mut out);
    assert_eq!(out, vec![UiEvent::Wheel(-120)]);
}
