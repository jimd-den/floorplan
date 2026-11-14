//! neighborhood: seed -> Neighborhood. A street network of
//! blocks, in the layout family the seed picks, with lots along
//! the streets and one house per lot from the existing generator,
//! each facing its street. Lot size biases the style: the small
//! east/west lots get Cottage/Townhouse, wide lots get
//! Ranch/Colonial/Modern/Craftsman. Lot sizes are chosen so any
//! generated house fits its lot with setbacks (max house extent
//! 26 x 16 m, proven in domain::neighborhood).

use crate::domain::layout::{Density, Layout};
use crate::domain::neighborhood::Neighborhood;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::layout::plan_layout;
use crate::use_cases::lots::{lay_block_with_density, lay_court_with_density};
use crate::use_cases::quant;

/// Street width, in metres.
pub const STREET_W: f64 = 8.0;
/// Front setback (house front to its street), in metres.
pub const FRONT_SETBACK: f64 = 5.0;
/// Back setback, in metres.
pub const BACK_SETBACK: f64 = 3.0;
/// Side setback, in metres.
pub const SIDE_SETBACK: f64 = 2.0;
/// Depth of the east/west lots into the block, in metres.
pub const SIDE_LOT_D: f64 = 24.0;
/// Default block count (columns, rows).
pub const DEFAULT_BLOCKS: (u32, u32) = (2, 2);

/// Builds a neighborhood: `blocks` is (columns, rows) of blocks.
/// The layout family is chosen deterministically from the seed.
pub fn generate_neighborhood(seed: u32, blocks: (u32, u32)) -> Neighborhood {
    generate_neighborhood_layout(seed, blocks, Layout::from_seed(seed))
}

/// Builds a neighborhood in a chosen layout family. The test
/// entry point for verifying the invariants per family.
pub fn generate_neighborhood_layout(seed: u32, blocks: (u32, u32), layout: Layout) -> Neighborhood {
    generate_neighborhood_full(seed, blocks, layout, Density::Low)
}

/// Builds a neighborhood at a chosen residential density (`Low`,
/// `Medium`, or `High`).
pub fn generate_neighborhood_density(
    seed: u32,
    blocks: (u32, u32),
    density: Density,
) -> Neighborhood {
    generate_neighborhood_full(seed, blocks, Layout::from_seed(seed), density)
}

/// Builds a neighborhood in a chosen layout family and density tier.
pub fn generate_neighborhood_full(
    seed: u32,
    blocks: (u32, u32),
    layout: Layout,
    density: Density,
) -> Neighborhood {
    let mut rng = Rng::new(seed);
    let block = quant(rng.range_f64(76.0, 96.0));
    let plan = plan_layout(&mut rng, layout, blocks, block);
    let mut lots = Vec::new();
    let mut houses = Vec::new();
    for b in &plan.blocks {
        match &b.court {
            Some(_) => lay_court_with_density(&mut rng, &mut lots, &mut houses, b, &plan.streets, density),
            None => lay_block_with_density(
                &mut rng,
                &mut lots,
                &mut houses,
                b.rect,
                (b.north, b.south, b.west, b.east),
                density,
            ),
        }
    }
    Neighborhood { seed, blocks, layout, streets: plan.streets, lots, houses }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;

    #[test]
    fn same_seed_gives_the_same_neighborhood() {
        assert_eq!(generate_neighborhood(42, (2, 2)), generate_neighborhood(42, (2, 2)));
    }

    #[test]
    fn different_seeds_give_different_neighborhoods() {
        assert_ne!(generate_neighborhood(1, (2, 2)), generate_neighborhood(2, (2, 2)));
    }

    #[test]
    fn the_layout_family_comes_from_the_seed() {
        assert_eq!(generate_neighborhood(0, (2, 2)).layout, Layout::Grid);
        assert_eq!(generate_neighborhood(1, (2, 2)).layout, Layout::Offset);
        assert_eq!(generate_neighborhood(2, (2, 2)).layout, Layout::CulDeSac);
        assert_eq!(generate_neighborhood_layout(42, (2, 2), Layout::Offset).layout, Layout::Offset);
    }

    #[test]
    fn the_default_blocks_give_a_modest_house_count() {
        for seed in 0..20u32 {
            let n = generate_neighborhood(seed, DEFAULT_BLOCKS);
            assert!((12..=24).contains(&n.houses.len()), "seed {seed}: {} houses", n.houses.len());
        }
    }

    #[test]
    fn every_house_is_inside_its_lot_with_setbacks() {
        for seed in 0..20u32 {
            let n = generate_neighborhood(seed, (2, 2));
            for h in &n.houses {
                let lot = n.lots[h.lot].rect;
                assert!(h.rect.x >= lot.x + SIDE_SETBACK - 1e-9, "seed {seed}: west");
                assert!(h.rect.right() <= lot.right() - SIDE_SETBACK + 1e-9, "seed {seed}: east");
                assert!(h.rect.y >= lot.y + SIDE_SETBACK - 1e-9, "seed {seed}: north");
                assert!(h.rect.bottom() <= lot.bottom() - SIDE_SETBACK + 1e-9, "seed {seed}: south");
            }
        }
    }

    #[test]
    fn every_lot_fronts_a_street_it_touches() {
        for seed in 0..20u32 {
            let n = generate_neighborhood(seed, (2, 2));
            for lot in &n.lots {
                let s = &n.streets[lot.street];
                let touches = (lot.rect.y - s.rect.bottom()).abs() < 1e-9
                    || (lot.rect.bottom() - s.rect.y).abs() < 1e-9
                    || (lot.rect.x - s.rect.right()).abs() < 1e-9
                    || (lot.rect.right() - s.rect.x).abs() < 1e-9;
                assert!(touches, "seed {seed}: lot {lot:?} misses street {s:?}");
            }
        }
    }

    #[test]
    fn east_and_west_lots_get_small_styles() {
        for seed in 0..20u32 {
            let n = generate_neighborhood(seed, (2, 2));
            for h in &n.houses {
                if h.rotation == 90 || h.rotation == 270 {
                    assert!(
                        matches!(h.house.style, Style::Cottage | Style::Townhouse),
                        "seed {seed}: {:?} on a side lot",
                        h.house.style
                    );
                }
            }
        }
    }
}
