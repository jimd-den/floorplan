//! Clearance and walkability tests for furniture placement.

use floorplan::domain::furniture::{Facing, Furniture, FurnitureKind};
use floorplan::domain::geometry::{Axis, Rect};
use floorplan::domain::model::{Floor, Opening, OpeningKind, Program, Room};
use floorplan::domain::style::Style;
use floorplan::frameworks_drivers::rng::Rng;
use floorplan::use_cases::furnish::furnish;
use floorplan::use_cases::furnish_clear::{blocks_door, blocks_window, door_zone, walkable};
use floorplan::use_cases::generate::generate_house;

fn room(rect: Rect) -> Room {
    Room { id: 0, program: Program::Bedroom, rect }
}

fn floor_with(doors: Vec<Opening>, windows: Vec<Opening>) -> Floor {
    Floor {
        level: 0,
        rooms: vec![room(Rect::new(0.0, 0.0, 6.0, 5.0))],
        openings: doors.into_iter().chain(windows).collect(),
        furniture: vec![],
        open_doors: vec![],
    }
}

fn door(id: u32, x: f64, y: f64, axis: Axis) -> Opening {
    Opening { id, kind: OpeningKind::Door, x, y, axis, width: 0.9, from: 0, to: Some(1) }
}

fn window(id: u32, x: f64, y: f64, axis: Axis) -> Opening {
    Opening { id, kind: OpeningKind::Window, x, y, axis, width: 1.0, from: 0, to: None }
}

#[test]
fn no_piece_blocks_the_door() {
    let floor = floor_with(vec![door(0, 3.0, 0.0, Axis::Horizontal)], vec![]);
    let mut rng = Rng::new(2);
    let pieces = furnish(&mut rng, &floor);
    for p in &pieces {
        assert!(!blocks_door(&floor.rooms[0], &p.rect, &floor.openings[0]), "{p:?} blocks the door");
    }
}

#[test]
fn no_piece_blocks_the_window() {
    let floor = floor_with(vec![], vec![window(0, 3.0, 0.0, Axis::Horizontal)]);
    let mut rng = Rng::new(3);
    let pieces = furnish(&mut rng, &floor);
    for p in &pieces {
        assert!(
            !p.kind.is_tall() || !blocks_window(&p.rect, &floor.openings[0]),
            "{p:?} blocks the window"
        );
    }
}

#[test]
fn the_door_zone_is_one_sided() {
    let r = room(Rect::new(0.0, 0.0, 6.0, 5.0));
    let d = door(0, 3.0, 0.0, Axis::Horizontal);
    let zone = door_zone(&r, &d);
    assert!(zone.y >= 0.0, "zone leaks north of the wall: {zone:?}");
    assert!((zone.bottom() - 0.9).abs() < 1e-9, "zone should reach 0.9 m in: {zone:?}");
}

#[test]
fn an_empty_room_is_walkable() {
    let r = room(Rect::new(0.0, 0.0, 4.0, 4.0));
    let d = door(0, 2.0, 0.0, Axis::Horizontal);
    assert!(walkable(&r, &[&d], &[]));
}

#[test]
fn a_piece_blocking_the_only_door_makes_the_room_unwalkable() {
    let r = room(Rect::new(0.0, 0.0, 4.0, 4.0));
    let d = door(0, 2.0, 0.0, Axis::Horizontal);
    let blocker = Furniture::new(1, FurnitureKind::Cabinet, 0, Rect::new(1.0, 0.2, 2.0, 0.6), Facing::South);
    assert!(!walkable(&r, &[&d], &[blocker]), "a cabinet in the doorway must block the room");
}

#[test]
fn furnished_rooms_are_walkable_from_every_door() {
    for seed in 0..20u32 {
        for style in Style::all() {
            let house = generate_house(seed, style);
            for floor in &house.floors {
                for room in &floor.rooms {
                    let doors: Vec<&Opening> = floor
                        .openings
                        .iter()
                        .filter(|o| o.kind != OpeningKind::Window && (o.from == room.id || o.to == Some(room.id)))
                        .collect();
                    let pieces: Vec<Furniture> =
                        floor.furniture.iter().filter(|p| p.room_id == room.id).cloned().collect();
                    assert!(
                        walkable(room, &doors, &pieces),
                        "seed {seed} {style:?} room {} is not walkable",
                        room.id
                    );
                }
            }
        }
    }
}
