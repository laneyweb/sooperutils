//! Keyboard state tracking for macOS.
//!
//! Key-name resolution is intentionally disabled. The macOS Text Input
//! Services (TIS/TSM) APIs originally used here must be called on the main
//! thread; when called from the background event-tap thread, modern macOS
//! aborts the process with a `dispatch_assert_queue` trap (SIGTRAP /
//! "Trace/BPT trap"). Events still carry their `EventType` (e.g.
//! `KeyPress(Key::KeyA)`) — only `event.name` is left unset, which is all
//! this application needs.
#![allow(clippy::upper_case_acronyms)]
use crate::rdev::{EventType, KeyboardState};

pub struct Keyboard {
    shift: bool,
    caps_lock: bool,
}
impl Keyboard {
    pub fn new() -> Option<Keyboard> {
        Some(Keyboard {
            shift: false,
            caps_lock: false,
        })
    }
}

impl KeyboardState for Keyboard {
    fn add(&mut self, _event_type: &EventType) -> Option<String> {
        None
    }

    fn reset(&mut self) {
        self.shift = false;
        self.caps_lock = false;
    }
}
