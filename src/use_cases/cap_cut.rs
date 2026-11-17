//! Strip cuts: the geometry of carving a capped room out of
//! an oversized leaf. Split out of `caps` so both files stay
//! under the 200-line ceiling.

use crate::domain::geometry::{Axis, Rect, Segment};
use crate::domain::model::Room;
use crate::domain::room_size::ASPECT_HARD_CAP;
use crate::use_cases::bsp::Split;
use crate::use_cases::footprint::MIN_SIDE;
use crate::use_cases::placement::longest_exterior;

/// The strip depth for a cut across the long side: at least
/// MIN_SIDE, and at least half the short side, so the kept
/// piece keeps aspect <= 2.0 (ADR 0015). A 4.9 m-wide leaf
/// cut at 2.4 m would keep a 2.04:1 piece.
fn strip_depth(rect: &Rect) -> f64 {
    let lo = rect.w.min(rect.d);
    MIN_SIDE.max(lo / ASPECT_HARD_CAP)
}

/// Whether a rect can be strip-cut to its cap: the capped room
/// keeps a strip of `strip_depth`, the remainder keeps the rest.
/// The same condition `strip_cuts` uses, exposed so the
/// assignment can prefer leaves the cap step can actually fix.
pub(crate) fn is_cuttable(rect: &Rect, cap: f64) -> bool {
    let lo = rect.w.min(rect.d);
    let hi = rect.w.max(rect.d);
    let d = strip_depth(rect);
    lo * d <= cap + 1e-9 && hi - d >= MIN_SIDE - 1e-9
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

/// One strip cut: the capped room (a `strip_depth` strip) at the
/// low or the high end of the long side, the remainder keeping
/// the rest.
fn cut_at(rect: &Rect, low_end: bool) -> (Rect, Rect, Split) {
    let depth = strip_depth(rect);
    if rect.w >= rect.d {
        let at = if low_end { rect.x + depth } else { rect.right() - depth };
        let (a, b) = rect.split(Axis::Vertical, at);
        let (small, rest) = if low_end { (a, b) } else { (b, a) };
        let line = Segment::new(at, rect.y, at, rect.bottom());
        (small, rest, Split { axis: Axis::Vertical, at, line })
    } else {
        let at = if low_end { rect.y + depth } else { rect.bottom() - depth };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::room_size::aspect;

    #[test]
    fn a_study_leaf_no_strip_can_fix_is_not_cuttable() {
        // Seed 109 Colonial: a 4.9 x 5.9 m study (28.91 sq m).
        // A 2.4 m strip keeps 4.9 x 2.4 (11.76 sq m, under the
        // 12 cap) but its aspect is 2.04 — over the cap. The
        // aspect-legal strip (2.45 m) keeps 12.005 sq m, over
        // the cap. No strip cut can fix this leaf legally.
        assert!(strip_cuts(&Rect::new(0.0, 0.0, 4.9, 5.9), 12.0).is_none());
    }

    #[test]
    fn a_cuttable_wide_leaf_keeps_its_strip_within_aspect() {
        // 4.85 x 8.0 m study: the 2.4 m strip would be 2.02:1,
        // the aspect-legal 2.425 m strip is exactly 2.0:1.
        let cuts = strip_cuts(&Rect::new(0.0, 0.0, 4.85, 8.0), 12.0).unwrap();
        for (small, _, _) in cuts {
            let a = aspect(&small);
            assert!(a <= 2.0 + 1e-9, "kept strip aspect is {a}");
        }
    }
}
