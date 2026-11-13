//! The fear engine's dread coupling: the dial, the rungs, and the
//! systems that move a house's dread.
//!
//! Pure functions over `DreadState`. The shell supplies the systems
//! (darkness, time in the dark, tasks done, light relief) as data and
//! the wrong-detail candidates; this module decides what the dial
//! allows and writes the event log. Task generation and wrong-detail
//! generation live in `haunt_tasks` and `haunt_events`. It never
//! draws and never reads the clock.

use crate::domain::haunt::{DreadRung, DreadState, Event, Task};
use crate::frameworks_drivers::rng::Rng;

/// The systems that move dread, as data. All values are per-step.
pub struct Systems {
    /// Darkness the player is standing in, 0..1.
    pub darkness: f64,
    /// Minutes spent in one dark room.
    pub dark_minutes: f64,
    /// Tasks completed this step.
    pub tasks_done: u32,
    /// Relief from light the player is standing in, 0..1.
    pub light_relief: f64,
}

/// A fresh house at the given dial: dread 0, no events.
pub fn new_dread(dial: u8) -> DreadState {
    DreadState { level: 0, dial, rung: DreadRung::Calm, event_log: Vec::new() }
}

/// The highest rung the dial lets the engine spend. Three bands of
/// dial, so a low dial can never buy an event-rung detail.
pub fn allowed_rung(dial: u8) -> DreadRung {
    match dial {
        0 => DreadRung::Calm,
        1..=33 => DreadRung::Unease,
        34..=66 => DreadRung::Wrong,
        _ => DreadRung::Event,
    }
}

/// True when the current rung is at or below the dial's cap.
pub fn may_spend_event(state: &DreadState) -> bool {
    state.dial > 0 && state.rung <= allowed_rung(state.dial)
}

/// Advances dread by one step. At dial 0 the sandbox is neutral:
/// dread stays at 0 and nothing is logged.
pub fn step(state: &mut DreadState, systems: &Systems, candidates: &[Event]) {
    if state.dial == 0 {
        state.level = 0;
        state.rung = DreadRung::Calm;
        return;
    }
    let spent = if may_spend_event(state) { candidates.len() } else { 0 };
    state.event_log.extend_from_slice(&candidates[..spent]);
    let gain = systems.darkness * 10.0
        + systems.dark_minutes * 0.5
        - systems.light_relief * 8.0
        - systems.tasks_done as f64 * 6.0
        + spent as f64 * 3.0;
    state.level = (state.level as f64 + gain).clamp(0.0, 100.0) as u8;
    state.rung = DreadRung::from_level(state.level);
}

/// Marks a task done and grounds the player by six points. Returns
/// false for an unknown or already-done task; a task completes once.
pub fn complete_task(state: &mut DreadState, tasks: &mut [Task], id: u32) -> bool {
    let Some(task) = tasks.iter_mut().find(|t| t.id == id) else {
        return false;
    };
    if task.done {
        return false;
    }
    task.done = true;
    state.level = state.level.saturating_sub(6);
    state.rung = DreadRung::from_level(state.level);
    true
}

/// Picks one element, or `None` when the list is empty. Shared by the
/// task and event generators so an empty house never panics.
pub(crate) fn pick<T: Copy>(items: &[T], rng: &mut Rng) -> Option<T> {
    if items.is_empty() { None } else { Some(*rng.pick(items)) }
}
