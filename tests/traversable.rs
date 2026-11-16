//! Traversability: every room is walkable from every other room
//! through door gaps, over many seeds and all styles. This is the
//! geometric proof, not a graph count: the walk crosses real door
//! gaps in real walls, from the entry room (ground floor) or the
//! stair landing (upper floor).

use floorplan::domain::model::{Floor, Program};
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::traverse::is_traversable;

fn entry_room(floor: &Floor) -> u32 {
    floor
        .rooms
        .iter()
        .find(|r| r.program == Program::Entry)
        .or_else(|| floor.rooms.iter().find(|r| r.program == Program::Living))
        .map(|r| r.id)
        .unwrap_or(0)
}

#[test]
fn every_room_is_walkable_from_the_entry_for_100_seeds() {
    for seed in 0..100u32 {
        for style in Style::all() {
            let house = generate_house(seed, style);
            for (fi, floor) in house.floors.iter().enumerate() {
                let start = if fi == 0 {
                    entry_room(floor)
                } else {
                    house.stairs.map(|(_, b)| b).unwrap_or_else(|| entry_room(floor))
                };
                assert!(
                    is_traversable(floor, start),
                    "seed {seed} {style:?} floor {fi}: a room is not walkable from room {start}"
                );
            }
        }
    }
}
