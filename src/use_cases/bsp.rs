//! BSP subdivision: recursively splits a rectangle into rooms.
//! Splits alternate by aspect ratio, positions are quantized to 0.1 m,
//! and no leaf ever drops below `MIN_SIDE` — no sliver rooms.
//! ADR 0014: a cut is only valid when both children keep aspect
//! <= 2.0. A leaf that admits no such single cut is re-split
//! into aspect-valid pieces, never emitted whole.
//! ADR 0015: no cut lands a child in the dead zone (6.0, 6.5)
//! sq m — over the Bath cap, under the habitable floor.

use crate::domain::geometry::{Axis, Rect, Segment};
use crate::domain::room_size::{ASPECT_HARD_CAP, HABITABLE_MIN_AREA, aspect};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::footprint::MIN_SIDE;
use crate::use_cases::long_rect::split_long_rect;
use crate::use_cases::quant;
use crate::use_cases::split_pieces::record_piece_splits;

/// One cut: the axis, the coordinate, and the full cut segment.
#[derive(Debug, Clone, PartialEq)]
pub struct Split {
    pub axis: Axis,
    pub at: f64,
    pub line: Segment,
}

/// The result of subdividing a rectangle: the leaves and every cut.
#[derive(Debug, Clone, PartialEq)]
pub struct Subdivision {
    pub leaves: Vec<Rect>,
    pub splits: Vec<Split>,
}

/// Subdivides `rect` into up to `target` leaves. Stops early when no
/// leaf can be split without breaking the minimum size. A leaf the
/// target leaves whole but over the aspect cap (the rest after
/// alcove strips, 2.9 x 13.9 m) is re-split into aspect-valid
/// pieces — the cap is a hard rule, so it wins over the count.
pub fn subdivide(rng: &mut Rng, rect: Rect, target: usize) -> Subdivision {
    let mut leaves = vec![rect];
    let mut splits = vec![];
    while leaves.len() < target {
        let Some(idx) = largest_splittable(&leaves) else { break };
        let leaf = leaves.remove(idx);
        let axis = pick_axis(rng, &leaf);
        let (a, b, split) = cut(rng, &leaf, axis);
        leaves.push(a);
        leaves.push(b);
        splits.push(split);
    }
    // Re-split any leaf still over the cap: a single cut cannot
    // fix a long thin leaf, but split_long_rect divides it into
    // as many pieces as the cap needs. Each piece boundary is
    // recorded so the skeleton walls it.
    let mut i = 0;
    while i < leaves.len() {
        if aspect(&leaves[i]) <= ASPECT_HARD_CAP {
            i += 1;
            continue;
        }
        let leaf = leaves.remove(i);
        let vertical = leaf.w >= leaf.d;
        let pieces = split_long_rect(leaf);
        record_piece_splits(&mut splits, &pieces, vertical);
        for (j, piece) in pieces.iter().enumerate() {
            leaves.insert(i + j, *piece);
        }
        i += pieces.len();
    }
    Subdivision { leaves, splits }
}

/// Index of the largest leaf that can still split, if any.
fn largest_splittable(leaves: &[Rect]) -> Option<usize> {
    leaves
        .iter()
        .enumerate()
        .filter(|(_, leaf)| can_split(leaf))
        .max_by(|(_, a), (_, b)| a.area().partial_cmp(&b.area()).unwrap())
        .map(|(i, _)| i)
}

/// A leaf can split when some axis admits a cut whose children
/// both keep aspect <= 2.0 (ADR 0014).
fn can_split(leaf: &Rect) -> bool {
    cut_range(leaf, Axis::Vertical).is_some() || cut_range(leaf, Axis::Horizontal).is_some()
}

/// The Bath cap in sq m (`cap_for(Program::Bath)`), spelled
/// out here because `caps` already imports `bsp`.
const BATH_CAP: f64 = 6.0;

