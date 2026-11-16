//! Clearance tests for moving furniture: a candidate rect must sit
//! inside its room, overlap no placed piece, and block no door
//! swing and no window.

use floorplan::domain::furniture::FurnitureKind;
use floorplan::domain::geometry::{Axis, Rect};
use floorplan::domain::model::{Opening, OpeningKind, Program, Room};
use floorplan::use_cases::furnish_clear::{blocks_door, blocks_window, door_zone, fits};

fn room(rect: Rect) -> Room {
    Room { id: 0, program: Program::Bedroom, rect }
}

fn door(id: u32, x: f64, y: f64, axis: Axis) -> Opening {
    Opening { id, kind: OpeningKind::Door, x, y, axis, width: 0.9, from: 0, to: Some(1) }
}

fn window(id: u32, x: f64, y: f64, axis: Axis) -> Opening {
    Opening { id, kind: OpeningKind::Window, x, y, axis, width: 1.0, from: 0, to: None }
}

#[test]
fn a_piece_in_the_door_swing_does_not_fit() {
    let r = room(Rect::new(0.0, 0.0, 6.0, 5.0));
    let d = door(0, 3.0, 0.0, Axis::Horizontal);
    // The door swing zone is the door's width into the room.
    let blocking = Rect::new(2.6, 0.1, 0.8, 0.8);
    assert!(blocks_door(&r, &blocking, &d), "the rect stands in the swing");
    assert!(!fits(&r, FurnitureKind::Cabinet, &blocking, &[&d], &[], &[]));
    let clear = Rect::new(4.5, 0.1, 0.8, 0.8);
    assert!(!blocks_door(&r, &clear, &d), "the rect is clear of the swing");
    assert!(fits(&r, FurnitureKind::Cabinet, &clear, &[&d], &[], &[]));
}

#[test]
fn a_tall_piece_covering_a_window_does_not_fit() {
    let r = room(Rect::new(0.0, 0.0, 6.0, 5.0));
    let w = window(0, 3.0, 0.0, Axis::Horizontal);
    let covering = Rect::new(2.6, 0.0, 0.8, 0.5);
    assert!(blocks_window(&covering, &w), "the rect covers the window");
    assert!(!fits(&r, FurnitureKind::Cabinet, &covering, &[], &[&w], &[]));
    // A low piece sits below the sill and fits.
    assert!(fits(&r, FurnitureKind::Table, &covering, &[], &[&w], &[]));
}

#[test]
fn the_door_zone_is_one_sided() {
    let r = room(Rect::new(0.0, 0.0, 6.0, 5.0));
    let d = door(0, 3.0, 0.0, Axis::Horizontal);
    let zone = door_zone(&r, &d);
    assert!(zone.y >= 0.0, "zone leaks north of the wall: {zone:?}");
    assert!((zone.bottom() - 0.9).abs() < 1e-9, "zone should reach 0.9 m in: {zone:?}");
}
