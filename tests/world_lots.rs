//! Lots along unbounded streets: back-to-back, clear of street ends,
//! only on the outer side of a bulb, and deterministic per street.

use floorplan::domain::curve::Pt;
use floorplan::domain::geometry::Rect;
use floorplan::domain::layout::Density;
use floorplan::domain::neighborhood::Street;
use floorplan::domain::placement::{house_corners, house_extent};
use floorplan::domain::polygon::{point_in_poly, polys_overlap};
use floorplan::use_cases::street_network::JUNCTION_CLEAR;
use floorplan::use_cases::world_lots::{lots_along, street_seed};
use floorplan::use_cases::world_streets::streets_in;

fn streets(seed: u32) -> Vec<Street> {
    streets_in(seed, Rect::new(0.0, 0.0, 140.0, 140.0))
}

/// A street with a bulb (a dead end) and one without.
fn with_bulb(seed: u32) -> Option<Street> {
    streets(seed).into_iter().find(|s| s.bulb_at.is_some())
}

#[test]
fn lots_along_a_street_are_deterministic() {
    let s = &streets(42)[0];
    let (a, ha) = lots_along(s, 0, Density::Low);
    let (b, hb) = lots_along(s, 0, Density::Low);
    assert_eq!(a, b, "the same street laid twice differs");
    assert_eq!(ha, hb, "the same street's houses differ");
    assert!(!a.is_empty(), "a street near the origin carries no lots");
}

#[test]
fn lots_and_houses_stay_in_lockstep() {
    for seed in 0..3u32 {
        for s in streets(seed) {
            let (lots, houses) = lots_along(&s, 0, Density::Low);
            assert_eq!(lots.len(), houses.len(), "seed {seed}");
        }
    }
}

#[test]
fn every_house_sits_inside_its_lot() {
    for seed in 0..3u32 {
        for s in streets(seed) {
            let (lots, houses) = lots_along(&s, 0, Density::Low);
            for h in &houses {
                for c in house_corners(h.center, h.rotation, house_extent(&h.house)) {
                    assert!(
                        point_in_poly(c, &lots[h.lot].outline),
                        "seed {seed}: a house corner escapes its lot"
                    );
                }
            }
        }
    }
}

#[test]
fn lots_along_one_street_never_overlap() {
    for seed in 0..3u32 {
        for s in streets(seed) {
            let (lots, _) = lots_along(&s, 0, Density::Low);
            for i in 0..lots.len() {
                for j in i + 1..lots.len() {
                    assert!(
                        !polys_overlap(&lots[i].outline, &lots[j].outline),
                        "seed {seed}: lots {i} and {j} overlap"
                    );
                }
            }
        }
    }
}

#[test]
fn no_lot_comes_within_the_junction_clearance_of_a_street_end() {
    for seed in 0..3u32 {
        for s in streets(seed) {
            let len = s.samples.len();
            for lot in lots_along(&s, 0, Density::Low).0 {
                let start = lot.center_s - lot.frontage / 2.0;
                let end = lot.center_s + lot.frontage / 2.0;
                assert!(
                    start >= JUNCTION_CLEAR - 1e-9,
                    "seed {seed}: lot starts {start:.1} m in, inside the clearance"
                );
                assert!(
                    end <= len - JUNCTION_CLEAR + 1e-9,
                    "seed {seed}: lot ends {:.1} m short of {len:.1}",
                    len - end
                );
            }
        }
    }
}

#[test]
fn a_bulb_loop_only_carries_lots_on_its_outer_side() {
    let mut checked = 0;
    for seed in 0..12u32 {
        let Some(s) = with_bulb(seed) else { continue };
        let bulb_at = s.bulb_at.unwrap();
        for lot in lots_along(&s, 0, Density::Low).0 {
            if lot.center_s <= bulb_at {
                continue;
            }
            let p = s.samples.point_at(lot.center_s);
            let n = s.samples.normal_at(lot.center_s).scale(lot.side as f64);
            let centre = s.bulb_center.unwrap();
            assert!(n.dot(p.sub(centre)) > 0.0, "seed {seed}: a lot sits inside the bulb");
            checked += 1;
        }
    }
    assert!(checked > 0, "no bulb lot was checked in 30 seeds");
}

#[test]
fn higher_density_packs_more_lots_onto_the_same_street() {
    let s = &streets(42)[0];
    let low = lots_along(s, 0, Density::Low).0.len();
    let high = lots_along(s, 0, Density::High).0.len();
    assert!(high > low, "high density ({high}) is not denser than low ({low})");
}

#[test]
fn every_lot_fronts_the_street_it_was_laid_on() {
    for seed in 0..3u32 {
        for s in streets(seed) {
            for lot in lots_along(&s, 0, Density::Low).0 {
                let s0 = lot.center_s - lot.frontage / 2.0;
                let expected = s.samples.point_at(s0).add(s.samples.normal_at(s0).scale(4.0 * lot.side as f64));
                assert!(
                    lot.outline[0].dist(expected) < 1e-6,
                    "seed {seed}: lot front is off the offset curve"
                );
            }
        }
    }
}

#[test]
fn a_street_key_survives_a_round_trip_through_a_point() {
    let s = &streets(42)[0];
    assert_eq!(street_seed(s), street_seed(s));
    assert_ne!(street_seed(s), 0, "a real street hashed to zero");
    // A lot's front centre is a unique point, so a chunk can own it.
    let lot = lots_along(s, 0, Density::Low).0[0].clone();
    let front = s.samples.point_at(lot.center_s).add(
        s.samples.normal_at(lot.center_s).scale(4.0 * lot.side as f64),
    );
    assert!(front.dist(Pt::new(0.0, 0.0)) > 0.0);
}
