//! Lot geometry: where lots sit along a curved street.
//!
//! A lot's front arc is the street offset 4 m to the lot's
//! side; its sides run along the street normal at the lot's
//! two ends, out to the lot's depth. Lot sizes are seeded by
//! density tier. Junctions and street ends keep a clearance
//! no lot may span, and on a bulb loop only the outer side
//! (away from the bulb centre) gets lots.

use crate::domain::curve::Pt;
use crate::domain::layout::Density;
use crate::domain::neighborhood::Street;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::street_network::JUNCTION_CLEAR;

/// The s-ranges a lot may not span: the street ends, and on
/// the spine the clearance around each junction.
pub fn exclusion_zones(street: &Street) -> Vec<(f64, f64)> {
    let len = street.samples.len();
    let mut zones = vec![(0.0, JUNCTION_CLEAR), (len - JUNCTION_CLEAR, len)];
    for &j in &street.junctions {
        zones.push((j - JUNCTION_CLEAR, j + JUNCTION_CLEAR));
    }
    zones
}

/// The lot size for a density tier: frontage, then depth.
pub fn lot_size(rng: &mut Rng, density: Density) -> (f64, f64) {
    match density {
        Density::Low => (rng.range_f64(32.0, 42.0), rng.range_f64(24.0, 28.0)),
        Density::Medium => (rng.range_f64(25.0, 32.0), rng.range_f64(24.0, 26.0)),
        Density::High => (rng.range_f64(15.0, 24.0), 24.0),
    }
}

/// The side of the street that faces away from the bulb centre
/// at arc length `s` — the only side that gets lots on a loop.
/// (A street with a bulb always carries its centre.)
pub fn bulb_outer_side(street: &Street, s: f64) -> i8 {
    let centre = street.bulb_center.unwrap_or(Pt::new(0.0, 0.0));
    let p = street.samples.point_at(s);
    let n = street.samples.normal_at(s);
    if n.dot(p.sub(centre)) > 0.0 { 1 } else { -1 }
}

/// A point on the lot's front arc: the street offset 4 m to
/// the lot's side (the asphalt edge) at arc length `s`.
pub fn front_point(street: &Street, s: f64, side: i8) -> Pt {
    offset_point(street, s, side, 4.0)
}

/// The lot polygon: the front arc (the street offset 4 m to
/// the lot's side), then the back arc (the street offset
/// 4 + depth, run back the other way). Both edges follow the
/// curve — a straight back edge would cut across behind the
/// lot's own centre on a tight curve, and the polygon would
/// no longer contain the house it is meant to hold. The front
/// and back arcs have the same number of points, so the split
/// is at half the outline; the first back point is the
/// back-right corner, the last is the back-left.
pub fn lot_outline(street: &Street, s: f64, frontage: f64, depth: f64, side: i8) -> Vec<Pt> {
    let n = ((frontage / 2.0).ceil() as usize).max(1);
    let mut outline: Vec<Pt> = (0..=n)
        .map(|k| front_point(street, s + frontage * k as f64 / n as f64, side))
        .collect();
    for k in (0..=n).rev() {
        outline.push(offset_point(street, s + frontage * k as f64 / n as f64, side, 4.0 + depth));
    }
    outline
}

/// A point on the street offset `off` metres to the lot's side
/// at arc length `s`.
fn offset_point(street: &Street, s: f64, side: i8, off: f64) -> Pt {
    let p = street.samples.point_at(s);
    p.add(street.samples.normal_at(s).scale(off * side as f64))
}
