//! generate_house: the pipeline footprint -> floors -> stairs.
//! Each floor is built by `floor::build_floor_retiled`, which
//! re-tiles until no room exceeds its program cap.

use crate::domain::model::House;
use crate::domain::style::Style;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::floor::{build_floor_retiled, hall_or_living, upper_rects};
use crate::use_cases::footprint::footprint;

/// Builds a whole house from a seed and a style.
pub fn generate_house(seed: u32, style: Style) -> House {
    let mut rng = Rng::new(seed);
    let spec = style.spec();
    let fp = footprint(&mut rng, style);
    let mut next_room = 0u32;
    let ground = build_floor_retiled(
        &mut rng,
        &fp.rects,
        &fp.joins,
        0,
        style,
        &mut next_room,
        fp.garage,
    );
    let mut floors = vec![ground];
    let mut stairs = None;
    if spec.stories == 2 {
        let upper_rects = upper_rects(&fp, style);
        let upper = build_floor_retiled(
            &mut rng,
            &upper_rects,
            &[],
            1,
            style,
            &mut next_room,
            None,
        );
        let g_hall = hall_or_living(&floors[0]);
        let u_hall = hall_or_living(&upper);
        stairs = Some((g_hall, u_hall));
        floors.push(upper);
    }
    // Furniture is placed after the plan is final, from its own RNG
    // seeded by the house seed and floor, so the plan's re-tiling is
    // independent of how many pieces a room can hold.
    for (i, floor) in floors.iter_mut().enumerate() {
        let mut furn_rng = Rng::new(seed ^ (i as u32 + 1).wrapping_mul(0x9E37_79B9));
        floor.furniture = crate::use_cases::furnish::furnish(&mut furn_rng, floor);
    }
    with_lights(House {
        seed,
        style,
        footprint: fp,
        floors,
        stairs,
        lights: Vec::new(),
    })
}

/// Fills a freshly built house's light sources from its rooms,
/// furniture, windows, and porch. Kept separate so the plan pipeline
/// stays light-free until the house exists.
fn with_lights(mut house: House) -> House {
    house.lights = crate::use_cases::light::lights_for_house(&house);
    house
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::OpeningKind;

    #[test]
    fn same_seed_gives_the_same_house() {
        let a = generate_house(42, Style::Ranch);
        let b = generate_house(42, Style::Ranch);
        assert_eq!(a, b);
    }

    #[test]
    fn every_style_generates_rooms_doors_and_windows() {
        for style in Style::all() {
            let house = generate_house(7, style);
            for floor in &house.floors {
                assert!(!floor.rooms.is_empty(), "{:?} floor {} has no rooms", style, floor.level);
                assert!(
                    floor.openings.iter().any(|o| o.kind == OpeningKind::Door),
                    "{:?} floor {} has no door",
                    style,
                    floor.level
                );
                assert!(
                    floor.openings.iter().any(|o| o.kind == OpeningKind::Window),
                    "{:?} floor {} has no window",
                    style,
                    floor.level
                );
            }
        }
    }

    #[test]
    fn two_storey_styles_have_stairs() {
        for style in [Style::Colonial, Style::Townhouse] {
            let house = generate_house(7, style);
            assert_eq!(house.floors.len(), 2, "{:?} should have two floors", style);
            let (a, b) = house.stairs.expect("stairs expected");
            let ids: Vec<u32> = house.floors.iter().flat_map(|f| f.rooms.iter().map(|r| r.id)).collect();
            assert!(ids.contains(&a) && ids.contains(&b));
        }
    }

    #[test]
    fn one_storey_styles_have_no_stairs() {
        for style in [Style::Ranch, Style::Craftsman, Style::Modern, Style::Cottage] {
            let house = generate_house(7, style);
            assert_eq!(house.floors.len(), 1, "{:?} should have one floor", style);
            assert!(house.stairs.is_none());
        }
    }

    #[test]
    fn rooms_cover_the_footprint_exactly() {
        let house = generate_house(11, Style::Ranch);
        let footprint_area: f64 = house.footprint.rects.iter().map(|r| r.area()).sum();
        let rooms_area: f64 = house.floors[0]
            .rooms
            .iter()
            .filter(|r| r.program != crate::domain::model::Program::Garage)
            .map(|r| r.rect.area())
            .sum();
        assert!((footprint_area - rooms_area).abs() < 1e-6);
    }
}
