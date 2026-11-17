//! Integration: the room-size study rules (ADR 0014) hold in
//! generated houses — no room above the 2.0 aspect cap, no
//! room-sized hall or entry, no habitable room under the IRC
//! floor (6.5 sq m, short side 2.13 m).

use floorplan::domain::model::{House, Program};
use floorplan::domain::room_size::{ASPECT_HARD_CAP, aspect, is_room_sized, meets_habitable_floor};
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;

fn style_for(i: u32) -> Style {
    Style::all()[(i % 6) as usize]
}

/// The five test floorplans, by (seed, style).
fn test_plans() -> [(u32, Style); 5] {
    [
        (42, Style::Ranch),
        (7, Style::Colonial),
        (101, Style::Craftsman),
        (202, Style::Modern),
        (303, Style::Cottage),
    ]
}

fn is_habitable(program: Program) -> bool {
    matches!(
        program,
        Program::Living | Program::Kitchen | Program::Dining | Program::Bedroom | Program::Study
    )
}

/// All three room-size rules for one house, with the seed in
/// every failure message.
fn assert_room_size_rules(house: &House, seed: u32) {
    for floor in &house.floors {
        for room in &floor.rooms {
            assert!(
                aspect(&room.rect) <= ASPECT_HARD_CAP + 1e-9,
                "seed {seed} {:?} room {} is {:.2}:1",
                room.program,
                room.id,
                aspect(&room.rect)
            );
            if room.program == Program::Hall || room.program == Program::Entry {
                assert!(
                    !is_room_sized(&room.rect),
                    "seed {seed} {:?} room {} is room-sized ({:.1} sq m)",
                    room.program,
                    room.id,
                    room.rect.area()
                );
            }
            if is_habitable(room.program) {
                assert!(
                    meets_habitable_floor(&room.rect),
                    "seed {seed} {:?} room {} is sub-floor ({:.1} sq m, short side {:.2} m)",
                    room.program,
                    room.id,
                    room.rect.area(),
                    room.rect.w.min(room.rect.d)
                );
            }
        }
    }
}

#[test]
fn colonial_seed_7_passes_the_room_size_rules() {
    // Regression: floor 1 once held a 7.2 x 9.5 m hall, a
    // 2.9 x 12.8 m bedroom (4.4:1) and a 2.4 x 10.3 m bedroom.
    let house = generate_house(7, Style::Colonial);
    assert_room_size_rules(&house, 7);
}

#[test]
fn the_five_test_plans_pass_the_room_size_rules() {
    for (seed, style) in test_plans() {
        let house = generate_house(seed, style);
        assert_room_size_rules(&house, seed);
    }
}

#[test]
fn a_seed_sweep_passes_the_room_size_rules_and_keeps_the_core_rooms() {
    for seed in 0..200u32 {
        let house = generate_house(seed, style_for(seed));
        assert_room_size_rules(&house, seed);
        let programs: Vec<Program> = house
            .floors
            .iter()
            .flat_map(|f| f.rooms.iter().map(|r| r.program))
            .collect();
        for want in [Program::Bedroom, Program::Bath, Program::Living, Program::Kitchen] {
            assert!(programs.contains(&want), "seed {seed} has no {want:?}");
        }
    }
}
