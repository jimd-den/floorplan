//! Walls as boxes: every room edge, each shared wall once, split
//! around doors and windows — a lintel above every opening and a
//! sill below every window. An opening is a real gap in the wall
//! geometry, never a decal. The mesh reads these boxes; the SVG
//! reads the same segments.

use crate::domain::geometry::{Axis, Segment, EPS};
use crate::domain::mesh::Box3;
use crate::domain::model::{Floor, Opening, OpeningKind, Room};

/// Wall thickness, in metres.
pub const WALL_T: f32 = 0.15;
/// Height of one storey, in metres.
pub const STOREY_H: f32 = 2.7;
/// Height of a door opening, in metres.
pub const DOOR_H: f32 = 2.1;
/// Height of a window sill, in metres.
pub const WIN_SILL: f32 = 0.9;
/// Height of a window top, in metres.
pub const WIN_TOP: f32 = 2.1;

/// Every room edge, each shared wall once.
pub fn wall_segments(rooms: &[Room]) -> Vec<Segment> {
    let mut out: Vec<Segment> = vec![];
    for room in rooms {
        for e in room.rect.edges() {
            if !out.contains(&e) {
                out.push(e);
            }
        }
    }
    out
}

/// The wall boxes of one floor at height `base`: full-height boxes
/// between openings, a lintel above every opening, a sill below
/// every window. Each box carries `wall_material`.
pub fn wall_boxes(floor: &Floor, base: f32, wall_material: u32) -> Vec<(Box3, u32)> {
    let mut out = vec![];
    for seg in wall_segments(&floor.rooms) {
        let spans = openings_on(&floor.openings, &seg);
        out.extend(boxes_for_segment(&seg, &spans, base, wall_material));
    }
    out
}

/// The openings on one wall segment, as (start, end, kind) spans
/// along the segment, sorted by start.
pub fn openings_on(openings: &[Opening], seg: &Segment) -> Vec<(f64, f64, OpeningKind)> {
    let mut spans = vec![];
    for o in openings {
        let on = if seg.is_horizontal() {
            o.axis == Axis::Horizontal
                && (o.y - seg.y1).abs() < EPS
                && o.x >= seg.x1 - EPS
                && o.x <= seg.x2 + EPS
        } else {
            o.axis == Axis::Vertical
                && (o.x - seg.x1).abs() < EPS
                && o.y >= seg.y1 - EPS
                && o.y <= seg.y2 + EPS
        };
        if !on {
            continue;
        }
        let c = if seg.is_horizontal() { o.x } else { o.y };
        let lo = if seg.is_horizontal() { seg.x1 } else { seg.y1 };
        let hi = if seg.is_horizontal() { seg.x2 } else { seg.y2 };
        spans.push(((c - o.width / 2.0).max(lo), (c + o.width / 2.0).min(hi), o.kind));
    }
    spans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    spans
}

/// The boxes for one wall segment: full-height pieces between the
/// opening spans, a lintel above each opening, a sill below each
/// window.
fn boxes_for_segment(
    seg: &Segment,
    spans: &[(f64, f64, OpeningKind)],
    base: f32,
    wall_material: u32,
) -> Vec<(Box3, u32)> {
    let mut out = vec![];
    let (lo, hi) = if seg.is_horizontal() { (seg.x1, seg.x2) } else { (seg.y1, seg.y2) };
    let mut cursor = lo;
    for &(a, b, kind) in spans {
        if a > cursor + 1e-9 {
            out.push((wall_box(seg, cursor, a, base, base + STOREY_H), wall_material));
        }
        let top = match kind {
            OpeningKind::Window => WIN_TOP,
            _ => DOOR_H,
        };
        out.push((wall_box(seg, a, b, base + top, base + STOREY_H), wall_material));
        if kind == OpeningKind::Window {
            out.push((wall_box(seg, a, b, base, base + WIN_SILL), wall_material));
        }
        cursor = b.max(cursor);
    }
    if hi > cursor + 1e-9 {
        out.push((wall_box(seg, cursor, hi, base, base + STOREY_H), wall_material));
    }
    out
}

/// One wall box: the stretch [a, b] along the segment, from height
/// `h0` to `h1`, `WALL_T` thick and centred on the segment.
fn wall_box(seg: &Segment, a: f64, b: f64, h0: f32, h1: f32) -> Box3 {
    let t = WALL_T as f64 / 2.0;
    if seg.is_horizontal() {
        Box3::new([a as f32, h0, (seg.y1 - t) as f32], [b as f32, h1, (seg.y1 + t) as f32])
    } else {
        Box3::new([(seg.x1 - t) as f32, h0, a as f32], [(seg.x1 + t) as f32, h1, b as f32])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect;
    use crate::domain::model::Program;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    fn opening(id: u32, kind: OpeningKind, x: f64, y: f64, axis: Axis, width: f64) -> Opening {
        Opening { id, kind, x, y, axis, width, from: 0, to: None }
    }

    #[test]
    fn a_shared_wall_appears_once() {
        let rooms = vec![
            room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0)),
            room(1, Program::Kitchen, Rect::new(4.0, 0.0, 4.0, 4.0)),
        ];
        let segs = wall_segments(&rooms);
        assert_eq!(segs.len(), 7, "two 4x4 rooms share one wall: {segs:?}");
        assert_eq!(segs.iter().filter(|s| s.is_vertical() && (s.x1 - 4.0).abs() < 1e-9).count(), 1);
    }

    #[test]
    fn a_door_leaves_a_gap_and_a_lintel() {
        let floor = Floor {
            level: 0,
            rooms: vec![
                room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0)),
                room(1, Program::Kitchen, Rect::new(4.0, 0.0, 4.0, 4.0)),
            ],
            openings: vec![opening(0, OpeningKind::Door, 4.0, 2.0, Axis::Vertical, 0.9)],
            furniture: vec![],
            open_doors: vec![],
        };
        let boxes = wall_boxes(&floor, 0.0, 0);
        // The gap: the door centre at mid-door height is in no box.
        assert!(!boxes.iter().any(|(b, _)| b.contains([4.0, 1.0, 2.0])));
        // The lintel: above the door, the wall is solid.
        assert!(boxes.iter().any(|(b, _)| b.contains([4.0, 2.4, 2.0])));
        // The wall beside the door is solid at mid height.
        assert!(boxes.iter().any(|(b, _)| b.contains([4.0, 1.0, 0.5])));
    }

    #[test]
    fn a_window_leaves_a_gap_a_sill_and_a_lintel() {
        let floor = Floor {
            level: 0,
            rooms: vec![room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0))],
            openings: vec![opening(0, OpeningKind::Window, 2.0, 0.0, Axis::Horizontal, 1.2)],
            furniture: vec![],
            open_doors: vec![],
        };
        let boxes = wall_boxes(&floor, 0.0, 0);
        assert!(!boxes.iter().any(|(b, _)| b.contains([2.0, 1.5, 0.0])));
        assert!(boxes.iter().any(|(b, _)| b.contains([2.0, 0.5, 0.0])), "no sill");
        assert!(boxes.iter().any(|(b, _)| b.contains([2.0, 2.4, 0.0])), "no lintel");
    }
}
