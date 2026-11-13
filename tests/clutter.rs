//! Clutter: it sits on a surface, never on the floor, and it follows
//! a piece that the player moves.

use floorplan::domain::furniture::{FurnitureKind, FurnitureState};
use floorplan::domain::style::Style;
use floorplan::use_cases::clutter::clutter_boxes;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::materials::CLUTTER;

#[test]
fn clutter_sits_on_a_surface_not_the_floor() {
    let house = generate_house(42, Style::Modern);
    let boxes = clutter_boxes(&house.floors[0], 0.0);
    assert!(!boxes.is_empty(), "no clutter on a furnished house");
    for (b, m) in boxes {
        assert_eq!(m, CLUTTER);
        assert!(b.min[1] >= 0.45, "clutter on the floor: {b:?}");
    }
}

#[test]
fn clutter_follows_a_moved_piece() {
    let mut house = generate_house(42, Style::Modern);
    let piece = house.floors[0]
        .furniture
        .iter_mut()
        .find(|p| p.kind == FurnitureKind::Table)
        .expect("no table in seed 42");
    piece.state = FurnitureState::Moved { x: 1.0, y: 1.0 };
    let boxes = clutter_boxes(&house.floors[0], 0.0);
    assert!(
        boxes.iter().any(|(b, _)| (b.min[0] - 1.15).abs() < 0.01 && (b.min[2] - 1.15).abs() < 0.01),
        "clutter did not follow the table"
    );
}
