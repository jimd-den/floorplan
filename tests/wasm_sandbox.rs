//! WASM sandbox tests: home-fixing tasks, coupled M3->M5 dread steps,
//! and interactive house mesh updates across the WASM boundary.

use floorplan::adapters::wasm::{
    HauntStepInput, MESH_MAGIC, haunt_step_json, house_interact_mesh_bytes, house_mesh_bytes,
    house_tasks_json,
};
use floorplan::domain::furniture::FurnitureKind;
use floorplan::domain::model::OpeningKind;
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;

#[test]
fn house_tasks_json_names_real_rooms_and_furniture_for_home_fixing() {
    let bytes = house_tasks_json(42, 3).expect("valid style");
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("\"kind\":\"check_lights\""));
    assert!(text.contains("\"kind\":\"restore_power\""));
    assert!(text.contains("\"kind\":\"catalog\""));
    assert!(text.contains("\"kind\":\"close_doors\""));
    assert!(text.contains("\"done\":false"));
    assert!(house_tasks_json(42, 99).is_err());
}

#[test]
fn haunt_step_json_is_silent_at_dial_zero_and_active_at_high_dial() {
    let house = generate_house(42, Style::Modern);
    let room_id = house.floors[0].rooms[0].id;
    let calm = HauntStepInput {
        seed: 42,
        style_index: 3,
        dial: 0,
        level: 0,
        room_id,
        carrying_flashlight: false,
        daylight_minutes: 0,
        tick: 1,
        dark_minutes: 30.0,
        tasks_done: 0,
    };
    let text0 = String::from_utf8(haunt_step_json(calm).unwrap()).unwrap();
    assert!(text0.contains("\"level\":0"));
    assert!(text0.contains("\"rung\":\"calm\""));
    assert!(text0.contains("\"events\":[]"));

    let active = HauntStepInput { dial: 100, level: 20, ..calm };
    let text1 = String::from_utf8(haunt_step_json(active).unwrap()).unwrap();
    assert!(!text1.contains("\"level\":0"), "dread must rise in the dark at dial 100");
    assert!(!text1.contains("\"events\":[]"), "dial 100 must spend wrong details");
}

#[test]
fn house_interact_mesh_bytes_updates_mesh_and_refuses_invalid_commands() {
    let house = generate_house(42, Style::Modern);
    let door_id = house.floors[0]
        .openings
        .iter()
        .find(|o| o.kind == OpeningKind::Door)
        .unwrap()
        .id;
    let lamp_id = house.floors[0]
        .furniture
        .iter()
        .find(|p| p.kind == FurnitureKind::Lamp)
        .unwrap()
        .id;

    let base_mesh = house_mesh_bytes(42, 3).unwrap();
    let opened = house_interact_mesh_bytes(42, 3, 0, door_id, 0, 0).expect("open door");
    assert_eq!(&opened[0..4], &MESH_MAGIC);
    assert_ne!(opened, base_mesh, "opening a door must move its panel in the mesh");

    let lit = house_interact_mesh_bytes(42, 3, 2, lamp_id, 0, 0).expect("toggle lamp");
    assert_ne!(lit, base_mesh, "toggling a lamp must change its shade material in the mesh");

    assert!(house_interact_mesh_bytes(42, 3, 0, 99999, 0, 0).is_err());
    assert!(house_interact_mesh_bytes(42, 3, 99, door_id, 0, 0).is_err());
}
