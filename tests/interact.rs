use floorplan::domain::furniture::{FurnitureKind, FurnitureState};
use floorplan::domain::geometry::Rect;
use floorplan::domain::interaction::Interaction;
use floorplan::domain::model::OpeningKind;
use floorplan::domain::style::Style;
use floorplan::use_cases::furnish_clear::blocks_door;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::interact::apply;

fn first_chair_id(house: &floorplan::domain::model::House) -> u32 {
    house.floors[0]
        .furniture
        .iter()
        .find(|p| p.kind == FurnitureKind::Chair)
        .expect("no chair")
        .id
}

#[test]
fn moving_a_chair_to_a_free_spot_works() {
    let mut house = generate_house(42, Style::Modern);
    let id = first_chair_id(&house);
    let chair = house.floors[0].furniture.iter().find(|p| p.id == id).unwrap();
    let room = house.floors[0].rooms.iter().find(|r| r.id == chair.room_id).unwrap();
    let size = (chair.rect.w, chair.rect.d);
    let others: Vec<Rect> = house.floors[0]
        .furniture
        .iter()
        .filter(|p| p.room_id == room.id && p.id != id)
        .map(|p| p.rect)
        .collect();
    let doors: Vec<&floorplan::domain::model::Opening> = house.floors[0]
        .openings
        .iter()
        .filter(|o| o.kind != OpeningKind::Window && (o.from == room.id || o.to == Some(room.id)))
        .collect();

    // Find a spot that is inside the room, overlaps nothing, and is
    // not in a door swing; the plan is dense, so scan rather than
    // assume a corner is free.
    let spot = (0..30)
        .flat_map(|i| (0..30).map(move |j| (i, j)))
        .map(|(i, j)| (room.rect.x + 0.1 + i as f64 * 0.1, room.rect.y + 0.1 + j as f64 * 0.1))
        .find(|&(x, y)| {
            let to = Rect::new(x, y, size.0, size.1);
            to.right() <= room.rect.right() + 1e-9
                && to.bottom() <= room.rect.bottom() + 1e-9
                && !others.iter().any(|r| r.overlaps(&to))
                && !doors.iter().any(|o| blocks_door(room, &to, o))
        })
        .expect("the room has no free spot for a chair");

    let to = Rect::new(spot.0, spot.1, size.0, size.1);
    let ok = apply(&mut house, Interaction::MoveFurniture { furniture_id: id, to });
    assert!(ok, "a free spot should accept the chair");
    let chair = house.floors[0].furniture.iter().find(|p| p.id == id).unwrap();
    assert_eq!(chair.state, FurnitureState::Moved { x: to.x, y: to.y });
}

#[test]
fn moving_a_chair_onto_another_piece_is_refused() {
    let mut house = generate_house(42, Style::Modern);
    let id = first_chair_id(&house);
    let table = house.floors[0]
        .furniture
        .iter()
        .find(|p| p.kind == FurnitureKind::Table)
        .expect("no table");
    let to = Rect::new(table.rect.x, table.rect.y, 0.45, 0.45);
    assert!(!apply(&mut house, Interaction::MoveFurniture { furniture_id: id, to }));
}
