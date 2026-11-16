//! Lot laying: lots along curved streets (ADR 0010).
//!
//! One house per lot, placed to face its street with the
//! standard setbacks; a lot is kept only when its house fits
//! inside it, so lots and houses stay in lockstep. A lot that
//! would overlap one already placed is dropped — the
//! no-overlap invariant is enforced here, not hoped for. The
//! lot's shape lives in `lot_geometry`, its walkway in
//! `walkway`.

use crate::domain::layout::Density;
use crate::domain::neighborhood::{Lot, PlacedHouse, Street};
use crate::domain::polygon::polys_overlap;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::lot_geometry::{bulb_outer_side, exclusion_zones, lot_outline, lot_size};
use crate::use_cases::place_house::place_house;
use crate::use_cases::street_network::JUNCTION_CLEAR;

/// Lays lots and houses along every street. Lots are laid
/// street by street; a lot that overlaps one already placed is
/// dropped with its house, so the returned lots and houses
/// stay in lockstep and never overlap.
pub fn lay_lots(
    rng: &mut Rng,
    streets: &[Street],
    density: Density,
) -> (Vec<Lot>, Vec<PlacedHouse>) {
    let mut lots: Vec<Lot> = Vec::new();
    let mut houses: Vec<PlacedHouse> = Vec::new();
    for (i, street) in streets.iter().enumerate() {
        let (sl, sh) = lay_street(rng, street, i, density);
        for (lot, mut house) in sl.into_iter().zip(sh) {
            if lots.iter().any(|l| polys_overlap(&lot.outline, &l.outline)) {
                continue;
            }
            house.lot = lots.len();
            lots.push(lot);
            houses.push(house);
        }
    }
    (lots, houses)
}

/// Lays both sides of one street. Lots along one street are
/// placed back-to-back, so they cannot overlap each other; the
/// cross-street check lives in `lay_lots`.
fn lay_street(rng: &mut Rng, street: &Street, index: usize, density: Density) -> (Vec<Lot>, Vec<PlacedHouse>) {
    let mut lots = Vec::new();
    let mut houses = Vec::new();
    for side in [1i8, -1] {
        let (mut l, mut h) = lay_side(rng, street, index, density, side);
        lots.append(&mut l);
        houses.append(&mut h);
    }
    (lots, houses)
}

/// Lays lots along one side of one street, from the start of
/// the street to its end, skipping the junction clearances and
/// the inner side of any bulb loop.
fn lay_side(rng: &mut Rng, street: &Street, index: usize, density: Density, side: i8) -> (Vec<Lot>, Vec<PlacedHouse>) {
    let mut lots = Vec::new();
    let mut houses = Vec::new();
    let zones = exclusion_zones(street);
    let len = street.samples.len();
    let mut s = JUNCTION_CLEAR;
    while s < len - JUNCTION_CLEAR {
        let (frontage, depth) = lot_size(rng, density);
        // A lot must not straddle the bulb start: the approach
        // and the loop are different curve regimes, and a lot
        // spanning the join is a bent wedge that holds no house.
        if let Some(bulb_at) = street.bulb_at
            && s < bulb_at && s + frontage > bulb_at {
            s = bulb_at;
            continue;
        }
        if s + frontage > len - JUNCTION_CLEAR {
            break;
        }
        if let Some(end) = zones.iter().find(|&&(a, b)| s < b && s + frontage > a).map(|&(_, b)| b) {
            s = end;
            continue;
        }
        let center_s = s + frontage / 2.0;
        if let Some(bulb_at) = street.bulb_at
            && center_s > bulb_at && side != bulb_outer_side(street, center_s) {
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
        if let Some(house) = place_house(rng, &lot, street) {
            lots.push(lot);
            houses.push(house);
        }
        s += frontage;
    }
    (lots, houses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::placement::{front_for, house_corners, house_extent, placed_point};
    use crate::domain::polygon::point_in_poly;
    use crate::use_cases::neighborhood::WALKWAY_W;
    use crate::use_cases::street_network::plan_network;
    use crate::use_cases::walkway::entry_door_point;

    fn laid(seed: u32) -> (Vec<Lot>, Vec<PlacedHouse>) {
        let mut rng = Rng::new(seed);
        let streets = plan_network(&mut rng, (2, 2));
        lay_lots(&mut rng, &streets, Density::Low)
    }

    #[test]
    fn lots_and_houses_stay_in_lockstep() {
        for seed in 0..20u32 {
            let (lots, houses) = laid(seed);
            assert_eq!(lots.len(), houses.len(), "seed {seed}");
            assert!(!lots.is_empty(), "seed {seed}: no lots");
            for (i, h) in houses.iter().enumerate() {
                assert_eq!(h.lot, i, "seed {seed}: house {i} points at lot {}", h.lot);
            }
        }
    }

    #[test]
    fn no_two_lots_overlap() {
        for seed in 0..20u32 {
            let (lots, _) = laid(seed);
            for i in 0..lots.len() {
                for j in i + 1..lots.len() {
                    assert!(!polys_overlap(&lots[i].outline, &lots[j].outline), "seed {seed}: lots {i} and {j} overlap");
                }
            }
        }
    }

    #[test]
    fn every_house_sits_inside_its_lot() {
        for seed in 0..20u32 {
            let (lots, houses) = laid(seed);
            for h in &houses {
                let extent = house_extent(&h.house);
                for c in house_corners(h.center, h.rotation, extent) {
                    assert!(point_in_poly(c, &lots[h.lot].outline), "seed {seed}: a house corner escapes its lot");
                }
            }
        }
    }

    #[test]
    fn every_house_faces_its_street() {
        for seed in 0..20u32 {
            let mut rng = Rng::new(seed);
            let streets = plan_network(&mut rng, (2, 2));
            let (lots, houses) = lay_lots(&mut rng, &streets, Density::Low);
            for h in &houses {
                let lot = &lots[h.lot];
                let street = &streets[lot.street];
                let outward = street.samples.normal_at(lot.center_s).scale(lot.side as f64);
                let facing = front_for(h.rotation);
                let align = facing.dot(outward.scale(-1.0));
                assert!(align > 0.99, "seed {seed}: house faces away from its street (align {align:.3})");
            }
        }
    }

    #[test]
    fn every_walkway_reaches_its_door() {
        for seed in 0..20u32 {
            let (_, houses) = laid(seed);
            for h in &houses {
                let extent = house_extent(&h.house);
                let door = placed_point(h.center, h.rotation, extent, entry_door_point(&h.house, extent));
                let (d1, d2) = (h.walkway[2].dist(door), h.walkway[3].dist(door));
                assert!(d1.min(d2) < 1.2, "seed {seed}: walkway misses its door by {:.2} m", d1.min(d2));
                let w = h.walkway[0].dist(h.walkway[1]);
                assert!((w - WALKWAY_W).abs() < 0.01, "seed {seed}: walkway is {w:.2} m wide");
            }
        }
    }
}
