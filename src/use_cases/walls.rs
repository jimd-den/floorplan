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
/// How far a wall reaches below the floor slab, so the slab edge is
/// never a coloured band around the house.
const FOOTING: f32 = 0.1;

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
    wall_boxes_toned(floor, base, wall_material, wall_material)
}

/// The wall boxes of one floor, siding on the exterior and painted
/// wall on the interior. A segment is exterior when it belongs to
/// exactly one room's edges.
pub fn wall_boxes_toned(floor: &Floor, base: f32, interior: u32, exterior: u32) -> Vec<(Box3, u32)> {
    let mut out = vec![];
    for (seg, is_exterior) in wall_segments_toned(&floor.rooms) {
        let spans = openings_on(&floor.openings, &seg);
        let material = if is_exterior { exterior } else { interior };
        out.extend(boxes_for_segment(&seg, &spans, base, material));
    }
    out
}

/// Every wall segment with whether it is exterior (one room edge).
pub fn wall_segments_toned(rooms: &[Room]) -> Vec<(Segment, bool)> {
    let mut counts: Vec<(Segment, usize)> = vec![];
    for room in rooms {
        for e in room.rect.edges() {
            if let Some(entry) = counts.iter_mut().find(|(s, _)| *s == e) {
                entry.1 += 1;
            } else {
                counts.push((e, 1));
            }
        }
    }
    counts.into_iter().map(|(s, n)| (s, n == 1)).collect()
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
            out.push((wall_box(seg, cursor, a, base - FOOTING, base + STOREY_H), wall_material));
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
        out.push((wall_box(seg, cursor, hi, base - FOOTING, base + STOREY_H), wall_material));
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
