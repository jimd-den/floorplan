//! The alcove: a re-tiling cut that carves properly-sized leaves
//! out of a footprint rect.
//!
//! A rect that subdivides into only oversized leaves cannot hold
//! a bath (cap 6 sq m) — no strip cut fits, because 6 sq m at
//! MIN_SIDE needs a 2.5 m strip and the leaf's narrow side is
//! under 4.9 m. The alcove redraws the rect instead: it cuts one
//! 2.4 m strip off the west side (orientation A) or the north
//! side (orientation B), splits the strip into a 2.4 x 2.5 m leaf
//! (6 sq m, exactly the bath cap) and a remainder, and subdivides
//! the rest normally. The pinned leaf always has an exterior edge
//! — the west edge in A, the north edge in B — so it still gets a
//! window. A rect can hold several strips, one per pin, so a rect
//! with two over-cap baths gets two alcoves.

use crate::domain::geometry::{Axis, Rect, Segment};
use crate::domain::model::Program;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::bsp::{Split, subdivide};
use crate::use_cases::footprint::MIN_SIDE;

/// The bath leaf: 2.4 m wide, 2.5 m deep, exactly 6 sq m.
pub const BATH_W: f64 = 2.4;
pub const BATH_D: f64 = 2.5;

/// True when `rect` admits `n` alcove strips: each strip is
/// 2.4 m wide (or deep) and needs 4.9 m in the other axis, and
/// the rest after all strips must still be a valid leaf.
pub fn alcove_fits_n(rect: &Rect, n: usize) -> bool {
    if n == 0 {
        return true;
    }
    let strip = BATH_W * n as f64;
    (rect.w >= strip + MIN_SIDE && rect.d >= BATH_D + MIN_SIDE)
        || (rect.d >= strip + MIN_SIDE && rect.w >= BATH_D + MIN_SIDE)
}

/// True when a corner alcove can be cut from `rect`: the strip
/// leaves a valid remainder on both sides (no sliver).
pub fn alcove_fits(rect: &Rect) -> bool {
    alcove_fits_n(rect, 1)
}

/// The result of re-subdividing a rect with an alcove.
pub struct AlcoveSubdivision {
    pub leaves: Vec<Rect>,
    pub splits: Vec<Split>,
    /// (leaf index, program) pins, in leaf order. The alcove
    /// leaves are the first leaf of each strip: indices
    /// 0, 2, 4, ... — one per pin.
    pub pins: Vec<(usize, Program)>,
}

/// Re-subdivides `rect` so it holds bath-sized leaves, one per
/// pin. Each pin pins one alcove leaf — Bath, or Study, which
/// also fits 6 sq m. The alcove leaves always have an exterior
/// edge (their west side in A, north side in B), so a pinned
/// Study still gets a window. When the rect admits no alcove,
/// or admits fewer strips than there are pins, this is a plain
/// subdivision with no pins.
pub fn subdivide_with_alcove(
    rng: &mut Rng,
    rect: Rect,
    target: usize,
    pins: &[Program],
) -> AlcoveSubdivision {
    if pins.is_empty() || !alcove_fits_n(&rect, pins.len()) {
        let sub = subdivide(rng, rect, target);
        return AlcoveSubdivision { leaves: sub.leaves, splits: sub.splits, pins: vec![] };
    }
    let n = pins.len();
    // Orientation A: vertical strips on the west side, each split
    // into the bath leaf (north) and a remainder. Orientation B:
    // the same, transposed. A is preferred when both fit.
    let vertical = rect.w >= BATH_W * n as f64 + MIN_SIDE && rect.d >= BATH_D + MIN_SIDE;
    let mut leaves = vec![];
    let mut splits = vec![];
    let mut out_pins = vec![];
    let mut rest = rect;
    for &pin in pins {
        let (bath_leaf, strip_rest, strip_split, new_rest, rest_split) = if vertical {
            let at = rest.x + BATH_W;
            let (strip, new_rest) = rest.split(Axis::Vertical, at);
            let strip_split = Split {
                axis: Axis::Vertical,
                at,
                line: Segment::new(at, rest.y, at, rest.bottom()),
            };
            let bat = rest.y + BATH_D;
            let (bath_leaf, strip_rest) = strip.split(Axis::Horizontal, bat);
            let rest_split = Split {
                axis: Axis::Horizontal,
                at: bat,
                line: Segment::new(strip.x, bat, strip.right(), bat),
            };
            (bath_leaf, strip_rest, strip_split, new_rest, rest_split)
        } else {
            let at = rest.y + BATH_W;
            let (strip, new_rest) = rest.split(Axis::Horizontal, at);
            let strip_split = Split {
                axis: Axis::Horizontal,
                at,
                line: Segment::new(rest.x, at, rest.right(), at),
            };
            let bat = rest.x + BATH_D;
            let (bath_leaf, strip_rest) = strip.split(Axis::Vertical, bat);
            let rest_split = Split {
                axis: Axis::Vertical,
                at: bat,
                line: Segment::new(bat, strip.y, bat, strip.bottom()),
            };
            (bath_leaf, strip_rest, strip_split, new_rest, rest_split)
        };
        out_pins.push((leaves.len(), pin));
        leaves.push(bath_leaf);
        leaves.push(strip_rest);
        splits.push(strip_split);
        splits.push(rest_split);
        rest = new_rest;
    }
    // The rest keeps its area-proportional share of the leaves:
    // the strips are small in area, so charging them two leaves
    // each from the count would starve the rest into one huge
    // room. The floor is the leaf count that keeps the rest's
    // average leaf at the rect's original average.
    let by_area = ((target as f64) * rest.area() / rect.area()).ceil() as usize;
    let rest_target = target.saturating_sub(2 * n).max(by_area).max(1);
    let rest_sub = subdivide(rng, rest, rest_target);
    leaves.extend(rest_sub.leaves);
    splits.extend(rest_sub.splits);
    AlcoveSubdivision { leaves, splits, pins: out_pins }
}
