// Hotkey state machine: handles double-tap arming for insta-delete.
// See docs/architecture.md "Hotkey State Machine".

use std::time::{Duration, Instant};

const ARM_WINDOW: Duration = Duration::from_secs(2);

#[derive(Default)]
pub enum InstaDeleteState {
    #[default]
    Idle,
    Armed { at: Instant },
}

pub enum Transition {
    Armed,
    Confirmed,
    Disarmed,
    Ignored,
}

impl InstaDeleteState {
    pub fn on_press(&mut self) -> Transition {
        let now = Instant::now();
        match *self {
            InstaDeleteState::Idle => {
                *self = InstaDeleteState::Armed { at: now };
                Transition::Armed
            }
            InstaDeleteState::Armed { at } if now.duration_since(at) <= ARM_WINDOW => {
                *self = InstaDeleteState::Idle;
                Transition::Confirmed
            }
            InstaDeleteState::Armed { .. } => {
                *self = InstaDeleteState::Armed { at: now };
                Transition::Armed
            }
        }
    }

    pub fn tick(&mut self) -> Transition {
        match *self {
            InstaDeleteState::Armed { at } if Instant::now().duration_since(at) > ARM_WINDOW => {
                *self = InstaDeleteState::Idle;
                Transition::Disarmed
            }
            _ => Transition::Ignored,
        }
    }
}
