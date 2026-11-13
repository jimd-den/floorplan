//! Per-family neighborhood tests: the invariants hold for 100
//! seeds in each layout family, every house faces its street
//! (enterable), and each family's layouts are all distinct.

use floorplan::adapters::neighborhood_json::render_neighborhood_json;
use floorplan::domain::layout::Layout;
use floorplan::domain::neighborhood::Neighborhood;
use floorplan::use_cases::neighborhood::{
    BACK_SETBACK, FRONT_SETBACK, SIDE_SETBACK, generate_neighborhood, generate_neighborhood_layout,
};
use floorplan::use_cases::variety::neighborhood_hash;
use std::collections::HashSet;

/// Seeds whose family (by seed % 3) is the given one.
fn family_seeds(layout: Layout) -> Vec<u32> {
    let idx = Layout::all().iter().position(|&l| l == layout).unwrap() as u32;
    (0..100u32).map(|k| 3 * k + idx).collect()
}

/// The full invariant set for one neighborhood: inside the lot
/// with setbacks, no overlaps, one house per lot, every lot
/// street-fronted, and every house facing its street.
fn check_neighborhood(n: &Neighborhood, seed: u32) {
    for h in &n.houses {
        let lot = n.lots[h.lot].rect;
        assert!(h.rect.x >= lot.x - 1e-9 && h.rect.right() <= lot.right() + 1e-9, "seed {seed}: outside lot");
        assert!(h.rect.y >= lot.y - 1e-9 && h.rect.bottom() <= lot.bottom() + 1e-9, "seed {seed}: outside lot");
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
                h.rect.x - lot.x >= SIDE_SETBACK - 1e-9 && lot.right() - h.rect.right() >= SIDE_SETBACK - 1e-9
            }
            _ => h.rect.y - lot.y >= SIDE_SETBACK - 1e-9 && lot.bottom() - h.rect.bottom() >= SIDE_SETBACK - 1e-9,
        };
        assert!(front_ok && back_ok && side_ok, "seed {seed}: setback violated");
        // Enterable: the lot's street is on the side the house faces.
        let lot_street = &n.streets[n.lots[h.lot].street];
        let faces_street = match h.rotation {
            0 => (lot_street.rect.y - lot.bottom()).abs() < 1e-9,
            180 => (lot.y - lot_street.rect.bottom()).abs() < 1e-9,
            90 => (lot_street.rect.x - lot.right()).abs() < 1e-9,
            _ => (lot.x - lot_street.rect.right()).abs() < 1e-9,
        };
        assert!(faces_street, "seed {seed}: house does not face its street");
    }
    for i in 0..n.houses.len() {
        for j in i + 1..n.houses.len() {
            assert!(!n.houses[i].rect.overlaps(&n.houses[j].rect), "seed {seed}: houses {i} and {j} overlap");
        }
    }
    assert_eq!(n.lots.len(), n.houses.len(), "seed {seed}: lot/house count mismatch");
    let mut seen: HashSet<usize> = HashSet::new();
    for h in &n.houses {
        assert!(seen.insert(h.lot), "seed {seed}: lot {} has two houses", h.lot);
    }
    for lot in &n.lots {
        let s = &n.streets[lot.street];
        let touches = (lot.rect.y - s.rect.bottom()).abs() < 1e-9
            || (lot.rect.bottom() - s.rect.y).abs() < 1e-9
            || (lot.rect.x - s.rect.right()).abs() < 1e-9
            || (lot.rect.right() - s.rect.x).abs() < 1e-9;
        assert!(touches, "seed {seed}: lot without street frontage");
    }
}

#[test]
fn the_invariants_hold_for_100_seeds_per_family() {
    for layout in Layout::all() {
        for seed in family_seeds(layout) {
            let n = generate_neighborhood_layout(seed, (2, 2), layout);
            assert_eq!(n.layout, layout);
            check_neighborhood(&n, seed);
        }
    }
}

#[test]
fn each_family_gives_100_distinct_layouts() {
    for layout in Layout::all() {
        let mut hashes = HashSet::new();
        for seed in family_seeds(layout) {
            let n = generate_neighborhood_layout(seed, (2, 2), layout);
            assert!(hashes.insert(neighborhood_hash(&n)), "{layout:?} seed {seed}: hash collision");
        }
        assert_eq!(hashes.len(), 100, "{layout:?}: not all distinct");
    }
}

#[test]
fn the_json_names_the_layout_family() {
    assert!(render_neighborhood_json(&generate_neighborhood(0, (2, 2))).contains("\"layout\":\"grid\""));
    assert!(render_neighborhood_json(&generate_neighborhood(1, (2, 2))).contains("\"layout\":\"offset\""));
    assert!(render_neighborhood_json(&generate_neighborhood(2, (2, 2))).contains("\"layout\":\"cul-de-sac\""));
}
