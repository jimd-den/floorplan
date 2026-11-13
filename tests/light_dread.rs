//! Light-to-dread integration tests: daylight by sun position, a lit
//! room grounding the player, a flicker as a dread event, and the
//! flashlight standing in for a missing fixture.

use floorplan::domain::haunt::{Event, EventKind};
use floorplan::domain::light::{Daylight, LightSource, LightState};
use floorplan::domain::model::{House, OpeningKind};
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::haunt::{new_dread, step};
use floorplan::use_cases::light_dread::{
    Lighting, PlayerLight, flicker_event, room_light, systems_for,
};

fn house42() -> House {
    generate_house(42, Style::Modern)
}

fn room_ids(house: &House) -> Vec<u32> {
    house.floors.iter().flat_map(|f| f.rooms.iter().map(|r| r.id)).collect()
}

fn ceiling_id(room_id: u32) -> u32 {
    1000 + room_id
}

fn first_room_with_window(house: &House) -> u32 {
    house
        .floors
        .iter()
        .flat_map(|f| f.openings.iter())
        .find(|o| o.kind == OpeningKind::Window)
        .map(|o| o.from)
        .expect("no window in the house")
}

fn lights_with_room_off(house: &House, room_id: u32) -> Vec<LightSource> {
    let mut lights = house.lights.clone();
    for light in &mut lights {
        if light.room_id == Some(room_id) {
            light.state = LightState::Off;
        }
    }
    lights
}

#[test]
fn windows_admit_daylight_by_sun_position() {
    let house = house42();
    let room_id = first_room_with_window(&house);
    let noon = Lighting { lights: &house.lights, daylight: Daylight::at(720), tick: 0 };
    assert!((room_light(room_id, &noon) - 1.0).abs() < 1e-9, "noon does not fill the room");
    let night = Lighting { lights: &house.lights, daylight: Daylight::at(0), tick: 0 };
    assert_eq!(room_light(room_id, &night), 0.0, "midnight is not dark");
}

#[test]
fn a_lit_room_lowers_dread_and_a_dark_room_raises_it() {
    let house = house42();
    let room_id = room_ids(&house)[0];
    let player = PlayerLight {
        room_id,
        carrying_flashlight: false,
        dark_minutes: 10.0,
        tasks_done: 0,
    };
    let dark_lights = lights_with_room_off(&house, room_id);
    let night = Daylight::at(0);
    let dark = systems_for(&player, &Lighting { lights: &dark_lights, daylight: night, tick: 0 });
    assert_eq!(dark.darkness, 1.0);
    let mut state = new_dread(50);
    let before = state.level;
    step(&mut state, &dark, &[]);
    assert!(state.level > before, "darkness must raise dread");

    let mut lit_lights = dark_lights.clone();
    lit_lights.iter_mut().find(|l| l.id == ceiling_id(room_id)).unwrap().state = LightState::On;
    let lit = systems_for(&player, &Lighting { lights: &lit_lights, daylight: night, tick: 0 });
    assert_eq!(lit.light_relief, 1.0);
    let before = state.level;
    step(&mut state, &lit, &[]);
    assert!(state.level < before, "light must ground the player");
}

#[test]
fn a_flickering_light_is_a_dread_event() {
    let house = house42();
    let room_id = room_ids(&house)[0];
    let mut lights = lights_with_room_off(&house, room_id);
    lights.iter_mut().find(|l| l.id == ceiling_id(room_id)).unwrap().state = LightState::Flicker;

    assert_eq!(
        flicker_event(room_id, &lights),
        Some(Event { kind: EventKind::LightFlicker, target: room_id })
    );
    let on = Lighting { lights: &lights, daylight: Daylight::at(0), tick: 0 };
    let off = Lighting { lights: &lights, daylight: Daylight::at(0), tick: 4 };
    assert_eq!(room_light(room_id, &on), 0.5);
    assert_eq!(room_light(room_id, &off), 0.0);
}

#[test]
fn the_flashlight_lights_a_dark_room_at_night() {
    let house = house42();
    let room_id = room_ids(&house)[0];
    let lights = lights_with_room_off(&house, room_id);
    let player = PlayerLight {
        room_id,
        carrying_flashlight: true,
        dark_minutes: 0.0,
        tasks_done: 0,
    };
    let systems = systems_for(&player, &Lighting { lights: &lights, daylight: Daylight::at(0), tick: 0 });
    assert!(systems.darkness < 1.0, "the flashlight left the room dark");
    assert!(systems.light_relief > 0.0);
}
