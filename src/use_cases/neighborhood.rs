//! neighborhood: seed -> Neighborhood. A spline street network
//! (one spine, cul-de-sac branches) with lots along the streets
//! and one house per lot from the existing generator, each facing
//! its street. Lot size biases the style: narrow lots get
//! Cottage/Townhouse, wide lots get Ranch/Colonial/Modern/Craftsman.
//! Lot sizes are chosen so any generated house fits its lot with
//! setbacks (max house extent 26 x 16 m, proven in domain::neighborhood).

use crate::domain::layout::Density;
use crate::domain::neighborhood::Neighborhood;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::lots::lay_lots;
use crate::use_cases::street_network::plan_network;

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
    Neighborhood { seed, blocks, streets, lots, houses }
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
}
