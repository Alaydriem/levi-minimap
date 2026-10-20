use std::collections::VecDeque;

use crate::structs::ui::UiEvent;

// input from the game thread for the next UI frame; at capacity the oldest event is dropped
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiEventQueue {
    capacity: usize,
    events: VecDeque<UiEvent>,
    dropped: u64,
}

impl UiEventQueue {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self { capacity, events: VecDeque::with_capacity(capacity), dropped: 0 }
    }

    pub fn push(&mut self, event: UiEvent) {
        if self.events.len() == self.capacity {
            self.events.pop_front();
            self.dropped += 1;
        }
        self.events.push_back(event);
    }

    pub fn drain_into(&mut self, out: &mut Vec<UiEvent>) {
        out.extend(self.events.drain(..));
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    // events dropped because the queue was full, since it was made
    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}
