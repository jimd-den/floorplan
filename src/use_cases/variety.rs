//! Plan, neighborhood, and chunk variety: pure FNV-1a hashing over
//! the seed-free domain structures. The combinatorial floors live in
//! `use_cases::variety_bounds`.
//!
//! Nothing here imports `adapters/`: the hashes walk the domain data
//! directly with zero string allocation (NFR-6). Seeds are excluded on
//! purpose so distinct hashes mean distinct plans or neighborhoods.

use crate::domain::geometry::{Axis, Rect};
use crate::domain::model::House;
use crate::domain::neighborhood::Neighborhood;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

fn feed_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn feed_u64(hash: u64, v: u64) -> u64 {
    feed_bytes(hash, &v.to_le_bytes())
}

fn feed_f64(hash: u64, v: f64) -> u64 {
    let q = (v * 100.0).round() as i64;
    feed_u64(hash, q as u64)
}

fn feed_rect(mut hash: u64, r: Rect) -> u64 {
    hash = feed_f64(hash, r.x);
    hash = feed_f64(hash, r.y);
    hash = feed_f64(hash, r.w);
    feed_f64(hash, r.d)
}

fn feed_house(mut hash: u64, house: &House) -> u64 {
    hash = feed_bytes(hash, house.style.id().as_bytes());
    for r in &house.footprint.rects {
        hash = feed_rect(hash, *r);
    }
    if let Some(p) = house.footprint.porch {
        hash = feed_rect(feed_u64(hash, 1), p);
    }
    if let Some(g) = house.footprint.garage {
        hash = feed_rect(feed_u64(hash, 2), g);
    }
    for floor in &house.floors {
        hash = feed_u64(hash, floor.level as u64);
        for room in &floor.rooms {
            hash = feed_u64(hash, room.id as u64);
            hash = feed_bytes(hash, room.program.name().as_bytes());
            hash = feed_rect(hash, room.rect);
        }
        for o in &floor.openings {
            hash = feed_u64(hash, o.id as u64);
            hash = feed_f64(hash, o.x);
            hash = feed_f64(hash, o.y);
            hash = feed_f64(hash, o.width);
            hash = feed_u64(hash, matches!(o.axis, Axis::Horizontal) as u64);
        }
    }
    hash
}

/// Hashes a plan. Same plan, same hash; the seed plays no part.
pub fn plan_hash(house: &House) -> u64 {
    feed_house(FNV_OFFSET, house)
}

/// Hashes a neighborhood (seed-free). Same neighborhood, same hash.
pub fn neighborhood_hash(n: &Neighborhood) -> u64 {
    let mut hash = FNV_OFFSET;
    hash = feed_u64(hash, n.blocks.0 as u64);
    hash = feed_u64(hash, n.blocks.1 as u64);
    for s in &n.streets {
        for c in &s.control {
            hash = feed_f64(hash, c.x);
            hash = feed_f64(hash, c.y);
        }
        hash = feed_u64(hash, s.bulb_at.is_some() as u64);
        if let Some(c) = s.bulb_center {
            hash = feed_f64(hash, c.x);
            hash = feed_f64(hash, c.y);
        }
    }
    for l in &n.lots {
        for p in &l.outline {
            hash = feed_f64(hash, p.x);
            hash = feed_f64(hash, p.y);
        }
        hash = feed_u64(hash, l.street as u64);
    }
    for h in &n.houses {
        hash = feed_u64(hash, h.lot as u64);
        hash = feed_f64(hash, h.rotation);
        hash = feed_f64(hash, h.center.x);
        hash = feed_f64(hash, h.center.y);
        for p in &h.walkway {
            hash = feed_f64(hash, p.x);
            hash = feed_f64(hash, p.y);
        }
        hash = feed_house(hash, &h.house);
    }
    hash
}

/// A cheap structural hash of one chunk: the jittered points of the
/// cell's four corners and the four cell edges. This is exactly the
/// tuple the variety bound counts, so it can be sampled tens of
/// thousands of times; `neighborhood_hash` stays the expensive
/// ground truth for a whole chunk.
pub fn chunk_layout_hash(seed: u32, cx: i32, cz: i32) -> u64 {
    use crate::domain::lattice::{Edge, Node, edge_present, node_point};
    let mut hash = FNV_OFFSET;
    for j in 0..=1 {
        for i in 0..=1 {
            let p = node_point(seed, Node::new(cx + i, cz + j));
            hash = feed_f64(hash, p.x);
            hash = feed_f64(hash, p.y);
        }
    }
    let edges = [
        Edge::east(Node::new(cx, cz)),
        Edge::east(Node::new(cx, cz + 1)),
        Edge::south(Node::new(cx, cz)),
        Edge::south(Node::new(cx + 1, cz)),
    ];
    for edge in edges {
        hash = feed_u64(hash, edge.a.i as u64);
        hash = feed_u64(hash, edge.a.j as u64);
        hash = feed_u64(hash, edge_present(seed, edge) as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;

    #[test]
    fn same_house_gives_the_same_hash() {
        let house = generate_house(1, Style::Ranch);
        assert_eq!(plan_hash(&house), plan_hash(&house));
    }

    #[test]
    fn different_houses_give_different_hashes() {
        assert_ne!(
            plan_hash(&generate_house(1, Style::Ranch)),
            plan_hash(&generate_house(2, Style::Ranch))
        );
        assert_ne!(
            plan_hash(&generate_house(1, Style::Ranch)),
            plan_hash(&generate_house(1, Style::Modern))
        );
    }

    #[test]
    fn the_hash_is_never_zero_for_real_input() {
        assert_ne!(plan_hash(&generate_house(42, Style::Ranch)), 0);
    }

    #[test]
    fn same_neighborhood_gives_the_same_hash() {
        let n = crate::use_cases::neighborhood::generate_neighborhood(1, (2, 2));
        assert_eq!(neighborhood_hash(&n), neighborhood_hash(&n));
    }

    #[test]
    fn different_neighborhoods_give_different_hashes() {
        let a = crate::use_cases::neighborhood::generate_neighborhood(1, (2, 2));
        let b = crate::use_cases::neighborhood::generate_neighborhood(2, (2, 2));
        assert_ne!(neighborhood_hash(&a), neighborhood_hash(&b));
    }
}
