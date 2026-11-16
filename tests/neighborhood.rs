//! Neighborhood integration tests: the curved invariants over 100
//! seeds, variety, and determinism of the SVG and JSON outputs.
//! Streets are spline centerlines and lots are polygons, so every
//! check here runs on polygons, never on rectangles.

use floorplan::adapters::neighborhood_json::render_neighborhood_json;
use floorplan::adapters::neighborhood_svg::render_neighborhood_svg;
use floorplan::domain::curve::Pt;
use floorplan::domain::neighborhood::Neighborhood;
use floorplan::domain::placement::{front_for, house_corners, house_extent};
use floorplan::domain::polygon::{point_in_poly, polys_overlap};
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::neighborhood::{FRONT_SETBACK, STREET_W, generate_neighborhood};
use floorplan::use_cases::variety::neighborhood_hash;
use std::collections::HashSet;

fn neighborhood(seed: u32) -> Neighborhood {
    generate_neighborhood(seed, (2, 2))
}

/// The placed corners of one house, in plan order.
fn corners(n: &Neighborhood, index: usize) -> [Pt; 4] {
    let h = &n.houses[index];
    house_corners(h.center, h.rotation, house_extent(&h.house))
}

#[test]
fn the_extent_is_never_wider_than_26_or_deeper_than_16() {
    for seed in 0..200u32 {
        for style in Style::all() {
            let e = house_extent(&generate_house(seed, style));
            assert!(e.w <= 26.0 + 1e-9, "seed {seed} {style:?}: width {}", e.w);
            assert!(e.d <= 16.0 + 1e-9, "seed {seed} {style:?}: depth {}", e.d);
        }
    }
}

#[test]
fn every_house_is_inside_its_lot_over_100_seeds() {
    for seed in 0..100u32 {
        let n = neighborhood(seed);
        for i in 0..n.houses.len() {
            for c in corners(&n, i) {
                assert!(
                    point_in_poly(c, &n.lots[n.houses[i].lot].outline),
                    "seed {seed}: a house corner escapes its lot"
                );
            }
        }
    }
}

#[test]
fn every_house_keeps_its_front_setback_over_100_seeds() {
    for seed in 0..100u32 {
        let n = neighborhood(seed);
        for h in &n.houses {
            let lot = &n.lots[h.lot];
            let street = &n.streets[lot.street];
            // The front edge midpoint sits STREET_W/2 + FRONT_SETBACK
            // out from the centerline along the lot's outward normal.
            let extent = house_extent(&h.house);
            let front_mid = h.center.add(front_for(h.rotation).scale(extent.d / 2.0));
            let outward = street.samples.normal_at(lot.center_s).scale(lot.side as f64);
            let from_centre = front_mid.sub(street.samples.point_at(lot.center_s)).dot(outward);
            assert!(
                from_centre - STREET_W / 2.0 >= FRONT_SETBACK - 1e-6,
                "seed {seed}: front setback is {:.2} m",
                from_centre - STREET_W / 2.0
            );
        }
    }
}

#[test]
fn no_two_houses_overlap_over_100_seeds() {
    for seed in 0..100u32 {
        let n = neighborhood(seed);
        for i in 0..n.houses.len() {
            for j in i + 1..n.houses.len() {
                assert!(
                    !polys_overlap(&corners(&n, i), &corners(&n, j)),
                    "seed {seed}: houses {i} and {j} overlap"
                );
            }
        }
    }
}

#[test]
fn no_two_lots_overlap_over_100_seeds() {
    for seed in 0..100u32 {
        let n = neighborhood(seed);
        for i in 0..n.lots.len() {
            for j in i + 1..n.lots.len() {
                assert!(
                    !polys_overlap(&n.lots[i].outline, &n.lots[j].outline),
                    "seed {seed}: lots {i} and {j} overlap"
                );
            }
        }
    }
}

#[test]
fn every_lot_fronts_a_street_and_holds_one_house_over_100_seeds() {
    for seed in 0..100u32 {
        let n = neighborhood(seed);
        assert_eq!(n.lots.len(), n.houses.len(), "seed {seed}: lot/house count mismatch");
        let mut seen: HashSet<usize> = HashSet::new();
        for h in &n.houses {
            assert!(seen.insert(h.lot), "seed {seed}: lot {} has two houses", h.lot);
        }
        for lot in &n.lots {
            assert!(lot.street < n.streets.len(), "seed {seed}: lot fronts no street");
            let street = &n.streets[lot.street];
            // The lot's front corner sits on the street's offset
            // curve, 4 m out on the lot's side: fronting is
            // geometric, not just an index pointing at a street.
            let s0 = lot.center_s - lot.frontage / 2.0;
            let expected = street.samples.point_at(s0).add(
                street.samples.normal_at(s0).scale(4.0 * lot.side as f64),
            );
            assert!(
                lot.outline[0].dist(expected) < 1e-6,
                "seed {seed}: lot front is off its street's offset curve"
            );
        }
    }
}

#[test]
fn every_house_faces_its_street_over_100_seeds() {
    for seed in 0..100u32 {
        let n = neighborhood(seed);
        for h in &n.houses {
            let lot = &n.lots[h.lot];
            let street = &n.streets[lot.street];
            let outward = street.samples.normal_at(lot.center_s).scale(lot.side as f64);
            let facing = front_for(h.rotation);
            let align = facing.dot(outward.scale(-1.0));
            assert!(align > 0.99, "seed {seed}: house faces away from its street (align {align:.3})");
        }
    }
}

#[test]
fn a_hundred_seeds_give_a_hundred_distinct_neighborhoods() {
    let mut hashes = HashSet::new();
    for seed in 0..100u32 {
        let n = neighborhood(seed);
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
fn the_street_network_is_a_spine_with_bulb_branches() {
    let n = neighborhood(42);
    assert!(!n.streets.is_empty(), "no streets");
    let spine = &n.streets[0];
    assert!(spine.samples.len() > 0.0, "spine has no length");
    for branch in n.streets.iter().skip(1) {
        let start = branch.control[0];
        let joins = spine.samples.points.iter().any(|p| p.dist(start) < 0.5);
        assert!(joins, "branch starts off the spine");
        assert!(branch.bulb_at.is_some(), "branch has no turnaround bulb");
        assert!(branch.bulb_center.is_some(), "bulb has no centre");
    }
}
