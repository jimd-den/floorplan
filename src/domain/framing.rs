//! Framing: the structural frame of a house — wall framing (bottom
//! plate, studs, top plate), floor framing (joists + rim), and roof
//! framing (rafters + ridge beam). This is the skeleton the user
//! meant: the load-bearing structure, not the circulation graph
//! (`domain::skeleton`, which only proves rooms connect). Walls,
//! doors, and windows attach to the frame: every opening sits in a
//! framed bay with a header above it and trimmer studs at its
//! sides. The frame is built from the floor plan, so it is
//! deterministic in the seed and carries the permutations.

use crate::domain::geometry::{Axis, Rect, Segment};

/// Stud spacing, in metres (16 in on centre).
pub const STUD_SPACING: f64 = 0.4;
/// Floor-joist spacing, in metres.
pub const JOIST_SPACING: f64 = 0.4;
/// Rafter spacing, in metres.
pub const RAFTER_SPACING: f64 = 0.6;
/// A stud: 0.04 m thick along the wall, 0.09 m deep (2x4 nominal).
pub const STUD_T: f64 = 0.04;
pub const STUD_D: f64 = 0.09;
/// A plate (bottom or top): 0.04 m tall.
pub const PLATE_H: f64 = 0.04;
/// A floor joist: 0.2 m deep.
pub const JOIST_D: f64 = 0.2;
/// A rafter: 0.15 m deep.
pub const RAFTER_D: f64 = 0.15;
/// The top plate is doubled in real framing.
pub const TOP_PLATES: usize = 2;

/// A framed opening in a wall: the door or window it holds, the
/// span along the wall, and the header height above the floor. The
/// header carries the load over the opening; a trimmer stud stands
/// at each side.
#[derive(Debug, Clone, PartialEq)]
pub struct FramedOpening {
    pub opening_id: u32,
    pub lo: f64,
    pub hi: f64,
    pub header_y: f64,
}

/// One framed wall: the segment it stands on, the stud offsets
/// along it, and the framed openings. The bottom plate runs the
/// whole segment at floor level; the top plate (doubled) runs it at
/// ceiling height. Studs skip the openings — a trimmer stands at
/// each opening side instead. `base` is the floor's height.
#[derive(Debug, Clone, PartialEq)]
pub struct WallFrame {
    pub segment: Segment,
    pub studs: Vec<f64>,
    pub openings: Vec<FramedOpening>,
    pub base: f32,
}

/// One framed floor: the joists spanning the short direction plus
/// the rim joist around the edge. `direction` is the axis the
/// joists run along; the offsets are along the other axis. `base`
/// is the floor's height.
#[derive(Debug, Clone, PartialEq)]
pub struct FloorFrame {
    pub rect: Rect,
    pub joists: Vec<f64>,
    pub direction: Axis,
    pub base: f32,
}

/// The roof frame: the ridge beam and the rafters from ridge to
/// eave. `span` is the rafter run (ridge to eave, in metres).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RoofFrame {
    pub ridge: Segment,
    pub rafters: Vec<f64>,
    pub span: f64,
}

/// The structural frame of a house.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Framing {
    pub wall_frames: Vec<WallFrame>,
    pub floor_frames: Vec<FloorFrame>,
    pub roof_frame: RoofFrame,
}

