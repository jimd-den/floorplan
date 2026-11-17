//! Doors: one per backbone link, the entry door, and the rule that
//! a floor has at most one double door, on its living/dining opening.

use floorplan::domain::geometry::{Rect, shared_edge};
use floorplan::domain::model::{OpeningKind, Program, Room};
use floorplan::domain::skeleton::Skeleton;
use floorplan::frameworks_drivers::rng::Rng;
use floorplan::use_cases::bsp::subdivide;
use floorplan::use_cases::door_width::{interior_door_width, is_double_door};
use floorplan::use_cases::doors::place_doors;
use floorplan::use_cases::programs::assign_programs;
use floorplan::use_cases::skeleton::build_skeleton;

fn room(id: u32, program: Program, rect: Rect) -> Room {
    Room { id, program, rect }
}

/// Two rooms from one vertical split of a 10x6 rect, and the
/// skeleton built from them.
fn two_rooms() -> (Vec<Room>, Skeleton) {
    let mut rng = Rng::new(4);
    let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 10.0, 6.0), 2);
    let rooms = vec![
        room(0, Program::Living, sub.leaves[0]),
        room(1, Program::Dining, sub.leaves[1]),
    ];
    let skeleton = build_skeleton(&rooms, &sub.splits, &[], 0);
    (rooms, skeleton)
}

#[test]
fn a_split_gets_exactly_one_door() {
    let (rooms, skeleton) = two_rooms();
    let mut rng = Rng::new(4);
    let doors = place_doors(&mut rng, &rooms, &skeleton, 0);
    assert_eq!(doors.iter().filter(|d| d.kind == OpeningKind::Door).count(), 1);
}

#[test]
fn the_door_sits_on_the_shared_wall() {
    let (rooms, skeleton) = two_rooms();
    let mut rng = Rng::new(4);
    let doors = place_doors(&mut rng, &rooms, &skeleton, 0);
    let door = doors.iter().find(|d| d.kind == OpeningKind::Door).unwrap();
    let shared = shared_edge(&rooms[0].rect, &rooms[1].rect).unwrap();
    assert!((door.x - shared.x1).abs() < 1e-9, "door x must sit on the shared wall");
    assert!(door.y >= shared.y1 - 1e-9 && door.y <= shared.y2 + 1e-9);
    assert!(door.width <= shared.len() - 0.5);
}

#[test]
fn a_wide_opening_between_main_rooms_gets_a_double_door() {
    let (rooms, skeleton) = two_rooms();
    let mut rng = Rng::new(4);
    let doors = place_doors(&mut rng, &rooms, &skeleton, 0);
    let door = doors.iter().find(|d| d.kind == OpeningKind::Door).unwrap();
    let shared = shared_edge(&rooms[0].rect, &rooms[1].rect).unwrap();
    assert_eq!(door.width, interior_door_width(shared.len(), false, true));
    assert_eq!(door.width, 1.2, "a 6 m opening takes two 0.6 m leaves");
    assert!(is_double_door(door.width), "a 6 m opening must take a double door, not {}", door.width);
}

#[test]
fn a_floor_has_at_most_one_double_door() {
    let mut rng = Rng::new(5);
    let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 12.0, 9.0), 6);
    let programs = assign_programs(&sub.leaves, 0, false);
    let rooms: Vec<Room> = sub
        .leaves
        .iter()
        .enumerate()
        .map(|(i, r)| room(i as u32, programs[i], *r))
        .collect();
    let entry = rooms.iter().find(|r| r.program == Program::Entry).unwrap().id;
    let skeleton = build_skeleton(&rooms, &sub.splits, &[], entry);
    let doors = place_doors(&mut rng, &rooms, &skeleton, entry);
    let doubles = doors.iter().filter(|d| is_double_door(d.width)).count();
    assert!(doubles <= 1, "a house has at most one double door, not {doubles}");
}

#[test]
fn the_entry_door_is_a_single_nine_tenths_door() {
    let (rooms, skeleton) = two_rooms();
    let mut rng = Rng::new(4);
    let doors = place_doors(&mut rng, &rooms, &skeleton, 0);
    let entry = doors.iter().find(|d| d.kind == OpeningKind::EntryDoor).unwrap();
    assert_eq!(entry.width, 0.9, "a 5 m front does not make a wide front door");
    assert!(!is_double_door(entry.width), "the front door is a single, not {}", entry.width);
    assert_eq!(entry.from, 0);
    assert_eq!(entry.to, None);
}

#[test]
fn every_room_is_reachable_from_the_entry() {
    let mut rng = Rng::new(5);
    let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 12.0, 9.0), 6);
    let programs = assign_programs(&sub.leaves, 0, false);
    let rooms: Vec<Room> = sub
        .leaves
        .iter()
        .enumerate()
        .map(|(i, r)| room(i as u32, programs[i], *r))
        .collect();
    let entry = rooms.iter().find(|r| r.program == Program::Entry).unwrap().id;
    let skeleton = build_skeleton(&rooms, &sub.splits, &[], entry);
    let doors = place_doors(&mut rng, &rooms, &skeleton, entry);
    let mut seen = vec![entry];
    let mut stack = vec![entry];
    while let Some(cur) = stack.pop() {
        for d in doors.iter().filter(|d| d.kind == OpeningKind::Door) {
            let next = if d.from == cur { d.to } else if d.to == Some(cur) { Some(d.from) } else { None };
            if let Some(n) = next
                && !seen.contains(&n)
            {
                seen.push(n);
                stack.push(n);
            }
        }
    }
    assert_eq!(seen.len(), rooms.len(), "unreachable rooms remain");
}
