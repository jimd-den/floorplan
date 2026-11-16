//! Walkway: the ribbon from the street sidewalk to a house's
//! entry door.
//!
//! The walkway starts at the sidewalk's outer edge (5.5 m out
//! from the centreline, at the street position nearest the
//! door) and runs to the door itself, 1.2 m wide. It is the
//! connection the grid never had: the sidewalk leads to the
//! door, not just to the house.

use crate::domain::curve::Pt;
use crate::domain::geometry::Rect;
use crate::domain::model::{House, OpeningKind};
use crate::domain::neighborhood::Street;
use crate::use_cases::neighborhood::{SIDEWALK_W, STREET_W, WALKWAY_W};

/// The entry door's local point: the EntryDoor opening on the
/// ground floor, or the front centre when there is none.
pub fn entry_door_point(house: &House, extent: Rect) -> Pt {
    let door = house
        .floors
        .first()
        .and_then(|f| f.openings.iter().find(|o| o.kind == OpeningKind::EntryDoor));
    match door {
        Some(o) => Pt::new(o.x, o.y),
        None => Pt::new(extent.w / 2.0, extent.d),
    }
}

/// The walkway: a 1.2 m ribbon from the sidewalk's outer edge
/// (5.5 m out from the centreline, at the door's street
/// position) to the entry door.
pub fn walkway_for(door: Pt, street: &Street, side: i8) -> [Pt; 4] {
    let s_door = nearest_s(street, door);
    let outward = street.samples.normal_at(s_door).scale(side as f64);
    let start = street.samples.point_at(s_door).add(outward.scale(STREET_W / 2.0 + SIDEWALK_W));
    let dir = door.sub(start).unit();
    let perp = dir.perp().scale(WALKWAY_W / 2.0);
    [start.add(perp), start.sub(perp), door.sub(perp), door.add(perp)]
}

/// The arc length of the street sample nearest a point.
fn nearest_s(street: &Street, p: Pt) -> f64 {
    let mut best = (f64::MAX, 0.0f64);
    for (i, q) in street.samples.points.iter().enumerate() {
        let d = q.dist(p);
        if d < best.0 {
            best = (d, street.samples.lengths[i]);
        }
    }
    best.1
}
