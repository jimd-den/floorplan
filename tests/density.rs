//! Higher-density neighborhood tests: house counts, the curved
//! invariants, and lot sizes across all density tiers. Lots are
//! polygons along spline streets, so setbacks are checked as
//! house corners inside the lot polygon and overlaps as polygon
//! overlaps; there are no layout families left to iterate.

use floorplan::domain::layout::Density;
use floorplan::domain::placement::{house_corners, house_extent};
use floorplan::domain::polygon::{point_in_poly, polys_overlap};
use floorplan::use_cases::neighborhood::{
    DEFAULT_BLOCKS, generate_neighborhood_density, generate_neighborhood_full,
};
use std::collections::HashSet;

#[test]
fn medium_and_high_density_pack_more_houses_per_block_than_low() {
    for seed in 0..15u32 {
        let low = generate_neighborhood_density(seed, DEFAULT_BLOCKS, Density::Low);
        let med = generate_neighborhood_density(seed, DEFAULT_BLOCKS, Density::Medium);
        let high = generate_neighborhood_density(seed, DEFAULT_BLOCKS, Density::High);
        assert!(
            med.houses.len() > low.houses.len(),
            "seed {seed}: medium ({}) not denser than low ({})",
            med.houses.len(),
            low.houses.len()
        );
        assert!(
            high.houses.len() > med.houses.len(),
            "seed {seed}: high ({}) not denser than medium ({})",
            high.houses.len(),
            med.houses.len()
        );
        // The curved frame's street length (~370 m for 2x2) caps
        // high density: measured 19-30 over seeds 0-19, mean
        // 23.9. The grid-era floor of 32 assumed the grid's
        // denser street network and is unreachable here; 18
        // keeps high well above medium's floor of 12.
        assert!(
            high.houses.len() >= 18,
            "seed {seed}: high density only placed {} houses",
            high.houses.len()
        );
    }
}

#[test]
fn all_density_tiers_preserve_the_curved_invariants() {
    for density in Density::all() {
        for seed in 0..15u32 {
            let n = generate_neighborhood_full(seed, DEFAULT_BLOCKS, density);
            assert_eq!(n.lots.len(), n.houses.len(), "{density:?} seed {seed}");
            let mut seen = HashSet::new();
            for h in &n.houses {
                assert!(seen.insert(h.lot), "{density:?} seed {seed}: lot {} has two houses", h.lot);
                let extent = house_extent(&h.house);
                for c in house_corners(h.center, h.rotation, extent) {
                    assert!(
                        point_in_poly(c, &n.lots[h.lot].outline),
                        "{density:?} seed {seed}: a house corner escapes its lot"
                    );
                }
            }
            for i in 0..n.houses.len() {
                for j in i + 1..n.houses.len() {
                    let a = &n.houses[i];
                    let b = &n.houses[j];
                    let qa = house_corners(a.center, a.rotation, house_extent(&a.house));
                    let qb = house_corners(b.center, b.rotation, house_extent(&b.house));
                    assert!(
                        !polys_overlap(&qa, &qb),
                        "{density:?} seed {seed}: houses {i} and {j} overlap"
                    );
                }
            }
            for i in 0..n.lots.len() {
                for j in i + 1..n.lots.len() {
                    assert!(
                        !polys_overlap(&n.lots[i].outline, &n.lots[j].outline),
                        "{density:?} seed {seed}: lots {i} and {j} overlap"
                    );
                }
            }
        }
    }
}

#[test]
fn lot_sizes_follow_their_density_tier() {
    for density in Density::all() {
        let (f_lo, f_hi, d_lo, d_hi) = match density {
            Density::Low => (32.0, 42.0, 24.0, 28.0),
            Density::Medium => (25.0, 32.0, 24.0, 26.0),
            Density::High => (15.0, 24.0, 24.0, 24.0),
        };
        for seed in 0..15u32 {
            let n = generate_neighborhood_full(seed, DEFAULT_BLOCKS, density);
            for lot in &n.lots {
                assert!(
                    lot.frontage >= f_lo - 1e-9 && lot.frontage <= f_hi + 1e-9,
                    "{density:?} seed {seed}: frontage {:.2} m outside {f_lo}..{f_hi}",
                    lot.frontage
                );
                assert!(
                    lot.depth >= d_lo - 1e-9 && lot.depth <= d_hi + 1e-9,
                    "{density:?} seed {seed}: depth {:.2} m outside {d_lo}..{d_hi}",
                    lot.depth
                );
            }
        }
    }
}
