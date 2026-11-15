//! Window placement on one exterior stretch: size and centre the
//! window inside the widest part of the stretch that is clear of the
//! openings already placed on that wall. A window never shares wall
//! length with a door (spec §4).

use crate::domain::geometry::{Axis, Segment};
use crate::domain::model::{Opening, OpeningKind, Room};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::quant;

/// Preferred wall margin at each end of a window, in metres.
const MARGIN: f64 = 0.3;
/// Narrowest stretch that can host a window, in metres.
const MIN_SPAN: f64 = 0.8;
/// Minimum window width, in metres.
const MIN_W: f64 = 0.6;

/// One window on `stretch`, inside the widest part of it that clears
/// every opening in `doors`; `None` when no part is wide enough.
pub(crate) fn place_on_stretch(
    rng: &mut Rng,
    room: &Room,
    stretch: &Segment,
    width: f64,
    doors: &[Opening],
    next_id: &mut u32,
) -> Option<Opening> {
    let (lo, hi) = bounds(stretch);
    let (a, b) = widest_free(lo, hi, &door_spans(stretch, doors))?;
    let avail = b - a;
    if avail < MIN_SPAN {
        return None;
    }
    let width = width.min(avail - 0.2).max(MIN_W);
    let margin = ((avail - width) / 2.0).min(MARGIN);
    let slack = ((avail - width) / 2.0 - margin).max(0.0);
    let centre = a + avail / 2.0
        + if slack > 0.0 { quant(rng.range_f64(-slack, slack)) } else { 0.0 };
    Some(window_at(room, stretch, centre, width, *next_id))
}

/// The stretch's span along its wall.
fn bounds(stretch: &Segment) -> (f64, f64) {
    if stretch.is_horizontal() {
        (stretch.x1.min(stretch.x2), stretch.x1.max(stretch.x2))
    } else {
        (stretch.y1.min(stretch.y2), stretch.y1.max(stretch.y2))
    }
}

/// The widest gap in `[lo, hi]` after removing the blocking spans.
fn widest_free(lo: f64, hi: f64, blocks: &[(f64, f64)]) -> Option<(f64, f64)> {
    let mut best: Option<(f64, f64)> = None;
    let mut cursor = lo;
    for &(a, b) in blocks {
        if a > cursor + 1e-9 {
            best = widen(best, cursor, a);
        }
        cursor = cursor.max(b);
    }
    if hi > cursor + 1e-9 {
        best = widen(best, cursor, hi);
    }
    best
}

/// Keeps the wider of the current best gap and `[a, b]`.
fn widen(best: Option<(f64, f64)>, a: f64, b: f64) -> Option<(f64, f64)> {
    if best.is_none_or(|(p, q)| b - a > q - p) {
        Some((a, b))
    } else {
        best
    }
}

/// The spans of the doors that lie on this stretch's wall line.
fn door_spans(stretch: &Segment, doors: &[Opening]) -> Vec<(f64, f64)> {
    let (horizontal, line) = if stretch.is_horizontal() {
        (true, stretch.y1)
    } else {
        (false, stretch.x1)
    };
    let mut out = vec![];
    for d in doors {
        let on = if horizontal {
            d.axis == Axis::Horizontal && (d.y - line).abs() < 1e-9
        } else {
            d.axis == Axis::Vertical && (d.x - line).abs() < 1e-9
        };
        if !on {
            continue;
        }
        let (a, b) = if horizontal {
            (d.x - d.width / 2.0, d.x + d.width / 2.0)
        } else {
            (d.y - d.width / 2.0, d.y + d.width / 2.0)
        };
        let (lo, hi) = bounds(stretch);
        if a < hi && b > lo {
            out.push((a.max(lo), b.min(hi)));
        }
    }
    out.sort_by(|p, q| p.0.partial_cmp(&q.0).unwrap());
    out
}

/// The window opening centred at `centre` along `stretch`.
fn window_at(room: &Room, stretch: &Segment, centre: f64, width: f64, id: u32) -> Opening {
    let (x, y, axis) = if stretch.is_horizontal() {
        (centre, stretch.y1, Axis::Horizontal)
    } else {
        (stretch.x1, centre, Axis::Vertical)
    };
    Opening { id, kind: OpeningKind::Window, x, y, axis, width, from: room.id, to: None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect;
    use crate::domain::model::{OpeningKind, Program};

    fn room() -> Room {
        Room { id: 0, program: Program::Living, rect: Rect::new(0.0, 0.0, 6.0, 5.0) }
    }

    fn door(x: f64, y: f64, axis: Axis, width: f64) -> Opening {
        Opening { id: 1, kind: OpeningKind::EntryDoor, x, y, axis, width, from: 0, to: None }
    }

    #[test]
    fn a_clear_stretch_takes_the_requested_width() {
        let r = room();
        let s = Segment::new(0.0, 0.0, 6.0, 0.0);
        let mut rng = Rng::new(1);
        let mut id = 0;
        let w = place_on_stretch(&mut rng, &r, &s, 1.8, &[], &mut id).expect("no window");
        assert_eq!(w.width, 1.8);
        assert!(w.x >= 1.0 && w.x <= 5.0, "window {} not inset", w.x);
    }

    #[test]
    fn a_door_on_the_wall_pushes_the_window_clear_of_it() {
        let r = room();
        let s = Segment::new(0.0, 0.0, 6.0, 0.0);
        let d = door(3.0, 0.0, Axis::Horizontal, 1.0);
        let mut rng = Rng::new(2);
        let mut id = 0;
        let w = place_on_stretch(&mut rng, &r, &s, 1.8, std::slice::from_ref(&d), &mut id).expect("no window");
        assert!(!w.overlaps(&d), "window {:?} overlaps the door", w);
    }

    #[test]
    fn a_stretch_narrower_than_the_minimum_is_refused() {
        let r = room();
        let s = Segment::new(0.0, 0.0, 0.7, 0.0);
        let mut rng = Rng::new(3);
        let mut id = 0;
        assert!(place_on_stretch(&mut rng, &r, &s, 1.8, &[], &mut id).is_none());
    }
}
