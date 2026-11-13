//! Strip cuts: the geometry of carving a capped room out of
//! an oversized leaf. Split out of `caps` so both files stay
//! under the 200-line ceiling.

use crate::domain::geometry::{Axis, Rect, Segment};
use crate::domain::model::Room;
use crate::use_cases::bsp::Split;
use crate::use_cases::footprint::MIN_SIDE;
use crate::use_cases::placement::longest_exterior;

/// Whether a rect can be strip-cut to its cap: the capped room
/// keeps a 2.4 m strip, the remainder keeps the rest. The same
/// condition `strip_cuts` uses, exposed so the assignment can
/// prefer leaves the cap step can actually fix.
pub(crate) fn is_cuttable(rect: &Rect, cap: f64) -> bool {
    let lo = rect.w.min(rect.d);
    let hi = rect.w.max(rect.d);
    lo * MIN_SIDE <= cap + 1e-9 && hi >= 2.0 * MIN_SIDE - 1e-9
}

/// The two strip cuts of a rect: the capped room at each end of the
/// long side. None when no cut yields a capped room and a remainder
/// both at least `MIN_SIDE` on every side.
pub(crate) fn strip_cuts(rect: &Rect, cap: f64) -> Option<[(Rect, Rect, Split); 2]> {
    if !is_cuttable(rect, cap) {
        return None;
    }
    Some([cut_at(rect, true), cut_at(rect, false)])
}

/// One strip cut: the capped room (a `MIN_SIDE` strip) at the low or
/// the high end of the long side, the remainder keeping the rest.
fn cut_at(rect: &Rect, low_end: bool) -> (Rect, Rect, Split) {
    if rect.w >= rect.d {
        let at = if low_end { rect.x + MIN_SIDE } else { rect.right() - MIN_SIDE };
        let (a, b) = rect.split(Axis::Vertical, at);
        let (small, rest) = if low_end { (a, b) } else { (b, a) };
        let line = Segment::new(at, rect.y, at, rect.bottom());
        (small, rest, Split { axis: Axis::Vertical, at, line })
    } else {
        let at = if low_end { rect.y + MIN_SIDE } else { rect.bottom() - MIN_SIDE };
        let (a, b) = rect.split(Axis::Horizontal, at);
        let (small, rest) = if low_end { (a, b) } else { (b, a) };
        let line = Segment::new(rect.x, at, rect.right(), at);
        (small, rest, Split { axis: Axis::Horizontal, at, line })
    }
}

/// Picks the cut whose remainder keeps the longest exterior edge,
/// and returns that exterior length with it. The capped room is in
/// the rect list, so the cut edge itself is not counted as exterior.
pub(crate) fn pick_cut(rooms: &[Room], i: usize, cuts: &[(Rect, Rect, Split); 2]) -> (Rect, Rect, Split, f64) {
    let mut best = 0;
    let mut best_ext = f64::MIN;
    for (k, (small, remainder, _)) in cuts.iter().enumerate() {
        let mut rects: Vec<Rect> = rooms
            .iter()
            .enumerate()
            .map(|(j, r)| if j == i { *remainder } else { r.rect })
            .collect();
        rects.push(*small);
        let ext = longest_exterior(&rects, i);
        if ext > best_ext {
            best_ext = ext;
            best = k;
        }
    }
    let (small, remainder, split) = cuts[best].clone();
    (small, remainder, split, best_ext)
}
