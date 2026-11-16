use floorplan::domain::furniture::{Facing, Furniture, FurnitureKind, FurnitureState};
use floorplan::domain::geometry::Rect;
use floorplan::domain::interaction::Interaction;
use floorplan::domain::model::OpeningKind;
use floorplan::domain::style::Style;
use floorplan::use_cases::furnish_clear::blocks_door;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::interact::apply;

/// A generated house is empty, so a move test uses a manual
/// fixture: a chair and a table in the first room.
fn house_with_chair_and_table() -> floorplan::domain::model::House {
    let mut house = generate_house(42, Style::Modern);
    let room = house.floors[0].rooms[0].clone();
    house.floors[0].furniture.push(Furniture::new(
        1,
        FurnitureKind::Chair,
        room.id,
        Rect::new(room.rect.x + 0.5, room.rect.y + 0.5, 0.45, 0.45),
        Facing::North,
    ));
    house.floors[0].furniture.push(Furniture::new(
        2,
        FurnitureKind::Table,
        room.id,
        Rect::new(room.rect.x + 2.0, room.rect.y + 0.5, 1.2, 0.8),
        Facing::North,
    ));
    house
}

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
    let mut house = house_with_chair_and_table();
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
    let mut house = house_with_chair_and_table();
    let id = first_chair_id(&house);
    let table = house.floors[0]
        .furniture
        .iter()
        .find(|p| p.kind == FurnitureKind::Table)
        .expect("no table");
    let to = Rect::new(table.rect.x, table.rect.y, 0.45, 0.45);
    assert!(!apply(&mut house, Interaction::MoveFurniture { furniture_id: id, to }));
}

/// A generated house is empty, so interactions that need a piece
/// use a manual fixture: a lamp and a bed in the first room.
fn house_with_pieces() -> floorplan::domain::model::House {
    let mut house = generate_house(42, Style::Modern);
    let room = house.floors[0].rooms[0].clone();
    house.floors[0].furniture.push(Furniture::new(
        1,
        FurnitureKind::Lamp,
        room.id,
        Rect::new(room.rect.x + 0.5, room.rect.y + 0.5, 0.4, 0.4),
        Facing::North,
    ));
    house.floors[0].furniture.push(Furniture::new(
        2,
        FurnitureKind::Bed,
        room.id,
        Rect::new(room.rect.x + 2.0, room.rect.y + 0.5, 1.6, 2.1),
        Facing::North,
    ));
    house
}

fn first_door(house: &floorplan::domain::model::House) -> u32 {
    house.floors[0]
        .openings
        .iter()
        .find(|o| o.kind == OpeningKind::Door)
        .expect("no door")
        .id
}

fn first_lamp(house: &floorplan::domain::model::House) -> u32 {
    house.floors[0]
        .furniture
        .iter()
        .find(|p| p.kind == FurnitureKind::Lamp)
        .expect("no lamp")
        .id
}

#[test]
fn opening_a_door_marks_it_open() {
    let mut house = generate_house(42, Style::Modern);
    let id = first_door(&house);
    assert!(apply(&mut house, Interaction::OpenDoor { door_id: id }));
    assert!(house.floors[0].open_doors.contains(&id));
    assert!(apply(&mut house, Interaction::CloseDoor { door_id: id }));
    assert!(!house.floors[0].open_doors.contains(&id));
}

#[test]
fn opening_a_missing_door_is_refused() {
    let mut house = generate_house(42, Style::Modern);
    assert!(!apply(&mut house, Interaction::OpenDoor { door_id: 99999 }));
}

#[test]
fn toggling_a_lamp_flips_it_on_and_off() {
    let mut house = house_with_pieces();
    let id = first_lamp(&house);
    assert!(apply(&mut house, Interaction::ToggleLamp { furniture_id: id }));
    let lamp = house.floors[0].furniture.iter().find(|p| p.id == id).unwrap();
    assert_eq!(lamp.state, FurnitureState::On);
    assert!(apply(&mut house, Interaction::ToggleLamp { furniture_id: id }));
    let lamp = house.floors[0].furniture.iter().find(|p| p.id == id).unwrap();
    assert_eq!(lamp.state, FurnitureState::Off);
}

#[test]
fn toggling_a_bed_is_refused() {
    let mut house = house_with_pieces();
    let bed = house.floors[0]
        .furniture
        .iter()
        .find(|p| p.kind == FurnitureKind::Bed)
        .expect("no bed")
        .id;
    assert!(!apply(&mut house, Interaction::ToggleLamp { furniture_id: bed }));
}

#[test]
fn moving_a_bed_is_refused() {
    let mut house = house_with_pieces();
    let bed = house.floors[0]
        .furniture
        .iter()
        .find(|p| p.kind == FurnitureKind::Bed)
        .expect("no bed")
        .id;
    let to = Rect::new(1.0, 1.0, 1.6, 2.1);
    assert!(!apply(&mut house, Interaction::MoveFurniture { furniture_id: bed, to }));
}
