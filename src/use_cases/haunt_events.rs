//! Seed-derived wrong details: small, deniable events naming real
//! doors, chairs, lamps, and rooms. Dial 0 is silent.
//!
//! The dread coupling that decides whether a detail is spent lives in
//! `haunt`; this module only proposes candidates from the house.

use crate::domain::furniture::FurnitureKind;
use crate::domain::haunt::{Event, EventKind};
use crate::domain::model::{House, OpeningKind};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::haunt::pick;

/// The seed-derived wrong details for a house at a dial. Dial 0 is
/// silent; the count rises with the dial. Every target is a real
/// door, chair, lamp, or room in this house.
pub fn wrong_details(house: &House, rng: &mut Rng, dial: u8) -> Vec<Event> {
    let count = match dial {
        0 => 0,
        1..=33 => 1,
        34..=66 => 2,
        _ => 3,
    };
    let mut candidates = Vec::new();
    if let Some(id) = pick(&door_ids(house), rng) {
        candidates.push(Event { kind: EventKind::DoorAjar, target: id });
    }
    if let Some(id) = pick(&furniture_ids(house, FurnitureKind::Chair), rng) {
        candidates.push(Event { kind: EventKind::ChairMoved, target: id });
    }
    if let Some(id) = pick(&furniture_ids(house, FurnitureKind::Lamp), rng) {
        candidates.push(Event { kind: EventKind::LampOn, target: id });
    }
    if let Some(id) = pick(&room_ids(house), rng) {
        candidates.push(Event { kind: EventKind::LightFlicker, target: id });
    }
    candidates.truncate(count);
    candidates
}

fn door_ids(house: &House) -> Vec<u32> {
    house
        .floors
        .iter()
        .flat_map(|f| f.openings.iter())
        .filter(|o| o.kind != OpeningKind::Window)
        .map(|o| o.id)
        .collect()
}

fn furniture_ids(house: &House, kind: FurnitureKind) -> Vec<u32> {
    house
        .floors
        .iter()
        .flat_map(|f| f.furniture.iter())
        .filter(|p| p.kind == kind)
        .map(|p| p.id)
        .collect()
}

fn room_ids(house: &House) -> Vec<u32> {
    house.floors.iter().flat_map(|f| f.rooms.iter()).map(|r| r.id).collect()
}
