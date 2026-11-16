//! Street lamps along an unbounded street.
//!
//! Lamps are placed by arc length: a pole on the sidewalk every
//! `LAMP_SPACING` metres, alternating sides so both sidewalks are
//! lit, with the arm turned toward the road. A lamp is skipped once
//! the street reaches its turnaround bulb — the loop is for turning,
//! not for standing on. Everything is a pure function of the street,
//! so two chunks that share a street place the same lamps.

use crate::domain::neighborhood::Street;
use crate::domain::placement::rotation_for;
use crate::domain::street_furniture::StreetLamp;

/// Distance between consecutive lamps, in metres. Sides alternate,
/// so each sidewalk gets a lamp every `2 * LAMP_SPACING`.
pub const LAMP_SPACING: f64 = 22.0;
/// Arc length of the first lamp, in metres.
pub const LAMP_FIRST: f64 = 16.0;
/// Lateral distance of the pole from the centerline, in metres:
/// the middle of the 1.5 m sidewalk (4.0..5.5 m).
pub const LAMP_OFFSET: f64 = 4.75;
/// No lamp within this arc length of a street's end, in metres.
pub const LAMP_END_CLEAR: f64 = 8.0;

/// Every lamp along one street, in arc-length order. Deterministic in
/// the street's own geometry.
pub fn lamps_along(street: &Street) -> Vec<StreetLamp> {
    let end = street.bulb_at.unwrap_or_else(|| street.samples.len());
    let stop = end - LAMP_END_CLEAR;
    let mut lamps = Vec::new();
    let mut s = LAMP_FIRST;
    let mut k = 0usize;
    while s <= stop {
        // Alternate sides, so the two sidewalks take turns.
        let side = if k.is_multiple_of(2) { 1.0 } else { -1.0 };
        let position = street
            .samples
            .point_at(s)
            .add(street.samples.normal_at(s).scale(LAMP_OFFSET * side));
        // The arm reaches across the road, away from the pole's side.
        let rotation = rotation_for(street.samples.normal_at(s).scale(-side));
        lamps.push(StreetLamp { position, rotation });
        s += LAMP_SPACING;
        k += 1;
    }
    lamps
}
