//! The neighborhood model: streets, lots, and placed houses.
//!
//! A neighborhood is a spline street network (see ADR 0010): each
//! street is a Catmull-Rom centerline, each lot is a polygon
//! fronting one street, one house stands on each lot, facing its
//! street. All coordinates are metres, x east, y south. A house's
//! local frame has its front at the south edge (largest y),
//! matching the plan generator. A placed house is a rotation plus
//! a centre: `rotation` is degrees, clockwise from south on a
//! map (0 faces south, 90 faces west, 180 faces north, -90 faces
//! east), and `center` is the placed centre of the house extent.

use crate::domain::curve::{Polyline, Pt};
use crate::domain::model::House;
use crate::domain::street_furniture::{RoadSign, StreetLamp};

/// A street: a Catmull-Rom centerline through `control`, sampled
/// into `samples`. `bulb_at` is the arc length where a turnaround
/// bulb begins (dead ends only); `bulb_center` is the bulb's
/// centre. `junctions` are the arc lengths where other streets
/// join this one — lots keep clear of them.
#[derive(Debug, Clone, PartialEq)]
pub struct Street {
    pub control: Vec<Pt>,
    pub samples: Polyline,
    pub bulb_at: Option<f64>,
    pub bulb_center: Option<Pt>,
    pub junctions: Vec<f64>,
}

/// A lot: a polygon fronting one street. The outline is the front
/// arc (on the street's offset curve) followed by the back arc
/// (the same span at the lot's depth, run back the other way);
/// the two arcs have the same number of points, so the split is
/// at half the outline, the first back point is the back-right
/// corner and the last is the back-left. Both edges follow the
/// curve — a straight back edge would cut across behind the
/// lot's own centre on a tight curve. `center_s` is the arc
/// length of the lot centre along its street; `side` is +1 on
/// the street's +normal side, -1 on the other. `frontage` is
/// the arc-length span along the street and `depth` is the
/// lot's depth along the street normal, both in metres.
#[derive(Debug, Clone, PartialEq)]
pub struct Lot {
    pub outline: Vec<Pt>,
    pub street: usize,
    pub center_s: f64,
    pub side: i8,
    pub frontage: f64,
    pub depth: f64,
}

/// A house placed on a lot, facing its street.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedHouse {
    pub house: House,
    /// Index of the lot this house stands on.
    pub lot: usize,
    /// Rotation of the house's local frame, in degrees, clockwise
    /// from south on a map (see the module preamble).
    pub rotation: f64,
    /// The placed centre of the house extent.
    pub center: Pt,
    /// The walkway ribbon corners, from the sidewalk edge to the
    /// entry door: [sidewalk-left, sidewalk-right, door-right,
    /// door-left].
    pub walkway: [Pt; 4],
}

/// A whole neighborhood.
#[derive(Debug, Clone, PartialEq)]
pub struct Neighborhood {
    pub seed: u32,
    /// The requested size, in street cells of about 85 m.
    pub blocks: (u32, u32),
    pub streets: Vec<Street>,
    pub lots: Vec<Lot>,
    pub houses: Vec<PlacedHouse>,
    /// Street lamps, in document order (their light ids are indices).
    pub lamps: Vec<StreetLamp>,
    /// Road signs, in document order.
    pub signs: Vec<RoadSign>,
}
