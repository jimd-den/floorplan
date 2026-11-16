//! Lots along an unbounded street.
//!
//! A street's lots are laid from its own seed alone — frontage and
//! depth by density, stepping along the arc length, clear of both
//! ends, and only on the outer side of a turnaround bulb. Because the
//! street is identical in every chunk that touches it, so is its lot
//! list; a chunk keeps the lots whose front centre falls in its own
//! rectangle, which partitions the world's lots exactly once each.

use crate::domain::curve::Pt;
use crate::domain::layout::Density;
use crate::domain::neighborhood::{Lot, PlacedHouse, Street};
use crate::domain::polygon::polys_overlap;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::lot_geometry::{bulb_outer_side, lot_outline, lot_size};
use crate::use_cases::place_house::place_house;
use crate::use_cases::street_network::JUNCTION_CLEAR;

/// Every lot and house along one street, in arc-length order, both
/// sides interleaved (+normal side first). Deterministic in the
/// street's own geometry.
pub fn lots_along(street: &Street, index: usize, density: Density) -> (Vec<Lot>, Vec<PlacedHouse>) {
    let mut rng = Rng::new(street_seed(street));
    let mut lots = Vec::new();
    let mut houses = Vec::new();
    for side in [1i8, -1] {
        let (mut l, mut h) = side_lots(&mut rng, street, index, density, side);
        // Re-index onto the street's combined lot list: both sides
        // number their houses from zero on their own.
        let base = lots.len();
        for (k, house) in h.iter_mut().enumerate() {
            house.lot = base + k;
        }
        lots.append(&mut l);
        houses.append(&mut h);
    }
    (lots, houses)
}

/// Lays every street, then drops a lot that would overlap a lot on a
/// street with a smaller stable key. Two streets meeting at a junction
/// both keep clear of the node, but their deep lots can still meet in
/// the corner; this resolves that once, by key, so two chunks that see
/// the same streets make the same decision.
pub fn lay_streets(streets: &[Street], density: Density) -> (Vec<Lot>, Vec<PlacedHouse>) {
    let mut order: Vec<usize> = (0..streets.len()).collect();
    order.sort_by_key(|&i| {
        let p = streets[i].control[0];
        (street_seed(&streets[i]), p.x as i64, p.y as i64)
    });
    let mut lots: Vec<Lot> = Vec::new();
    let mut houses: Vec<PlacedHouse> = Vec::new();
    let mut boxes: Vec<([f64; 2], [f64; 2])> = Vec::new();
    for index in order {
        let (street_lots, street_houses) = lots_along(&streets[index], index, density);
        for (lot, mut house) in street_lots.into_iter().zip(street_houses) {
            let b = bbox(&lot.outline);
            let clash = boxes
                .iter()
                .zip(lots.iter())
                .any(|(other, kept)| boxes_meet(b, *other) && polys_overlap(&lot.outline, &kept.outline));
            if clash {
                continue;
            }
            house.lot = lots.len();
            boxes.push(b);
            lots.push(lot);
            houses.push(house);
        }
    }
    (lots, houses)
}

/// The axis-aligned bounds of a polygon: (min, max).
fn bbox(poly: &[Pt]) -> ([f64; 2], [f64; 2]) {
    let mut lo = [f64::MAX; 2];
    let mut hi = [f64::MIN; 2];
    for p in poly {
        lo[0] = lo[0].min(p.x);
        lo[1] = lo[1].min(p.y);
        hi[0] = hi[0].max(p.x);
        hi[1] = hi[1].max(p.y);
    }
    (lo, hi)
}

/// True when two boxes share area. Touching boxes do not meet, which
/// matches `polys_overlap`'s treatment of a shared edge.
fn boxes_meet(a: ([f64; 2], [f64; 2]), b: ([f64; 2], [f64; 2])) -> bool {
    a.0[0] < b.1[0] && b.0[0] < a.1[0] && a.0[1] < b.1[1] && b.0[1] < a.1[1]
}

/// One side of one street, from the clearance at its start to the
/// clearance at its end. A lot that would straddle the start of a
/// turnaround bulb is skipped, and on the loop only the outer side
/// carries lots (the inner side is the bulb itself).
fn side_lots(
    rng: &mut Rng,
    street: &Street,
    index: usize,
    density: Density,
    side: i8,
) -> (Vec<Lot>, Vec<PlacedHouse>) {
    let mut lots = Vec::new();
    let mut houses = Vec::new();
    let len = street.samples.len();
    let mut s = JUNCTION_CLEAR;
    while s < len - JUNCTION_CLEAR {
        let (frontage, depth) = lot_size(rng, density);
        if let Some(bulb_at) = street.bulb_at
            && s < bulb_at
            && s + frontage > bulb_at
        {
            s = bulb_at;
            continue;
        }
        if s + frontage > len - JUNCTION_CLEAR {
            break;
        }
        let center_s = s + frontage / 2.0;
        if let Some(bulb_at) = street.bulb_at
            && center_s > bulb_at
            && side != bulb_outer_side(street, center_s)
        {
            s += frontage;
            continue;
        }
        let lot = Lot {
            outline: lot_outline(street, s, frontage, depth, side),
            street: index,
            center_s,
            side,
            frontage,
            depth,
        };
        if let Some(mut house) = place_house(rng, &lot, street) {
            house.lot = lots.len();
            lots.push(lot);
            houses.push(house);
        }
        s += frontage;
    }
    (lots, houses)
}

/// A stable key for a street, from its own control points. The same
/// street built from two chunks hashes to the same key, so its lots
/// and houses come out identical.
pub fn street_seed(street: &Street) -> u32 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for p in &street.control {
        for byte in (p.x as i64)
            .to_le_bytes()
            .iter()
            .chain((p.y as i64).to_le_bytes().iter())
        {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
    }
    (hash >> 32) as u32
}

/// True when `p` is in the half-open rectangle: every point of the
/// plane belongs to exactly one chunk.
pub fn inside(rect: crate::domain::geometry::Rect, p: crate::domain::curve::Pt) -> bool {
    p.x >= rect.x && p.x < rect.right() && p.y >= rect.y && p.y < rect.bottom()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_street_key_is_stable_and_seeded_by_its_geometry() {
        use crate::domain::geometry::Rect;
        use crate::use_cases::world_streets::streets_in;
        let a = streets_in(42, Rect::new(0.0, 0.0, 140.0, 140.0));
        let b = streets_in(42, Rect::new(0.0, 0.0, 140.0, 140.0));
        assert_eq!(street_seed(&a[0]), street_seed(&b[0]));
        assert_ne!(street_seed(&a[0]), street_seed(&a[1]));
    }
}
