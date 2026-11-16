//! neighborhood: seed -> Neighborhood. A spline street network
//! (one spine, cul-de-sac branches) with lots along the streets
//! and one house per lot from the existing generator, each facing
//! its street. Lot size biases the style: narrow lots get
//! Cottage/Townhouse, wide lots get Ranch/Colonial/Modern/Craftsman.
//! Lot sizes are chosen so any generated house fits its lot with
//! setbacks (max house extent 26 x 16 m, proven in domain::neighborhood).
//! Street lamps and road signs are laid along the same streets; the
//! spine's ends are dead ends, a branch starts at a junction and ends
//! in a bulb.

use crate::domain::layout::Density;
use crate::domain::neighborhood::Neighborhood;
use crate::domain::street_furniture::{RoadSign, StreetLamp};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::lots::lay_lots;
use crate::use_cases::road_signs::{EndKind, StreetEnds, signs_along};
use crate::use_cases::street_lights::lamps_along;
use crate::use_cases::street_network::plan_network;
use crate::use_cases::world_lots::street_seed;

/// Street width, in metres.
pub const STREET_W: f64 = 8.0;
/// Front setback (house front to its street), in metres.
pub const FRONT_SETBACK: f64 = 5.0;
/// Back setback, in metres.
pub const BACK_SETBACK: f64 = 3.0;
/// Side setback, in metres.
pub const SIDE_SETBACK: f64 = 2.0;
/// Sidewalk width, in metres (each side of the street).
pub const SIDEWALK_W: f64 = 1.5;
/// Walkway width, in metres (sidewalk to the entry door).
pub const WALKWAY_W: f64 = 1.2;
/// Default block count (columns, rows) of street cells.
pub const DEFAULT_BLOCKS: (u32, u32) = (2, 2);

/// Builds a neighborhood: `blocks` is (columns, rows) of street
/// cells, about 85 m each.
pub fn generate_neighborhood(seed: u32, blocks: (u32, u32)) -> Neighborhood {
    generate_neighborhood_full(seed, blocks, Density::Low)
}

/// Builds a neighborhood at a chosen residential density (`Low`,
/// `Medium`, or `High`).
pub fn generate_neighborhood_density(
    seed: u32,
    blocks: (u32, u32),
    density: Density,
) -> Neighborhood {
    generate_neighborhood_full(seed, blocks, density)
}

/// Builds a neighborhood at a chosen residential density tier.
pub fn generate_neighborhood_full(
    seed: u32,
    blocks: (u32, u32),
    density: Density,
) -> Neighborhood {
    let mut rng = Rng::new(seed);
    let streets = plan_network(&mut rng, blocks);
    let (lots, houses) = lay_lots(&mut rng, &streets, density);
    let (lamps, signs) = street_furniture(&streets);
    Neighborhood { seed, blocks, streets, lots, houses, lamps, signs }
}

/// Lamps and signs along every street. Street 0 is the spine (its
/// ends are dead ends); every branch starts at a junction and ends
/// in a bulb, so it gets a stop and a name sign at its approach.
fn street_furniture(streets: &[crate::domain::neighborhood::Street]) -> (Vec<StreetLamp>, Vec<RoadSign>) {
    let mut lamps = Vec::new();
    let mut signs = Vec::new();
    for (i, street) in streets.iter().enumerate() {
        lamps.extend(lamps_along(street));
        let ends = if i == 0 {
            StreetEnds { start: EndKind::DeadEnd, end: EndKind::DeadEnd }
        } else {
            StreetEnds { start: EndKind::Junction, end: EndKind::DeadEnd }
        };
        signs.extend(signs_along(street, street_seed(street), ends));
    }
    (lamps, signs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_the_same_neighborhood() {
        assert_eq!(generate_neighborhood(42, (2, 2)), generate_neighborhood(42, (2, 2)));
    }

    #[test]
    fn different_seeds_give_different_neighborhoods() {
        assert_ne!(generate_neighborhood(1, (2, 2)), generate_neighborhood(2, (2, 2)));
    }

    #[test]
    fn the_default_blocks_give_a_modest_house_count() {
        for seed in 0..20u32 {
            let n = generate_neighborhood(seed, DEFAULT_BLOCKS);
            assert!((8..=40).contains(&n.houses.len()), "seed {seed}: {} houses", n.houses.len());
        }
    }

    #[test]
    fn a_neighborhood_carries_lamps_and_signs() {
        for seed in 0..20u32 {
            let n = generate_neighborhood(seed, DEFAULT_BLOCKS);
            assert!(!n.lamps.is_empty(), "seed {seed}: no street lamps");
            assert!(!n.signs.is_empty(), "seed {seed}: no road signs");
        }
    }
}
