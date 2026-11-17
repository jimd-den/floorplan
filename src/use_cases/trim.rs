//! Trim: window frames and glass, door frames, and baseboards.
//!
//! The wall boxes leave a real gap at every opening; this module
//! frames the gap and glazes the window, so a window reads as a
//! window and a door as a door. Baseboards run along the inner face
//! of every room edge, where the wall meets the floor.

use crate::domain::geometry::Axis;
use crate::domain::mesh::{Box3, Mesh};
use crate::domain::model::{Floor, Opening, OpeningKind, Program};
use crate::use_cases::materials::{BASEBOARD, DOOR_FRAME, GLASS, WINDOW_FRAME};
use crate::use_cases::walls::{DOOR_H, WALL_T, WIN_SILL, WIN_TOP, openings_on};

/// The width of a frame member, in metres.
const FRAME: f32 = 0.06;
/// The distance from a wall's inner face to the room side, where a
/// baseboard sits.
const BASE_IN: f32 = 0.08;
/// Baseboard height, in metres.
const BASE_H: f32 = 0.1;
/// Baseboard thickness, in metres.
const BASE_T: f32 = 0.02;

/// Adds every window frame, glass pane, door frame, and baseboard on
/// a floor, at `base` height.
pub fn add_trim(mesh: &mut Mesh, floor: &Floor, base: f32) {
    for o in &floor.openings {
        match o.kind {
            OpeningKind::Window => add_window(mesh, o, base),
            OpeningKind::Door | OpeningKind::EntryDoor => add_door_frame(mesh, o, base),
        }
    }
    add_baseboards(mesh, floor, base);
}

/// A window: four frame members around the opening and a glass pane.
fn add_window(mesh: &mut Mesh, o: &Opening, base: f32) {
    let (x, y, w) = (o.x as f32, o.y as f32, o.width as f32);
    let t = WALL_T;
    let (sill, top) = (base + WIN_SILL, base + WIN_TOP);
    let (lo, hi) = match o.axis {
        Axis::Horizontal => (x - w / 2.0, x + w / 2.0),
        Axis::Vertical => (y - w / 2.0, y + w / 2.0),
    };
    let members: [Box3; 4] = match o.axis {
        Axis::Horizontal => [
            Box3::new([lo, sill, y - t / 2.0 - 0.02], [lo + FRAME, top, y + t / 2.0 + 0.02]),
            Box3::new([hi - FRAME, sill, y - t / 2.0 - 0.02], [hi, top, y + t / 2.0 + 0.02]),
            Box3::new([lo, sill - FRAME, y - t / 2.0 - 0.03], [hi, sill, y + t / 2.0 + 0.03]),
            Box3::new([lo, top, y - t / 2.0 - 0.03], [hi, top + FRAME, y + t / 2.0 + 0.03]),
        ],
        Axis::Vertical => [
            Box3::new([x - t / 2.0 - 0.02, sill, lo], [x + t / 2.0 + 0.02, top, lo + FRAME]),
            Box3::new([x - t / 2.0 - 0.02, sill, hi - FRAME], [x + t / 2.0 + 0.02, top, hi]),
            Box3::new([x - t / 2.0 - 0.03, sill - FRAME, lo], [x + t / 2.0 + 0.03, sill, hi]),
            Box3::new([x - t / 2.0 - 0.03, top, lo], [x + t / 2.0 + 0.03, top + FRAME, hi]),
        ],
    };
    for b in members {
        mesh.add_box(b, WINDOW_FRAME);
    }
    let glass = match o.axis {
        Axis::Horizontal => Box3::new([lo + FRAME, sill, y - 0.02], [hi - FRAME, top, y + 0.02]),
        Axis::Vertical => Box3::new([x - 0.02, sill, lo + FRAME], [x + 0.02, top, hi - FRAME]),
    };
    mesh.add_box(glass, GLASS);
}

