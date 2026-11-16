//! Road signs along a street.
//!
//! Three kinds, each with a rule a driver can rely on: a stop sign at
//! every junction approach, on the right-hand sidewalk and facing the
//! driver; a street-name sign on the same corner's other side; and
//! speed-limit signs at regular intervals, facing the traffic that
//! travels with the street's arc length. A street end that is not a
//! junction (a cul-de-sac or the end of a road) gets no stop sign.
//! Signs are placed from the street alone, so two chunks that share a
//! street place the same signs.

use crate::domain::neighborhood::Street;
use crate::domain::placement::rotation_for;
use crate::domain::street_furniture::{RoadSign, SignKind};

/// Lateral distance of a sign post from the centerline, in metres:
/// the middle of the 1.5 m sidewalk.
pub const SIGN_OFFSET: f64 = 4.6;
/// How far a stop sign stands from the junction it serves, in
/// metres: just outside the lot clearance.
pub const SIGN_CLEAR: f64 = 14.0;
/// Arc length of the first speed-limit sign, in metres.
pub const SPEED_SIGN_FIRST: f64 = 55.0;
/// Distance between speed-limit signs, in metres.
pub const SPEED_SIGN_SPACING: f64 = 110.0;
/// The speed limits a neighborhood street can carry, mph.
pub const SPEED_LIMITS: [u32; 3] = [15, 20, 25];
/// Street names, chosen by the street's seed.
pub const STREET_NAMES: [&str; 8] =
    ["OAK", "MAPLE", "ELM", "PINE", "CEDAR", "BIRCH", "WILLOW", "CHESTNUT"];

/// What a street's end is: a junction (a stop sign belongs there) or
/// not (a cul-de-sac bulb or the end of a road).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndKind {
    Junction,
    DeadEnd,
}

/// The two ends of a street, in arc-length order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreetEnds {
    pub start: EndKind,
    pub end: EndKind,
}

/// Every sign along one street. `seed` picks the street name and the
/// speed limit, so it must be the street's own stable key.
pub fn signs_along(street: &Street, seed: u32, ends: StreetEnds) -> Vec<RoadSign> {
    let end = street.bulb_at.unwrap_or_else(|| street.samples.len());
    let name = format!("{} ST", STREET_NAMES[seed as usize % STREET_NAMES.len()]);
    let limit = SPEED_LIMITS[seed as usize % SPEED_LIMITS.len()];
    let mut signs = Vec::new();
    if ends.start == EndKind::Junction {
        signs.push(sign(street, SIGN_CLEAR, -1.0, 1.0, SignKind::Stop, "STOP".into()));
        signs.push(sign(street, SIGN_CLEAR, 1.0, 1.0, SignKind::StreetName, name.clone()));
    }
    if ends.end == EndKind::Junction && end > 2.0 * SIGN_CLEAR {
        signs.push(sign(street, end - SIGN_CLEAR, 1.0, -1.0, SignKind::Stop, "STOP".into()));
        signs.push(sign(street, end - SIGN_CLEAR, -1.0, -1.0, SignKind::StreetName, name));
    }
    let mut s = SPEED_SIGN_FIRST;
    while s < end - SIGN_CLEAR {
        signs.push(sign(street, s, 1.0, -1.0, SignKind::SpeedLimit, limit.to_string()));
        s += SPEED_SIGN_SPACING;
    }
    signs
}

/// One sign: `side` is the lateral side of the post, `along` the
/// direction the plate faces relative to the street's tangent (+1
/// faces a driver arriving at the start, -1 one arriving at the end).
fn sign(street: &Street, s: f64, side: f64, along: f64, kind: SignKind, text: String) -> RoadSign {
    let position = street
        .samples
        .point_at(s)
        .add(street.samples.normal_at(s).scale(SIGN_OFFSET * side));
    let rotation = rotation_for(street.samples.tangent_at(s).scale(along));
    RoadSign { position, rotation, kind, text }
}
