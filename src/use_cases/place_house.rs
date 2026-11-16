//! House placement: one house on one lot.
//!
//! The style follows the chord of the lot's front arc (narrow
//! lots get Townhouse). The house is centred on the lot and
//! turned to face its street, with the front setback from the
//! asphalt edge. A house that does not fit its lot is tried
//! once as a Townhouse, then the lot is dropped — placement
//! never forces a house that does not fit.

use crate::domain::model::House;
use crate::domain::neighborhood::{Lot, PlacedHouse, Street};
use crate::domain::placement::{house_corners, house_extent, placed_point, rotation_for};
use crate::domain::polygon::point_in_poly;
use crate::domain::style::Style;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::generate::generate_house;
use crate::use_cases::lot_geometry::front_point;
use crate::use_cases::neighborhood::{FRONT_SETBACK, STREET_W};
use crate::use_cases::walkway::{entry_door_point, walkway_for};

/// Places a house on a lot: the style follows the chord of
/// the front arc (narrow lots get Townhouse), and a house that
/// does not fit its lot is tried once as a Townhouse, then
/// the lot is dropped.
pub fn place_house(rng: &mut Rng, lot: &Lot, street: &Street) -> Option<PlacedHouse> {
    let chord = front_point(street, lot.center_s - lot.frontage / 2.0, lot.side)
        .dist(front_point(street, lot.center_s + lot.frontage / 2.0, lot.side));
    let style = style_for(rng, chord);
    let seed = rng.next_u32();
    let house = generate_house(seed, style);
    if let Some(placed) = try_place(&house, lot, street) {
        return Some(placed);
    }
    if style == Style::Townhouse {
        return None;
    }
    try_place(&generate_house(seed, Style::Townhouse), lot, street)
}

/// The style for a lot, by the chord of its front arc.
fn style_for(rng: &mut Rng, chord: f64) -> Style {
    if chord < 15.0 {
        Style::Townhouse
    } else if chord < 40.0 {
        *rng.pick(&[Style::Cottage, Style::Townhouse])
    } else {
        *rng.pick(&[Style::Ranch, Style::Colonial, Style::Modern, Style::Craftsman])
    }
}

/// Tries to place a house on its lot: centred on the lot,
/// facing the street, front setback from the asphalt edge. The
/// house is kept only when all four corners sit inside the lot.
/// The lot index is 0 here; the caller assigns the final index
/// once the lot is known to be kept.
fn try_place(house: &House, lot: &Lot, street: &Street) -> Option<PlacedHouse> {
    let extent = house_extent(house);
    let outward = street.samples.normal_at(lot.center_s).scale(lot.side as f64);
    let center = street.samples.point_at(lot.center_s)
        .add(outward.scale(STREET_W / 2.0 + FRONT_SETBACK + extent.d / 2.0));
    let rotation = rotation_for(outward.scale(-1.0));
    let corners = house_corners(center, rotation, extent);
    if !corners.iter().all(|&c| point_in_poly(c, &lot.outline)) {
        return None;
    }
    let door = placed_point(center, rotation, extent, entry_door_point(house, extent));
    let walkway = walkway_for(door, street, lot.side);
    Some(PlacedHouse { house: house.clone(), lot: 0, rotation, center, walkway })
}