/// A door: two jambs and a head, leaving the threshold open.
fn add_door_frame(mesh: &mut Mesh, o: &Opening, base: f32) {
    let (x, y, w) = (o.x as f32, o.y as f32, o.width as f32);
    let t = WALL_T;
    let top = base + DOOR_H;
    let (lo, hi) = match o.axis {
        Axis::Horizontal => (x - w / 2.0, x + w / 2.0),
        Axis::Vertical => (y - w / 2.0, y + w / 2.0),
    };
    let members: [Box3; 3] = match o.axis {
        Axis::Horizontal => [
            Box3::new([lo, base, y - t / 2.0 - 0.02], [lo + FRAME, top, y + t / 2.0 + 0.02]),
            Box3::new([hi - FRAME, base, y - t / 2.0 - 0.02], [hi, top, y + t / 2.0 + 0.02]),
            Box3::new([lo - 0.02, top, y - t / 2.0 - 0.02], [hi + 0.02, top + FRAME, y + t / 2.0 + 0.02]),
        ],
        Axis::Vertical => [
            Box3::new([x - t / 2.0 - 0.02, base, lo], [x + t / 2.0 + 0.02, top, lo + FRAME]),
            Box3::new([x - t / 2.0 - 0.02, base, hi - FRAME], [x + t / 2.0 + 0.02, top, hi]),
            Box3::new([x - t / 2.0 - 0.02, top, lo - 0.02], [x + t / 2.0 + 0.02, top + FRAME, hi + 0.02]),
        ],
    };
    for b in members {
        mesh.add_box(b, DOOR_FRAME);
    }
}

/// Four baseboards per room, plus chair rail and crown moulding in
/// the principal rooms (living, dining, entry), on the inner face of
/// each wall.
fn add_baseboards(mesh: &mut Mesh, floor: &Floor, base: f32) {
    for (b, m) in baseboard_boxes(floor, base) {
        mesh.add_box(b, m);
    }
}

/// The baseboard, chair rail, and crown boxes for a floor: one box
/// per band per room edge, on the inner face of each wall. A band
/// that would cross a door — baseboard and chair rail, both below
/// the 2.1 m door head — stops at the door and resumes past it;
/// the crown, above the head, runs continuously over the door.
pub fn baseboard_boxes(floor: &Floor, base: f32) -> Vec<(Box3, u32)> {
    let mut out = vec![];
    for room in &floor.rooms {
        let r = room.rect;
        let (x0, z0, x1, z1) = (r.x as f32, r.y as f32, r.right() as f32, r.bottom() as f32);
        let bands: &[(f32, f32)] = if matches!(room.program, Program::Living | Program::Dining | Program::Entry) {
            &[(base, base + BASE_H), (base + 0.90, base + 0.94), (base + 2.62, base + 2.68)]
        } else {
            &[(base, base + BASE_H)]
        };
        for (i, edge) in r.edges().iter().enumerate() {
            let doors: Vec<(f64, f64)> = openings_on(&floor.openings, edge)
                .into_iter()
                .filter(|(_, _, kind)| matches!(kind, OpeningKind::Door | OpeningKind::EntryDoor))
                .map(|(a, b, _)| (a, b))
                .collect();
            let (lo, hi) = if edge.is_horizontal() { (edge.x1, edge.x2) } else { (edge.y1, edge.y2) };
            for &(a, b) in bands {
                let pieces = if a < base + DOOR_H && b > base {
                    band_pieces(lo, hi, &doors)
                } else {
                    vec![(lo, hi)]
                };
                for (p, q) in pieces {
                    let (p, q) = (p as f32, q as f32);
                    let m = match i {
                        0 => Box3::new([p, a, z0 + BASE_IN], [q, b, z0 + BASE_IN + BASE_T]),
                        2 => Box3::new([p, a, z1 - BASE_IN - BASE_T], [q, b, z1 - BASE_IN]),
                        3 => Box3::new([x0 + BASE_IN, a, p], [x0 + BASE_IN + BASE_T, b, q]),
                        _ => Box3::new([x1 - BASE_IN - BASE_T, a, p], [x1 - BASE_IN, b, q]),
                    };
                    out.push((m, BASEBOARD));
                }
            }
        }
    }
    out
}

/// Splits the interval [lo, hi] at the door spans, returning the
/// runs of wall between doors. Spans arrive sorted by start.
fn band_pieces(lo: f64, hi: f64, doors: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut out = vec![];
    let mut cursor = lo;
    for &(a, b) in doors {
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
