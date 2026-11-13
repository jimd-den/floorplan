//! light: derive a house's light sources and switch them.
//!
//! Every room gets a ceiling light and every lamp a light of its own;
//! windows admit daylight by sun position and the flashlight is
//! carried. The coupling to dread lives in `light_dread`, in one
//! place, so "too dark" and "too scary" are never confused (SRD-001
//! FR-25). No clock and no rendering live here.

use crate::domain::furniture::{FurnitureKind, FurnitureState};
use crate::domain::light::{LightKind, LightSource, LightState};
use crate::domain::model::{House, OpeningKind};

/// The carried flashlight's id. It is the player's, not the house's.
pub const FLASHLIGHT_ID: u32 = 1;
/// The porch light's id, when the house has a porch.
pub const PORCH_ID: u32 = 2;
/// Ceiling light ids start here, one per room.
const CEILING_BASE: u32 = 1000;
/// Lamp light ids start here, one per lamp piece.
const LAMP_BASE: u32 = 2000;
/// Window light ids start here, one per room with a window.
const WINDOW_BASE: u32 = 3000;

/// Every light a house owns: a ceiling per room, a light per lamp, a
/// window per room with one, a porch light when there is a porch,
/// and the carried flashlight. Ceiling and porch lights start off,
/// lamps mirror their furniture, windows are open to the sky.
pub fn lights_for_house(house: &House) -> Vec<LightSource> {
    let mut lights = vec![LightSource {
        id: FLASHLIGHT_ID,
        kind: LightKind::Flashlight,
        room_id: None,
        x: 0.0,
        y: 0.0,
        radius: 12.0,
        state: LightState::On,
    }];
    for floor in &house.floors {
        for room in &floor.rooms {
            let (x, y) = room.rect.center();
            lights.push(LightSource {
                id: CEILING_BASE + room.id,
                kind: LightKind::Ceiling,
                room_id: Some(room.id),
                x,
                y,
                radius: 6.0,
                state: LightState::Off,
            });
            if has_window(floor, room.id) {
                lights.push(LightSource {
                    id: WINDOW_BASE + room.id,
                    kind: LightKind::Window,
                    room_id: Some(room.id),
                    x,
                    y,
                    radius: 3.0,
                    state: LightState::On,
                });
            }
        }
        for piece in &floor.furniture {
            if piece.kind != FurnitureKind::Lamp {
                continue;
            }
            let (x, y) = piece.rect.center();
            let state = if piece.state == FurnitureState::On {
                LightState::On
            } else {
                LightState::Off
            };
            lights.push(LightSource {
                id: LAMP_BASE + piece.id,
                kind: LightKind::Lamp,
                room_id: Some(piece.room_id),
                x,
                y,
                radius: 4.0,
                state,
            });
        }
    }
    if let Some(porch) = house.footprint.porch {
        let (x, y) = porch.center();
        lights.push(LightSource {
            id: PORCH_ID,
            kind: LightKind::Porch,
            room_id: None,
            x,
            y,
            radius: 5.0,
            state: LightState::Off,
        });
    }
    lights
}

/// True when the room has a window; only those admit daylight.
fn has_window(floor: &crate::domain::model::Floor, room_id: u32) -> bool {
    floor
        .openings
        .iter()
        .any(|o| o.kind == OpeningKind::Window && o.from == room_id)
}

/// Flips one light. A broken light is refused, not repaired.
pub fn toggle(lights: &mut [LightSource], light_id: u32) -> bool {
    let Some(light) = lights.iter_mut().find(|l| l.id == light_id) else {
        return false;
    };
    if light.state == LightState::Broken {
        return false;
    }
    light.state = light.state.toggle();
    true
}

/// Mirrors a lamp light's new state onto its furniture, so the mesh
/// and the lighting never disagree. Other light kinds are ignored.
pub fn sync_lamp(house: &mut House, light_id: u32) {
    let Some(light) = house
        .lights
        .iter()
        .find(|l| l.id == light_id && l.kind == LightKind::Lamp)
    else {
        return;
    };
    let state = if light.state == LightState::On {
        FurnitureState::On
    } else {
        FurnitureState::Off
    };
    let Some(furniture_id) = light_id.checked_sub(LAMP_BASE) else {
        return;
    };
    for floor in &mut house.floors {
        if let Some(piece) = floor
            .furniture
            .iter_mut()
            .find(|p| p.id == furniture_id && p.kind == FurnitureKind::Lamp)
        {
            piece.state = state;
        }
    }
}
