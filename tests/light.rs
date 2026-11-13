//! Light play integration tests: every room has a switch and a
//! working light, lamps mirror their furniture, and the porch has a
//! light when the house has a porch.

use floorplan::domain::furniture::{FurnitureKind, FurnitureState};
use floorplan::domain::interaction::Interaction;
use floorplan::domain::light::{LightKind, LightSource, LightState};
use floorplan::domain::model::House;
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::interact::apply;
use floorplan::use_cases::light::toggle;

fn house42() -> House {
    generate_house(42, Style::Modern)
}

fn room_ids(house: &House) -> Vec<u32> {
    house.floors.iter().flat_map(|f| f.rooms.iter().map(|r| r.id)).collect()
}

fn ceiling_id(room_id: u32) -> u32 {
    1000 + room_id
}

#[test]
fn every_room_has_a_ceiling_light_that_switches_on() {
    for seed in 0..5u32 {
        for style in Style::all() {
            let mut house = generate_house(seed, style);
            let ids = room_ids(&house);
            assert!(!ids.is_empty(), "{style:?} has no rooms");
            for room_id in ids {
                let light_id = ceiling_id(room_id);
                let light = house
                    .lights
                    .iter()
                    .find(|l| l.id == light_id)
                    .unwrap_or_else(|| panic!("seed {seed} {style:?}: room {room_id} has no ceiling light"));
                assert_eq!(light.kind, LightKind::Ceiling);
                assert_eq!(light.room_id, Some(room_id));
                assert_ne!(light.state, LightState::Broken, "a room light is broken");
                assert!(apply(&mut house, Interaction::ToggleLight { light_id }));
                let light = house.lights.iter().find(|l| l.id == light_id).unwrap();
                assert_eq!(light.state, LightState::On, "the switch did not light the room");
            }
        }
    }
}

#[test]
fn no_room_is_unwinnably_dark() {
    for seed in 0..5u32 {
        for style in Style::all() {
            let house = generate_house(seed, style);
            assert!(
                house.lights.iter().any(|l| l.kind == LightKind::Flashlight && l.state == LightState::On),
                "seed {seed} {style:?}: no working flashlight"
            );
            for room_id in room_ids(&house) {
                let light = house.lights.iter().find(|l| l.id == ceiling_id(room_id)).unwrap();
                assert_ne!(light.state, LightState::Broken, "seed {seed} {style:?}: room {room_id} is dark");
            }
        }
    }
}

#[test]
fn a_lamp_light_mirrors_its_furniture() {
    let house = house42();
    let lamps: Vec<_> = house
        .floors
        .iter()
        .flat_map(|f| f.furniture.iter())
        .filter(|p| p.kind == FurnitureKind::Lamp)
        .collect();
    assert!(!lamps.is_empty(), "the house has no lamps");
    for piece in lamps {
        let light = house
            .lights
            .iter()
            .find(|l| l.id == 2000 + piece.id)
            .expect("a lamp has no light");
        assert_eq!(light.room_id, Some(piece.room_id));
        assert_eq!(light.state == LightState::On, piece.state == FurnitureState::On);
    }
}

#[test]
fn toggling_a_lamp_light_updates_the_furniture() {
    let mut house = house42();
    let piece = house
        .floors
        .iter()
        .flat_map(|f| f.furniture.iter())
        .find(|p| p.kind == FurnitureKind::Lamp)
        .expect("no lamp");
    let (furniture_id, before) = (piece.id, piece.state);
    let light_id = 2000 + furniture_id;

    assert!(apply(&mut house, Interaction::ToggleLight { light_id }));

    let after = house
        .floors
        .iter()
        .flat_map(|f| f.furniture.iter())
        .find(|p| p.id == furniture_id)
        .unwrap()
        .state;
    assert_ne!(after, before, "the furniture did not follow the switch");
    let light = house.lights.iter().find(|l| l.id == light_id).unwrap();
    assert_eq!(light.state == LightState::On, after == FurnitureState::On);
}

#[test]
fn toggling_a_broken_light_is_refused() {
    let mut lights = vec![LightSource {
        id: 7,
        kind: LightKind::Ceiling,
        room_id: Some(1),
        x: 0.0,
        y: 0.0,
        radius: 6.0,
        state: LightState::Broken,
    }];
    assert!(!toggle(&mut lights, 7));
    assert_eq!(lights[0].state, LightState::Broken);
    assert!(!toggle(&mut lights, 999), "an unknown light is refused");
}

#[test]
fn a_porch_light_exists_when_the_house_has_a_porch() {
    let with_porch = Style::all()
        .iter()
        .map(|style| generate_house(42, *style))
        .find(|house| house.footprint.porch.is_some());
    let house = with_porch.expect("no style has a porch");
    assert!(house.lights.iter().any(|l| l.kind == LightKind::Porch));
}
