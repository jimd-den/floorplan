//! Higher-density neighborhood tests: house counts, setback/frontage
//! invariants across all layout families and density tiers, and WASM
//! density payloads.

use floorplan::domain::layout::{Density, Layout};
use floorplan::use_cases::neighborhood::{
    BACK_SETBACK, DEFAULT_BLOCKS, FRONT_SETBACK, SIDE_SETBACK, generate_neighborhood_density,
    generate_neighborhood_full,
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
        assert!(
            high.houses.len() >= 32,
            "seed {seed}: high density only placed {} houses",
            high.houses.len()
        );
    }
}

#[test]
fn all_density_tiers_and_layout_families_preserve_setbacks_and_frontage() {
    for density in Density::all() {
        for layout in Layout::all() {
            for seed in 0..15u32 {
                let n = generate_neighborhood_full(seed, DEFAULT_BLOCKS, layout, density);
                assert_eq!(n.lots.len(), n.houses.len());
                let mut seen = HashSet::new();
                for h in &n.houses {
                    assert!(seen.insert(h.lot));
                    let lot = n.lots[h.lot].rect;
                    let front_ok = match h.rotation {
                        0 => lot.bottom() - h.rect.bottom() >= FRONT_SETBACK - 1e-9,
                        180 => h.rect.y - lot.y >= FRONT_SETBACK - 1e-9,
                        90 => lot.right() - h.rect.right() >= FRONT_SETBACK - 1e-9,
                        _ => h.rect.x - lot.x >= FRONT_SETBACK - 1e-9,
                    };
                    let back_ok = match h.rotation {
                        0 => h.rect.y - lot.y >= BACK_SETBACK - 1e-9,
                        180 => lot.bottom() - h.rect.bottom() >= BACK_SETBACK - 1e-9,
                        90 => h.rect.x - lot.x >= BACK_SETBACK - 1e-9,
                        _ => lot.right() - h.rect.right() >= BACK_SETBACK - 1e-9,
                    };
                    let side_ok = match h.rotation {
                        0 | 180 => {
                            h.rect.x - lot.x >= SIDE_SETBACK - 1e-9
                                && lot.right() - h.rect.right() >= SIDE_SETBACK - 1e-9
                        }
                        _ => {
                            h.rect.y - lot.y >= SIDE_SETBACK - 1e-9
                                && lot.bottom() - h.rect.bottom() >= SIDE_SETBACK - 1e-9
                        }
                    };
                    assert!(
                        front_ok && back_ok && side_ok,
                        "{density:?} {layout:?} seed {seed}: setback violated"
                    );
                }
                for i in 0..n.houses.len() {
                    for j in i + 1..n.houses.len() {
                        assert!(
                            !n.houses[i].rect.overlaps(&n.houses[j].rect),
                            "{density:?} {layout:?} seed {seed}: houses {i} and {j} overlap"
                        );
                    }
                }
            }
        }
    }
}
