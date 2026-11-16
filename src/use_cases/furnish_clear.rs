//! Clearance checks for moving furniture: a candidate rect must
//! sit inside its room, overlap no placed piece, and block no door
//! swing and no window. Used by `interact` when a piece is moved;
//! generation places no furniture, so nothing here is called
//! while a house is built.

use crate::domain::furniture::{Furniture, FurnitureKind};
use crate::domain::geometry::{Axis, Rect};
use crate::domain::model::{Opening, Room};

/// A candidate rect is valid when it sits inside the room, overlaps
/// no placed piece, blocks no door swing and no window. Only tall
/// pieces can block a window; low pieces sit below the sill.
pub fn fits(
    room: &Room,
    kind: FurnitureKind,
    rect: &Rect,
    doors: &[&Opening],
    windows: &[&Opening],
    placed: &[Furniture],
) -> bool {
    if rect.x < room.rect.x - 1e-9
        || rect.y < room.rect.y - 1e-9
        || rect.right() > room.rect.right() + 1e-9
        || rect.bottom() > room.rect.bottom() + 1e-9
    {
        return false;
    }
    if placed.iter().any(|p| p.rect.overlaps(rect)) {
        return false;
    }
    if doors.iter().any(|o| blocks_door(room, rect, o)) {
        return false;
    }
    if kind.is_tall() && windows.iter().any(|o| blocks_window(rect, o)) {
        return false;
    }
    true
}

/// A piece blocks a door when it stands inside the door's swing:
/// a rectangle the door's width along the wall (plus a 0.1 m
/// margin) and the door's width into the room — on the room's
/// side of the wall only. A door swings into one room, not both.
pub fn blocks_door(room: &Room, rect: &Rect, o: &Opening) -> bool {
    rect.overlaps(&door_zone(room, o))
}

/// The door's swing zone on the room's side of the wall.
pub fn door_zone(room: &Room, o: &Opening) -> Rect {
    let w = o.width;
    let m = 0.1;
    match o.axis {
        Axis::Horizontal => {
            let x = o.x - w / 2.0 - m;
            let span = w + 2.0 * m;
            if (o.y - room.rect.y).abs() < 1e-9 {
                Rect::new(x, o.y, span, w)
            } else {
                Rect::new(x, o.y - w, span, w)
            }
        }
        Axis::Vertical => {
            let y = o.y - w / 2.0 - m;
            let span = w + 2.0 * m;
            if (o.x - room.rect.x).abs() < 1e-9 {
                Rect::new(o.x, y, w, span)
            } else {
                Rect::new(o.x - w, y, w, span)
            }
        }
    }
}

/// A piece blocks a window when it stands against the window's
/// wall and its span covers the window's span.
pub fn blocks_window(rect: &Rect, o: &Opening) -> bool {
    match o.axis {
        Axis::Horizontal => {
            let on_wall = (rect.y - o.y).abs() < 1e-9 || (rect.bottom() - o.y).abs() < 1e-9;
            on_wall && rect.x < o.x + o.width / 2.0 && rect.right() > o.x - o.width / 2.0
        }
        Axis::Vertical => {
            let on_wall = (rect.x - o.x).abs() < 1e-9 || (rect.right() - o.x).abs() < 1e-9;
            on_wall && rect.y < o.y + o.width / 2.0 && rect.bottom() > o.y - o.width / 2.0
        }
    }
}
