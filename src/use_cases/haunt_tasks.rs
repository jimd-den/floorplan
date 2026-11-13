//! Routine, grounding tasks generated from a real house.
//!
//! Every target is a room or piece that exists in this house; a kind
//! with no target is skipped. The dread coupling is in `haunt`.

use crate::domain::furniture::FurnitureKind;
use crate::domain::haunt::{Task, TaskKind, TaskTarget};
use crate::domain::model::{House, OpeningKind, Program};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::haunt::pick;

/// Generates the routine tasks for a house. Every target is a room or
/// piece that exists in this house; a kind with no target is skipped.
pub fn generate_tasks(house: &House, rng: &mut Rng) -> Vec<Task> {
    let mut tasks = Vec::new();
    let mut next_id = 1;
    let mut add = |kind, target| {
        tasks.push(Task { id: next_id, kind, target, done: false });
        next_id += 1;
    };
    if let Some(id) = pick(&lamp_rooms(house), rng) {
        add(TaskKind::CheckLights, TaskTarget::Room(id));
    }
    if let Some(id) = pick(&service_rooms(house), rng) {
        add(TaskKind::RestorePower, TaskTarget::Room(id));
    }
    if let Some(id) = pick(&all_furniture(house), rng) {
        add(TaskKind::Catalog, TaskTarget::Furniture(id));
    }
    if let Some(id) = pick(&door_rooms(house), rng) {
        add(TaskKind::CloseDoors, TaskTarget::Room(id));
    }
    tasks
}

fn lamp_rooms(house: &House) -> Vec<u32> {
    let mut ids = Vec::new();
    for floor in &house.floors {
        for room in &floor.rooms {
            let lit = floor
                .furniture
                .iter()
                .any(|p| p.room_id == room.id && p.kind == FurnitureKind::Lamp);
            if lit {
                ids.push(room.id);
            }
        }
    }
    ids
}

fn service_rooms(house: &House) -> Vec<u32> {
    house
        .floors
        .iter()
        .flat_map(|f| f.rooms.iter())
        .filter(|r| matches!(r.program, Program::Hall | Program::Entry))
        .map(|r| r.id)
        .collect()
}

fn all_furniture(house: &House) -> Vec<u32> {
    house.floors.iter().flat_map(|f| f.furniture.iter()).map(|p| p.id).collect()
}

fn door_rooms(house: &House) -> Vec<u32> {
    let mut ids = Vec::new();
    for floor in &house.floors {
        for opening in &floor.openings {
            if opening.kind == OpeningKind::Window {
                continue;
            }
            for id in std::iter::once(opening.from).chain(opening.to) {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
    }
    ids
}
