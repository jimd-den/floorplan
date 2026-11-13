use floorplan::domain::furniture::{FurnitureKind, FurnitureState};
use floorplan::domain::geometry::Rect;
use floorplan::domain::interaction::Interaction;
use floorplan::domain::style::Style;
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
    let to = Rect::new(room.rect.x + 0.1, room.rect.y + 0.1, chair.rect.w, chair.rect.d);
    let ok = apply(&mut house, Interaction::MoveFurniture { furniture_id: id, to });
    assert!(ok, "a free corner should accept the chair");
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
