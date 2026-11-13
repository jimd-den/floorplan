//! Trim: window frames and glass, door frames, and baseboards.
//!
//! The wall boxes leave a real gap at every opening; this module
//! frames the gap and glazes the window, so a window reads as a
//! window and a door as a door. Baseboards run along the inner face
//! of every room edge, where the wall meets the floor.

use crate::domain::geometry::Axis;
use crate::domain::mesh::{Box3, Mesh};
use crate::domain::model::{Floor, Opening, OpeningKind};
use crate::use_cases::materials::{BASEBOARD, DOOR_FRAME, GLASS, WINDOW_FRAME};
use crate::use_cases::walls::{DOOR_H, WALL_T, WIN_SILL, WIN_TOP};

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
            Box3::new([lo, sill, y - t / 2.0], [lo + FRAME, top, y + t / 2.0]),
            Box3::new([hi - FRAME, sill, y - t / 2.0], [hi, top, y + t / 2.0]),
            Box3::new([lo, sill - FRAME, y - t / 2.0], [hi, sill, y + t / 2.0]),
            Box3::new([lo, top, y - t / 2.0], [hi, top + FRAME, y + t / 2.0]),
        ],
        Axis::Vertical => [
            Box3::new([x - t / 2.0, sill, lo], [x + t / 2.0, top, lo + FRAME]),
            Box3::new([x - t / 2.0, sill, hi - FRAME], [x + t / 2.0, top, hi]),
            Box3::new([x - t / 2.0, sill - FRAME, lo], [x + t / 2.0, sill, hi]),
            Box3::new([x - t / 2.0, top, lo], [x + t / 2.0, top + FRAME, hi]),
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
            Box3::new([lo, base, y - t / 2.0], [lo + FRAME, top, y + t / 2.0]),
            Box3::new([hi - FRAME, base, y - t / 2.0], [hi, top, y + t / 2.0]),
            Box3::new([lo, top, y - t / 2.0], [hi, top + FRAME, y + t / 2.0]),
        ],
        Axis::Vertical => [
            Box3::new([x - t / 2.0, base, lo], [x + t / 2.0, top, lo + FRAME]),
            Box3::new([x - t / 2.0, base, hi - FRAME], [x + t / 2.0, top, hi]),
            Box3::new([x - t / 2.0, top, lo], [x + t / 2.0, top + FRAME, hi]),
        ],
    };
    for b in members {
        mesh.add_box(b, DOOR_FRAME);
    }
}

/// Four baseboards per room, on the inner face of each wall.
fn add_baseboards(mesh: &mut Mesh, floor: &Floor, base: f32) {
    for room in &floor.rooms {
        let r = room.rect;
        let (x0, z0, x1, z1) = (r.x as f32, r.y as f32, r.right() as f32, r.bottom() as f32);
        let (a, b) = (base, base + BASE_H);
        let members = [
            Box3::new([x0, a, z0 + BASE_IN], [x1, b, z0 + BASE_IN + BASE_T]),
            Box3::new([x0, a, z1 - BASE_IN - BASE_T], [x1, b, z1 - BASE_IN]),
            Box3::new([x0 + BASE_IN, a, z0], [x0 + BASE_IN + BASE_T, b, z1]),
            Box3::new([x1 - BASE_IN - BASE_T, a, z0], [x1 - BASE_IN, b, z1]),
        ];
        for b in members {
            mesh.add_box(b, BASEBOARD);
        }
    }
}
