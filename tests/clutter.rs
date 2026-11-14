//! Clutter: it sits on a surface, never on the floor, and it follows
//! a piece that the player moves.

use floorplan::domain::furniture::{FurnitureKind, FurnitureState};
use floorplan::domain::model::Program;
use floorplan::domain::style::Style;
use floorplan::use_cases::clutter::clutter_boxes;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::materials::CLUTTER;

#[test]
fn clutter_sits_on_a_surface_not_the_floor() {
    let house = generate_house(42, Style::Modern);
    let boxes = clutter_boxes(&house.floors[0], 0.0);
    assert!(!boxes.is_empty(), "no clutter on a furnished house");
    for (b, m) in &boxes {
        assert_eq!(*m, CLUTTER);
        assert!(b.min[1] >= 0.0, "clutter below the floor: {b:?}");
    }
}

#[test]
fn every_living_space_gets_a_flat_rug() {
    let house = generate_house(42, Style::Modern);
    let ground = &house.floors[0];
    let rugs: Vec<_> = ground.rooms.iter()
        .filter(|r| matches!(r.program, Program::Living | Program::Dining | Program::Bedroom | Program::Hall | Program::Entry))
        .filter(|r| clutter_boxes(ground, 0.0).iter().any(|(b, _)| {
            b.max[1] - b.min[1] <= 0.03
                && b.min[0] >= r.rect.x as f32
                && b.max[0] <= r.rect.right() as f32
        }))
        .collect();
    assert!(!rugs.is_empty(), "no rugs in the living spaces");
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
