//! The world lattice: the unbounded frame the streets are drawn on.
//!
//! A node sits near every multiple of `LATTICE` metres, jittered by a
//! seed-derived offset on a 1 m grid. Two nodes are joined by an east
//! or south edge unless the seed drops it, so the lattice is a full
//! connected grid with holes: holes make dead ends (cul-de-sacs),
//! degree-3 nodes read as T-junctions, degree-4 as crossroads, and the
//! 4-cycles are loops. Nothing here is bounded by a block count — a
//! node key is any `(i32, i32)`, so the map is infinite in both axes.
//!
//! Every value is a pure function of `(seed, key)`: a chunk can be
//! rebuilt alone, and two chunks that share a street compute exactly
//! the same street because they share the node keys.

use crate::domain::curve::Pt;

/// Distance between lattice nodes, in metres.
pub const LATTICE: f64 = 140.0;
/// Node jitter, in metres, on a 1 m grid: -`JITTER`..=`JITTER`.
pub const JITTER: i32 = 30;
/// Percent of lattice edges that exist (the rest become dead ends).
pub const EDGE_PERCENT: u64 = 92;

/// A lattice node, addressed by integer cell coordinates. Any pair of
/// `i32` is a valid node, which is what makes the map unbounded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node {
    pub i: i32,
    pub j: i32,
}

impl Node {
    pub fn new(i: i32, j: i32) -> Self {
        Self { i, j }
    }

    /// The node `di` cells east and `dj` cells south of this one.
    pub fn offset(self, di: i32, dj: i32) -> Node {
        Node::new(self.i + di, self.j + dj)
    }
}

/// One lattice edge: from `a` to its east or south neighbour `b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge {
    pub a: Node,
    pub b: Node,
}

impl Edge {
    /// The east edge leaving `a`, when the seed keeps it.
    pub fn east(a: Node) -> Edge {
        Edge { a, b: a.offset(1, 0) }
    }

    /// The south edge leaving `a`, when the seed keeps it.
    pub fn south(a: Node) -> Edge {
        Edge { a, b: a.offset(0, 1) }
    }
}

/// What a node looks like in the network, by how many streets meet it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topology {
    /// No street touches this node; it is not drawn.
    Absent,
    /// One street ends here: a cul-de-sac.
    DeadEnd,
    /// A street passes through: a bend.
    Curve,
    /// Three streets: a T-junction.
    Tee,
    /// Four streets: a crossroads.
    Cross,
}

impl Topology {
    /// The topology of a node with `degree` incident streets.
    pub fn of(degree: u32) -> Topology {
        match degree {
            0 => Topology::Absent,
            1 => Topology::DeadEnd,
            2 => Topology::Curve,
            3 => Topology::Tee,
            _ => Topology::Cross,
        }
    }

    /// Lower-case name for the JSON writer and reports.
    pub fn id(&self) -> &'static str {
        match self {
            Topology::Absent => "absent",
            Topology::DeadEnd => "dead_end",
            Topology::Curve => "curve",
            Topology::Tee => "tee",
            Topology::Cross => "cross",
        }
    }
}

/// The world position of a node: the lattice point plus a seeded
/// offset of `-JITTER`..=`JITTER` metres on each axis, quantized to
/// 1 m so the variety count is finite.
pub fn node_point(seed: u32, node: Node) -> Pt {
    let h = mix(seed, node.i, node.j, 0x01);
    let span = (2 * JITTER + 1) as u64;
    let jx = (h % span) as i32 - JITTER;
    let jy = ((h >> 32) % span) as i32 - JITTER;
    Pt::new(
        node.i as f64 * LATTICE + jx as f64,
        node.j as f64 * LATTICE + jy as f64,
    )
}

/// True when the seed keeps this edge. Dropping an edge is what
/// turns a crossroads into a T-junction and a curve into a dead end.
pub fn edge_present(seed: u32, edge: Edge) -> bool {
    // Edges are addressed by their north-west end, so the east and
    // south edges out of one node are distinct decisions.
    let salt = if edge.b.i > edge.a.i { 0x11 } else { 0x12 };
    let h = mix(seed, edge.a.i, edge.a.j, salt);
    h % 100 < EDGE_PERCENT
}

/// A stable 32-bit key for a node, for the per-node decisions that
/// follow (lot layout, house seeds, furniture).
pub fn node_seed(seed: u32, node: Node) -> u32 {
    (mix(seed, node.i, node.j, 0x21) >> 32) as u32
}

/// A stable 32-bit key for an edge.
pub fn edge_seed(seed: u32, edge: Edge) -> u32 {
    let salt = if edge.b.i > edge.a.i { 0x31 } else { 0x32 };
    (mix(seed, edge.a.i, edge.a.j, salt) >> 32) as u32
}

/// FNV-1a over the seed and the key, with a salt so that a node, an
/// east edge, and a south edge never share a stream. Pure, so a chunk
/// rebuilt alone sees exactly the same values.
fn mix(seed: u32, i: i32, j: i32, salt: u64) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325 ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for byte in seed
        .to_le_bytes()
        .iter()
        .chain(i.to_le_bytes().iter())
        .chain(j.to_le_bytes().iter())
    {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    // Final avalanche, so low bits are not biased by the byte order.
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51_afd7_ed55_8ccd);
    hash ^ (hash >> 29)
}