/// Stud offsets along a wall segment of length `len`, skipping the
/// `openings` (each a `(lo, hi)` span along the segment). Studs sit
/// every `STUD_SPACING`, starting at the segment start, and a
/// trimmer stud stands at each opening side. The far end always
/// carries a stud (the corner/end stud). The offsets are quantized
/// to 0.1 m like every other seeded dimension.
pub fn stud_positions(len: f64, openings: &[(f64, f64)]) -> Vec<f64> {
    let mut studs: Vec<f64> = vec![];
    let mut at = 0.0;
    while at <= len + 1e-9 {
        let inside = openings.iter().any(|&(lo, hi)| at > lo - 1e-9 && at < hi + 1e-9);
        if !inside {
            studs.push(crate::domain::geometry::quant(at));
        }
        at += STUD_SPACING;
    }
    // A trimmer stud at each opening side, so the header has
    // something to bear on, and a stud at the far end. The low
    // trimmer floors to the 0.1 m grid (never into the opening);
    // the high trimmer ceils.
    for &(lo, hi) in openings {
        let q_lo = (lo * 10.0).floor() / 10.0;
        let q_hi = (hi * 10.0).ceil() / 10.0;
        for q in [q_lo, q_hi] {
            if !studs.iter().any(|s| (s - q).abs() < 1e-9) {
                studs.push(q);
            }
        }
    }
    let end = crate::domain::geometry::quant(len);
    if !studs.iter().any(|s| (s - end).abs() < 1e-9) {
        studs.push(end);
    }
    studs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    studs
}

/// Joist offsets across a span of length `len`: one every
/// `JOIST_SPACING`, starting at the span start, plus the rim at the
/// far end. Quantized to 0.1 m.
pub fn joist_positions(len: f64) -> Vec<f64> {
    let mut joists: Vec<f64> = vec![];
    let mut at = 0.0;
    while at <= len + 1e-9 {
        joists.push(crate::domain::geometry::quant(at));
        at += JOIST_SPACING;
    }
    let end = crate::domain::geometry::quant(len);
    if !joists.iter().any(|j| (j - end).abs() < 1e-9) {
        joists.push(end);
    }
    joists
}

/// Rafter offsets along a ridge of length `len`: one every
/// `RAFTER_SPACING`, starting at the ridge start, plus one at the
/// far (gable) end. Quantized.
pub fn rafter_positions(len: f64) -> Vec<f64> {
    let mut rafters: Vec<f64> = vec![];
    let mut at = 0.0;
    while at <= len + 1e-9 {
        rafters.push(crate::domain::geometry::quant(at));
        at += RAFTER_SPACING;
    }
    let end = crate::domain::geometry::quant(len);
    if !rafters.iter().any(|r| (r - end).abs() < 1e-9) {
        rafters.push(end);
    }
    rafters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn studs_run_every_four_tenths_of_a_metre() {
        let studs = stud_positions(4.0, &[]);
        assert_eq!(studs, vec![0.0, 0.4, 0.8, 1.2, 1.6, 2.0, 2.4, 2.8, 3.2, 3.6, 4.0]);
    }

    #[test]
    fn studs_skip_a_door_opening_and_put_trimmers_at_its_sides() {
        let studs = stud_positions(4.0, &[(1.5, 2.3)]);
        assert!(!studs.iter().any(|s| *s > 1.5 && *s < 2.3), "a stud stands in the doorway: {studs:?}");
        assert!(studs.contains(&1.5), "no trimmer at the door's low side: {studs:?}");
        assert!(studs.contains(&2.3), "no trimmer at the door's high side: {studs:?}");
    }

    #[test]
    fn joists_run_every_four_tenths_across_the_span() {
        let joists = joist_positions(3.0);
        assert_eq!(joists, vec![0.0, 0.4, 0.8, 1.2, 1.6, 2.0, 2.4, 2.8, 3.0]);
    }

    #[test]
    fn rafters_run_every_six_tenths_along_the_ridge() {
        let rafters = rafter_positions(6.0);
        assert_eq!(rafters, vec![0.0, 0.6, 1.2, 1.8, 2.4, 3.0, 3.6, 4.2, 4.8, 5.4, 6.0]);
    }

    #[test]
    fn a_wall_frame_has_a_bottom_and_a_doubled_top_plate() {
        assert_eq!(TOP_PLATES, 2, "the top plate is doubled in real framing");
        assert!((PLATE_H - 0.04).abs() < 1e-9);
    }
}
