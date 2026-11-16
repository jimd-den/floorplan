//! The unbounded street network: continuity across chunk borders,
//! cul-de-sac bulbs, junctions, smoothness, and far-away chunks.
//!
//! Continuity is the property that makes streaming possible: two
//! neighbouring chunks must draw a shared street identically, because
//! each is generated alone and nothing is carried between them.

use floorplan::domain::geometry::Rect;
use floorplan::domain::lattice::{LATTICE, Node, Topology};
use floorplan::domain::neighborhood::Street;
use floorplan::use_cases::world_streets::{node_degree, streets_in, topology_at};

fn chunk(cx: i32, cz: i32) -> Rect {
    Rect::new(cx as f64 * LATTICE, cz as f64 * LATTICE, LATTICE, LATTICE)
}

/// The key of a street: its two end control points, rounded, so the
/// same edge found from two chunks compares equal.
fn key(s: &Street) -> (i64, i64, i64, i64) {
    let a = s.control[0];
    let b = *s.control.last().unwrap();
    (a.x as i64, a.y as i64, b.x as i64, b.y as i64)
}

#[test]
fn a_chunk_of_streets_is_deterministic_and_seeded() {
    assert_eq!(streets_in(42, chunk(0, 0)), streets_in(42, chunk(0, 0)));
    assert_ne!(streets_in(42, chunk(0, 0)), streets_in(43, chunk(0, 0)));
}

#[test]
fn neighbouring_chunks_draw_the_shared_street_identically() {
    let mut compared = 0;
    for seed in 0..20u32 {
        let west = streets_in(seed, chunk(0, 0));
        let east = streets_in(seed, chunk(1, 0));
        for w in &west {
            if let Some(e) = east.iter().find(|e| key(e) == key(w)) {
                assert_eq!(w, e, "seed {seed}: the shared street differs between chunks");
                compared += 1;
            }
        }
    }
    assert!(compared > 10, "only {compared} shared streets compared");
}

#[test]
fn a_street_is_one_spline_through_its_two_nodes() {
    for seed in 0..10u32 {
        for s in streets_in(seed, chunk(0, 0)) {
            assert!(s.control.len() >= 4, "a street needs its two nodes and two bends");
            let first = s.samples.points[0];
            let last = *s.samples.points.last().unwrap();
            assert!(first.dist(s.control[0]) < 1e-9, "seed {seed}: street does not start on its node");
            assert!(
                last.dist(*s.control.last().unwrap()) < 1e-9 || s.bulb_at.is_some(),
                "seed {seed}: street does not end on its node"
            );
        }
    }
}

#[test]
fn streets_are_smooth() {
    for seed in 0..20u32 {
        for s in streets_in(seed, chunk(0, 0)) {
            let mut worst = 0.0f64;
            for w in s.samples.points.windows(3) {
                let a = w[1].sub(w[0]).unit();
                let b = w[2].sub(w[1]).unit();
                let turn = a.dot(b).clamp(-1.0, 1.0).acos().to_degrees();
                worst = worst.max(turn);
            }
            assert!(worst < 30.0, "seed {seed}: sharpest turn {worst:.1}°");
        }
    }
}

#[test]
fn a_dead_end_ends_in_a_bulb_and_the_bulb_is_at_the_street_end() {
    let mut bulbs = 0;
    for seed in 0..30u32 {
        for s in streets_in(seed, chunk(0, 0)) {
            if let (Some(at), Some(centre)) = (s.bulb_at, s.bulb_center) {
                bulbs += 1;
                assert!((at - s.samples.len()).abs() < 1e-6 || at < s.samples.len(), "bulb not at the end");
                assert!(centre.dist(s.samples.point_at(at)) < 20.0, "bulb centre is off the street");
            }
        }
    }
    assert!(bulbs > 0, "no cul-de-sac bulb in 30 chunks");
}

#[test]
fn the_network_contains_dead_ends_curves_and_junctions() {
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..20u32 {
        for j in 0..3 {
            for i in 0..3 {
                seen.insert(topology_at(seed, Node::new(i, j)).id());
            }
        }
    }
    for want in ["dead_end", "curve", "tee", "cross"] {
        assert!(seen.contains(want), "the network never produces a {want}");
    }
}

#[test]
fn a_degree_is_the_number_of_kept_edges_touching_the_node() {
    let node = Node::new(1, 1);
    let degree = node_degree(42, node);
    assert!(degree <= 4, "degree {degree} exceeds a lattice node");
    assert_eq!(degree, node_degree(42, node));
}

#[test]
fn the_map_has_no_edge_and_no_fixed_size() {
    // Far from the origin, in every direction, the network still
    // generates: a chunk is addressed, never allocated.
    for (cx, cz) in [(1000, 0), (-1000, 0), (0, 1000), (0, -1000), (5000, -5000)] {
        let streets = streets_in(42, chunk(cx, cz));
        assert!(!streets.is_empty(), "chunk ({cx},{cz}) has no streets");
        for s in &streets {
            assert!(s.samples.len() >= 40.0, "chunk ({cx},{cz}): a street is too short");
        }
    }
}

#[test]
fn one_chunk_never_repeats_a_street() {
    for seed in 0..20u32 {
        let streets = streets_in(seed, chunk(0, 0));
        let mut keys: Vec<_> = streets.iter().map(key).collect();
        let before = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), before, "seed {seed}: a street is drawn twice");
    }
}

#[test]
fn every_street_is_close_enough_to_the_chunk_to_be_drawn_there() {
    let r = chunk(0, 0);
    let pad = LATTICE + 60.0;
    for s in streets_in(42, r) {
        let near = s
            .samples
            .points
            .iter()
            .any(|p| p.x >= r.x - pad && p.x <= r.right() + pad && p.y >= r.y - pad && p.y <= r.bottom() + pad);
        assert!(near, "a street outside the chunk's reach was returned");
    }
}

#[test]
fn topology_names_are_stable() {
    assert_eq!(Topology::DeadEnd.id(), "dead_end");
    assert_eq!(Topology::Tee.id(), "tee");
}