/// The valid cut positions on `axis`: the `at` interval in which
/// both children keep aspect <= 2.0 and sides >= `MIN_SIDE`,
/// and neither child lands in the dead zone (6.0, 6.5) sq m
/// (ADR 0015). None when no cut on this axis can do that. Each
/// child spans `cross` on the uncut side and a share of `len` on
/// the cut side; aspect <= 2.0 means the share is in
/// [cross/2, 2*cross]. The interval is quantized inward to 0.1 m
/// so the picked position stays valid. The dead-zone holes sit
/// near the ends (a small child is a cut near an end); the
/// longest surviving interval is returned, so the cut stays
/// random within it.
fn cut_range(leaf: &Rect, axis: Axis) -> Option<(f64, f64)> {
    let (len, cross, start) = match axis {
        Axis::Vertical => (leaf.w, leaf.d, leaf.x),
        Axis::Horizontal => (leaf.d, leaf.w, leaf.y),
    };
    let share_lo = MIN_SIDE.max(cross / ASPECT_HARD_CAP);
    let share_hi = ASPECT_HARD_CAP * cross;
    let lo = (start + share_lo).max(start + len - share_hi);
    let hi = (start + share_hi).min(start + len - share_lo);
    if lo > hi {
        return None;
    }
    let lo_q = (lo * 10.0).ceil() / 10.0;
    let hi_q = (hi * 10.0).floor() / 10.0;
    if lo_q > hi_q {
        return None;
    }
    let holes = [
        (start + BATH_CAP / cross, start + HABITABLE_MIN_AREA / cross),
        (start + len - HABITABLE_MIN_AREA / cross, start + len - BATH_CAP / cross),
    ];
    let mut pieces = vec![(lo_q, hi_q)];
    for (h0, h1) in holes {
        let mut next = vec![];
        for (a, b) in pieces {
            if h1 <= a || h0 >= b {
                next.push((a, b));
            } else {
                if a < h0 {
                    next.push((a, h0));
                }
                if h1 < b {
                    next.push((h1, b));
                }
            }
        }
        pieces = next;
    }
    pieces
        .into_iter()
        .map(|(a, b)| ((a * 10.0).ceil() / 10.0, (b * 10.0).floor() / 10.0))
        .filter(|(a, b)| a <= b)
        .max_by(|x, y| (x.1 - x.0).partial_cmp(&(y.1 - y.0)).unwrap())
}

/// Splits along the long side when the leaf is clearly oblong, else
/// random — but only among axes the leaf can actually split on.
fn pick_axis(rng: &mut Rng, leaf: &Rect) -> Axis {
    let vertical = cut_range(leaf, Axis::Vertical).is_some();
    let horizontal = cut_range(leaf, Axis::Horizontal).is_some();
    match (vertical, horizontal) {
        (true, true) => {
            let ratio = (leaf.w / leaf.d).max(leaf.d / leaf.w);
            if ratio > 1.4 {
                if leaf.w > leaf.d { Axis::Vertical } else { Axis::Horizontal }
            } else if rng.chance(0.5) {
                Axis::Vertical
            } else {
                Axis::Horizontal
            }
        }
        (true, false) => Axis::Vertical,
        (false, true) => Axis::Horizontal,
        (false, false) => Axis::Vertical,
    }
}

/// Cuts one leaf in two at a quantized position inside the valid
/// range, recording the cut. `pick_axis` only returns an axis
/// with a valid range, so the expect cannot fire. The position
/// is chosen among those that leave at least one child able to
/// split again (ADR 0015: a cut leaving two dead-end children
/// wastes the split — seed 28's 3.9 x 7.5 m rest cut at 3.75
/// leaves two 3.9 x 3.75 leaves, neither splittable); when no
/// such position exists, the position is uniform-random over
/// the range.
fn cut(rng: &mut Rng, leaf: &Rect, axis: Axis) -> (Rect, Rect, Split) {
    let (lo, hi) = cut_range(leaf, axis).expect("cut axis has no valid range");
    let steps = ((hi - lo) * 10.0).round() as i64;
    let open: Vec<i64> = (0..=steps)
        .filter(|&s| {
            let (a, b) = leaf.split(axis, quant(lo + s as f64 * 0.1));
            can_split(&a) || can_split(&b)
        })
        .collect();
    let at = if open.is_empty() {
        quant(lo + rng.range_i64(0, steps + 1) as f64 * 0.1)
    } else {
        let s = open[rng.range_i64(0, open.len() as i64) as usize];
        quant(lo + s as f64 * 0.1)
    };
    let (a, b) = leaf.split(axis, at);
    let line = match axis {
        Axis::Vertical => Segment::new(at, leaf.y, at, leaf.bottom()),
        Axis::Horizontal => Segment::new(leaf.x, at, leaf.right(), at),
    };
    (a, b, Split { axis, at, line })
}
