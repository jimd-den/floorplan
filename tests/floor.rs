//! Floor builds, as integration tests so the implementation
//! file stays under the 200-line ceiling.

use floorplan::domain::model::{Floor, Program as P};
use floorplan::domain::style::Style;
use floorplan::frameworks_drivers::rng::Rng;
use floorplan::use_cases::caps::cap_for;
use floorplan::use_cases::floor::build_floor_retiled;
use floorplan::use_cases::footprint::footprint;

fn floor_rooms(seed: u32, style: Style) -> Floor {
    let mut rng = Rng::new(seed);
    let fp = footprint(&mut rng, style);
    let mut next_room = 0;
    build_floor_retiled(&mut rng, &fp.rects, &fp.joins, 0, style, &mut next_room, fp.garage)
}

#[test]
fn re_tiling_is_deterministic() {
    let a = floor_rooms(97, Style::Modern);
    let b = floor_rooms(97, Style::Modern);
    assert_eq!(a, b);
}

#[test]
fn the_seed_97_modern_bath_is_capped_after_re_tiling() {
    // Seed 97 Modern carried a 53.0 sq m bath (15.6 x 3.4 m).
    let floor = floor_rooms(97, Style::Modern);
    for room in &floor.rooms {
        if let Some(cap) = cap_for(room.program) {
            assert!(
                room.rect.area() <= cap + 1e-9,
                "{:?} room {} is {:.1} sq m",
                room.program,
                room.id,
                room.rect.area()
            );
        }
    }
    assert!(floor.rooms.iter().any(|r| r.program == P::Bath), "the floor lost its bath");
}

#[test]
fn a_re_tiled_floor_still_tiles_its_footprint() {
    let mut rng = Rng::new(97);
    let fp = footprint(&mut rng, Style::Modern);
    let floor = floor_rooms(97, Style::Modern);
    let footprint_area: f64 = fp.rects.iter().map(|r| r.area()).sum();
    let rooms_area: f64 = floor
        .rooms
        .iter()
        .filter(|r| r.program != P::Garage)
        .map(|r| r.rect.area())
        .sum();
    assert!((footprint_area - rooms_area).abs() < 1e-6, "area changed");
}
