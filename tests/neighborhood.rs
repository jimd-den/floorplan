//! Neighborhood integration tests: the invariants over 100 seeds,
//! variety, and determinism of the SVG and JSON outputs.

use floorplan::adapters::neighborhood_json::render_neighborhood_json;
use floorplan::adapters::neighborhood_svg::render_neighborhood_svg;
use floorplan::use_cases::neighborhood::{BACK_SETBACK, FRONT_SETBACK, SIDE_SETBACK, generate_neighborhood};
use floorplan::use_cases::variety::neighborhood_hash;
use std::collections::HashSet;

fn inside(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    a.0 >= b.0 - 1e-9 && a.1 >= b.1 - 1e-9 && a.2 <= b.2 + 1e-9 && a.3 <= b.3 + 1e-9
}

#[test]
fn every_house_is_inside_its_lot_over_100_seeds() {
    for seed in 0..100u32 {
        let n = generate_neighborhood(seed, (2, 2));
        for h in &n.houses {
            let lot = n.lots[h.lot].rect;
            assert!(
                inside((h.rect.x, h.rect.y, h.rect.right(), h.rect.bottom()), (lot.x, lot.y, lot.right(), lot.bottom())),
                "seed {seed}: house outside its lot"
            );
        }
    }
}

#[test]
fn every_house_respects_the_setbacks_over_100_seeds() {
    for seed in 0..100u32 {
        let n = generate_neighborhood(seed, (2, 2));
        for h in &n.houses {
            let lot = n.lots[h.lot].rect;
            // Front and back setbacks are 5 and 3; sides are 2.
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
            assert!(front_ok && back_ok && side_ok, "seed {seed}: setback violated");
        }
    }
}

#[test]
fn no_two_houses_overlap_over_100_seeds() {
    for seed in 0..100u32 {
        let n = generate_neighborhood(seed, (2, 2));
        for i in 0..n.houses.len() {
            for j in i + 1..n.houses.len() {
                assert!(!n.houses[i].rect.overlaps(&n.houses[j].rect), "seed {seed}: houses {i} and {j} overlap");
            }
        }
    }
}

#[test]
fn every_lot_has_street_frontage_and_exactly_one_house_over_100_seeds() {
    for seed in 0..100u32 {
        let n = generate_neighborhood(seed, (2, 2));
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
}

#[test]
fn a_hundred_seeds_give_a_hundred_distinct_neighborhoods() {
    let mut hashes = HashSet::new();
    for seed in 0..100u32 {
        let n = generate_neighborhood(seed, (2, 2));
        assert!(hashes.insert(neighborhood_hash(&n)), "seed {seed}: hash collision");
    }
    assert_eq!(hashes.len(), 100);
}

#[test]
fn the_svg_and_json_are_byte_identical_for_the_same_seed() {
    for seed in [0u32, 7, 42, 106, 199] {
        let n = generate_neighborhood(seed, (3, 3));
        let svg_a = render_neighborhood_svg(&n);
        let svg_b = render_neighborhood_svg(&generate_neighborhood(seed, (3, 3)));
        assert_eq!(svg_a, svg_b, "seed {seed}: SVG differs");
        let json_a = render_neighborhood_json(&n);
        let json_b = render_neighborhood_json(&generate_neighborhood(seed, (3, 3)));
        assert_eq!(json_a, json_b, "seed {seed}: JSON differs");
    }
}

#[test]
fn the_street_grid_has_a_through_street_and_cross_streets() {
    let n = generate_neighborhood(42, (3, 3));
    // 4 vertical + 4 horizontal streets for a 3x3 grid.
    assert_eq!(n.streets.len(), 8);
    let vertical = n.streets.iter().filter(|s| !s.horizontal).count();
    let horizontal = n.streets.iter().filter(|s| s.horizontal).count();
    assert_eq!((vertical, horizontal), (4, 4));
    // A through-street spans the whole neighborhood height.
    let total_h = n.streets.iter().map(|s| s.rect.bottom()).fold(0.0f64, f64::max);
    assert!(n.streets.iter().any(|s| !s.horizontal && (s.rect.bottom() - total_h).abs() < 1e-9));
}
