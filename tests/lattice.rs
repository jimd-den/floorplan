//! The world lattice: jitter, edge holes, and topology classes.
//!
//! These are the properties the unbounded map rests on: a node stays
//! near its lattice point, neighbours keep their order (so edges never
//! cross), and some edges are dropped (so cul-de-sacs and T-junctions
//! exist). Every value is a pure function of the seed and the key.

use floorplan::domain::curve::Pt;
use floorplan::domain::lattice::{
    Edge, JITTER, LATTICE, Node, Topology, edge_present, edge_seed, node_point,
    node_seed,
};

fn node(i: i32, j: i32) -> Node {
    Node::new(i, j)
}

#[test]
fn a_node_sits_within_the_jitter_of_its_lattice_point() {
    for seed in 0..50u32 {
        for (i, j) in [(0, 0), (1, 0), (0, 1), (-3, 4), (100, -200)] {
            let p = node_point(seed, node(i, j));
            let (lx, ly) = (i as f64 * LATTICE, j as f64 * LATTICE);
            assert!((p.x - lx).abs() <= JITTER as f64 + 1e-9, "seed {seed}: x jitter");
            assert!((p.y - ly).abs() <= JITTER as f64 + 1e-9, "seed {seed}: y jitter");
            assert_eq!(p.x.fract(), 0.0, "seed {seed}: x not on the 1 m grid");
            assert_eq!(p.y.fract(), 0.0, "seed {seed}: y not on the 1 m grid");
        }
    }
}

#[test]
fn node_positions_are_deterministic_and_seeded() {
    assert_eq!(node_point(42, node(3, -2)), node_point(42, node(3, -2)));
    assert_ne!(node_point(42, node(3, -2)), node_point(43, node(3, -2)));
    assert_ne!(node_point(42, node(3, -2)), node_point(42, node(3, -1)));
}

#[test]
fn the_jitter_actually_varies_across_nodes() {
    let xs: std::collections::HashSet<i64> =
        (0..40).map(|i| node_point(7, node(i, 0)).x as i64).collect();
    assert!(xs.len() > 30, "jitter only produced {} values", xs.len());
}

#[test]
fn neighbours_keep_their_order_so_edges_never_cross() {
    // Jitter is bounded well under half the spacing, so the east
    // neighbour is always east and the south one always south: the
    // lattice stays a planar grid.
    for seed in 0..200u32 {
        let a = node_point(seed, node(0, 0));
        let east = node_point(seed, node(1, 0));
        let south = node_point(seed, node(0, 1));
        assert!(east.x > a.x, "seed {seed}: east neighbour is not east");
        assert!(south.y > a.y, "seed {seed}: south neighbour is not south");
    }
}

#[test]
fn most_edges_exist_but_some_are_dropped() {
    let kept = (0..400)
        .filter(|&i| edge_present(7, Edge::east(node(i, 0))))
        .count();
    assert!(kept > 300, "only {kept} of 400 east edges kept");
    assert!(kept < 400, "no edge was ever dropped");
}

#[test]
fn edge_presence_is_deterministic_and_seeded() {
    let e = Edge::south(node(2, 3));
    assert_eq!(edge_present(9, e), edge_present(9, e));
    assert_ne!(edge_present(9, e), edge_present(10, e) && edge_present(9, e));
}

#[test]
fn topology_follows_the_degree() {
    assert_eq!(Topology::of(0), Topology::Absent);
    assert_eq!(Topology::of(1), Topology::DeadEnd);
    assert_eq!(Topology::of(2), Topology::Curve);
    assert_eq!(Topology::of(3), Topology::Tee);
    assert_eq!(Topology::of(4), Topology::Cross);
}

#[test]
fn node_and_edge_seeds_are_stable_and_distinct() {
    assert_eq!(node_seed(42, node(1, 2)), node_seed(42, node(1, 2)));
    assert_ne!(node_seed(42, node(1, 2)), node_seed(42, node(1, 3)));
    assert_eq!(edge_seed(42, Edge::east(node(1, 2))), edge_seed(42, Edge::east(node(1, 2))));
    assert_ne!(edge_seed(42, Edge::east(node(1, 2))), edge_seed(42, Edge::south(node(1, 2))));
    assert_ne!(node_seed(42, node(1, 2)), edge_seed(42, Edge::east(node(1, 2))));
}

#[test]
fn a_far_away_node_is_addressed_exactly_like_a_near_one() {
    // The map is unbounded: nothing about the key is special, and a
    // node 10,000 cells out is computed, not stored.
    let far = node_point(42, node(10_000, -10_000));
    assert!((far.x - 10_000.0 * LATTICE).abs() <= JITTER as f64);
    assert!((far.y + 10_000.0 * LATTICE).abs() <= JITTER as f64);
    assert_ne!(far, Pt::new(0.0, 0.0));
}
