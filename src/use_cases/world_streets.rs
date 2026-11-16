//! Street networks for an unbounded map.
//!
//! A chunk asks for the streets near its rectangle; the lattice answers
//! with one spline per kept edge. Because an edge is addressed by its
//! two node keys, two chunks that share a street compute exactly the
//! same spline — that is what makes the map continuous across chunk
//! borders without any global state. A node with one incident street is
//! a dead end and gets a 300° turnaround bulb; three or four make a
//! T-junction or a crossroads.

use crate::domain::curve::{Polyline, Pt};
use crate::domain::geometry::Rect;
use crate::domain::lattice::{Edge, LATTICE, Node, Topology, edge_present, edge_seed, node_point};
use crate::domain::neighborhood::Street;
use crate::domain::spline::sample_catmull_rom;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::street_network::{BULB_RADIUS, SAMPLE_STEP, bulb_arc};

/// How far beyond the chunk rectangle streets are built, in metres.
/// Wide enough that a lot fronting a street just outside the border
/// still reaches into the chunk.
pub const MARGIN: f64 = 60.0;
/// Lateral jitter of the two mid control points, in metres.
const MID_JITTER: f64 = 12.0;

/// The streets that pass near `rect`, sorted by their key so the order
/// is stable. Deterministic in `(seed, rect)`. A street whose whole
/// span lies beyond the rect's reach is not returned: the node range
/// is generous by a cell, so the last ring is filtered out here.
pub fn streets_in(seed: u32, rect: Rect) -> Vec<Street> {
    let mut edges = Vec::new();
    for node in nodes_near(rect) {
        for edge in [Edge::east(node), Edge::south(node)] {
            if edge_present(seed, edge) {
                edges.push(edge);
            }
        }
    }
    edges.sort();
    edges
        .into_iter()
        .map(|e| street_for(seed, e))
        .filter(|s| reaches(s, rect))
        .collect()
}

/// True when any control point of `s` is within the rect grown by the
/// margin. Control points bound the spline (the curve stays inside
/// their hull), so this never drops a street that touches the chunk.
fn reaches(s: &Street, rect: Rect) -> bool {
    s.control.iter().any(|p| {
        p.x >= rect.x - MARGIN
            && p.x <= rect.right() + MARGIN
            && p.y >= rect.y - MARGIN
            && p.y <= rect.bottom() + MARGIN
    })
}

/// Every node whose edges could reach into `rect` (the rect grown by
/// one lattice cell plus the margin).
fn nodes_near(rect: Rect) -> Vec<Node> {
    let pad = LATTICE + MARGIN;
    let i0 = ((rect.x - pad) / LATTICE).floor() as i32;
    let i1 = ((rect.right() + pad) / LATTICE).ceil() as i32;
    let j0 = ((rect.y - pad) / LATTICE).floor() as i32;
    let j1 = ((rect.bottom() + pad) / LATTICE).ceil() as i32;
    let mut out = Vec::new();
    for j in j0..=j1 {
        for i in i0..=i1 {
            out.push(Node::new(i, j));
        }
    }
    out
}

/// How many kept streets meet at `node`: its own two edges plus the
/// west and north neighbours' edges into it.
pub fn node_degree(seed: u32, node: Node) -> u32 {
    let mut degree = 0;
    if edge_present(seed, Edge::east(node)) {
        degree += 1;
    }
    if edge_present(seed, Edge::south(node)) {
        degree += 1;
    }
    if edge_present(seed, Edge::east(node.offset(-1, 0))) {
        degree += 1;
    }
    if edge_present(seed, Edge::south(node.offset(0, -1))) {
        degree += 1;
    }
    degree
}

/// What the network does at `node`.
pub fn topology_at(seed: u32, node: Node) -> Topology {
    Topology::of(node_degree(seed, node))
}

/// One edge as a street: a Catmull-Rom centerline through the two node
/// points and two jittered midpoints, ending in a bulb when the far
/// node is a dead end.
fn street_for(seed: u32, edge: Edge) -> Street {
    // Always run the street out of the higher-degree node, so a dead
    // end is at the far end and only the append-a-bulb case exists.
    let (from, to) = if node_degree(seed, edge.a) == 1 && node_degree(seed, edge.b) > 1 {
        (edge.b, edge.a)
    } else {
        (edge.a, edge.b)
    };
    let (a, b) = (node_point(seed, from), node_point(seed, to));
    let control = control_points(seed, edge, a, b);
    let mut samples = sample_catmull_rom(&control, SAMPLE_STEP);
    let mut bulb_at = None;
    let mut bulb_center = None;
    if node_degree(seed, to) == 1 {
        let end = *samples.last().unwrap();
        let tangent = samples[samples.len() - 2].sub(end).scale(-1.0).unit();
        let mut rng = Rng::new(edge_seed(seed, edge));
        let left = rng.chance(0.5);
        let side = if left { tangent.perp() } else { tangent.perp().scale(-1.0) };
        let centre = end.add(side.scale(BULB_RADIUS));
        bulb_at = Some(Polyline::new(samples.clone()).len());
        bulb_center = Some(centre);
        samples.extend(bulb_arc(end, centre, left));
    }
    Street {
        control,
        samples: Polyline::new(samples),
        bulb_at,
        bulb_center,
        junctions: Vec::new(),
    }
}

/// The four control points: both nodes, and two midpoints pushed off
/// the chord by the edge's own seeded jitter.
fn control_points(seed: u32, edge: Edge, a: Pt, b: Pt) -> Vec<Pt> {
    let mut rng = Rng::new(edge_seed(seed, edge));
    let normal = b.sub(a).unit().perp();
    let push = |rng: &mut Rng, t: f64| {
        a.lerp(b, t).add(normal.scale(rng.range_f64(-MID_JITTER, MID_JITTER)))
    };
    vec![a, push(&mut rng, 0.35), push(&mut rng, 0.7), b]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64) -> Rect {
        Rect::new(x, y, LATTICE, LATTICE)
    }

    #[test]
    fn a_chunk_of_streets_is_deterministic() {
        assert_eq!(streets_in(42, rect(0.0, 0.0)), streets_in(42, rect(0.0, 0.0)));
        assert_ne!(streets_in(42, rect(0.0, 0.0)), streets_in(43, rect(0.0, 0.0)));
    }

    #[test]
    fn a_chunk_has_streets_and_they_are_long_enough() {
        let streets = streets_in(42, rect(0.0, 0.0));
        assert!(!streets.is_empty(), "no streets near the origin");
        for s in &streets {
            assert!(s.samples.len() >= 40.0, "street only {:.1} m", s.samples.len());
        }
    }
}
