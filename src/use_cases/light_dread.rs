//! light_dread: the one place light becomes fear.
//!
//! A room's lit fraction is the strongest of its own lights and the
//! daylight its windows admit; the carried flashlight can stand in
//! for a missing fixture. Darkness and light relief are complements,
//! so the fear engine sees one number, not two guesses. A flickering
//! light is offered as a wrong-detail candidate, never as a render
//! bug. No clock lives here; the shell supplies the tick.

use crate::domain::haunt::{Event, EventKind};
use crate::domain::light::{Daylight, LightKind, LightSource, LightState};
use crate::use_cases::haunt::Systems;

/// A carried flashlight lights a room less than a ceiling fixture.
const FLASHLIGHT_LEVEL: f64 = 0.8;

/// The lights in play this frame.
pub struct Lighting<'a> {
    pub lights: &'a [LightSource],
    pub daylight: Daylight,
    pub tick: u64,
}

/// The player's light state this frame: the room they stand in, the
/// flashlight they carry, and the dread systems only they can supply.
pub struct PlayerLight {
    pub room_id: u32,
    pub carrying_flashlight: bool,
    pub dark_minutes: f64,
    pub tasks_done: u32,
}

/// The lit fraction of a room, 0..1, from its own lights and the
/// daylight its windows admit. The carried flashlight is separate.
pub fn room_light(room_id: u32, lighting: &Lighting) -> f64 {
    let mut level = 0.0f64;
    for light in lighting.lights {
        if light.room_id != Some(room_id) {
            continue;
        }
        level = level.max(contribution(light, &lighting.daylight, lighting.tick));
    }
    level.clamp(0.0, 1.0)
}

/// How much one light contributes right now. A window follows the
/// sun; a flicker light is on half the time, deterministically.
fn contribution(light: &LightSource, daylight: &Daylight, tick: u64) -> f64 {
    match light.state {
        LightState::On => match light.kind {
            LightKind::Window => daylight.elevation().max(0.0),
            _ => 1.0,
        },
        LightState::Flicker => {
            if (tick / 4).is_multiple_of(2) { 0.5 } else { 0.0 }
        }
        LightState::Off | LightState::Broken => 0.0,
    }
}

/// The flashlight's contribution, or 0 when it is off or absent.
pub fn flashlight(lights: &[LightSource]) -> f64 {
    let on = lights
        .iter()
        .any(|l| l.kind == LightKind::Flashlight && l.state == LightState::On);
    if on { FLASHLIGHT_LEVEL } else { 0.0 }
}

/// The dread systems for the room the player stands in: darkness and
/// light relief are complements, so a lit room grounds the player.
pub fn systems_for(player: &PlayerLight, lighting: &Lighting) -> Systems {
    let fixed = room_light(player.room_id, lighting);
    let carried = if player.carrying_flashlight {
        flashlight(lighting.lights)
    } else {
        0.0
    };
    let level = fixed.max(carried).clamp(0.0, 1.0);
    Systems {
        darkness: 1.0 - level,
        dark_minutes: player.dark_minutes,
        tasks_done: player.tasks_done,
        light_relief: level,
    }
}

/// A flickering light in the room is a dread event, not a render bug.
pub fn flicker_event(room_id: u32, lights: &[LightSource]) -> Option<Event> {
    let flickering = lights
        .iter()
        .any(|l| l.room_id == Some(room_id) && l.state == LightState::Flicker);
    flickering.then_some(Event { kind: EventKind::LightFlicker, target: room_id })
}
