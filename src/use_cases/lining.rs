//! Interior wall linings: a painted skin on the room side of every
//! exterior wall.
//!
//! A wall box has one material, so an exterior wall would show its
//! siding colour inside. The lining is a thin box just inside the
//! room, split around openings so a window or door is never covered.
//! Baseboards sit in front of it.

use crate::domain::geometry::Segment;
use crate::domain::mesh::{Box3, Mesh};
use crate::domain::model::{Floor, OpeningKind, Room};
use crate::use_cases::materials::WALL;
use crate::use_cases::walls::{
    DOOR_H, STOREY_H, WALL_T, WIN_SILL, WIN_TOP, openings_on, wall_segments_toned,
};

/// Lining thickness, in metres.
const LINING_T: f32 = 0.01;

/// The lining boxes on a floor, at `base` height.
pub fn lining_boxes(floor: &Floor, base: f32) -> Vec<Box3> {
    let mut out = vec![];
    for (seg, exterior) in wall_segments_toned(&floor.rooms) {
        if !exterior {
            continue;
        }
        let Some(room) = floor.rooms.iter().find(|r| r.rect.edges().contains(&seg)) else {
            continue;
        };
        let spans = openings_on(&floor.openings, &seg);
        for (a, b) in solid_spans(&seg, &spans) {
            out.push(lining_box(&seg, room, a, b, base, base + STOREY_H));
        }
        for &(a, b, kind) in &spans {
            if kind == OpeningKind::Window {
                out.push(lining_box(&seg, room, a, b, base, base + WIN_SILL));
                out.push(lining_box(&seg, room, a, b, base + WIN_TOP, base + STOREY_H));
            } else {
                out.push(lining_box(&seg, room, a, b, base + DOOR_H, base + STOREY_H));
            }
        }
    }
    out
}

/// Adds the lining to a mesh, in the painted-wall material.
pub fn add_wall_linings(mesh: &mut Mesh, floor: &Floor, base: f32) {
    for b in lining_boxes(floor, base) {
        mesh.add_box(b, WALL);
    }
}

/// The solid spans of a segment: between openings and at its ends.
fn solid_spans(seg: &Segment, spans: &[(f64, f64, OpeningKind)]) -> Vec<(f64, f64)> {
    let (lo, hi) = if seg.is_horizontal() { (seg.x1, seg.x2) } else { (seg.y1, seg.y2) };
    let mut out = vec![];
    let mut cursor = lo;
    for &(a, b, _) in spans {
        if a > cursor + 1e-9 {
            out.push((cursor, a));
        }
        cursor = b.max(cursor);
    }
    if hi > cursor + 1e-9 {
        out.push((cursor, hi));
    }
    out
}

/// One lining box on the room side of the wall, between heights `h0`
/// and `h1` and along the segment from `a` to `b`.
fn lining_box(seg: &Segment, room: &Room, a: f64, b: f64, h0: f32, h1: f32) -> Box3 {
    let half = WALL_T / 2.0;
    let r = room.rect;
    if seg.is_horizontal() {
        let (z0, z1) = if (seg.y1 - r.y).abs() < 1e-9 {
            (seg.y1 as f32 + half, seg.y1 as f32 + half + LINING_T)
        } else {
            (r.bottom() as f32 - half - LINING_T, r.bottom() as f32 - half)
        };
        Box3::new([a as f32, h0, z0], [b as f32, h1, z1])
    } else {
        let (x0, x1) = if (seg.x1 - r.x).abs() < 1e-9 {
            (seg.x1 as f32 + half, seg.x1 as f32 + half + LINING_T)
        } else {
            (r.right() as f32 - half - LINING_T, r.right() as f32 - half)
        };
        Box3::new([x0, h0, a as f32], [x1, h1, b as f32])
    }
}
