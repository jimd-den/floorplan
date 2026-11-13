//! Clearance checks for furniture placement: a candidate rect
//! must sit inside its room, overlap no placed piece, and block
//! no door swing and no window. Split out of `furnish` so both
//! files stay under the 200-line ceiling.

use crate::domain::furniture::{Furniture, FurnitureKind};
use crate::domain::geometry::{Axis, Rect};
use crate::domain::model::{Opening, Room};

/// A candidate rect is valid when it sits inside the room, overlaps
/// no placed piece, blocks no door swing and no window. Only tall
/// pieces can block a window; low pieces sit below the sill.
pub(crate) fn fits(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Axis;
    use crate::domain::model::{Floor, OpeningKind, Program};
    use crate::frameworks_drivers::rng::Rng;
    use crate::use_cases::furnish::furnish;

    fn room_with(doors: Vec<Opening>, windows: Vec<Opening>) -> Floor {
        Floor {
            level: 0,
            rooms: vec![Room { id: 0, program: Program::Bedroom, rect: Rect::new(0.0, 0.0, 6.0, 5.0) }],
            openings: doors.into_iter().chain(windows).collect(),
            furniture: vec![],
            open_doors: vec![],
        }
    }

    fn door(id: u32, x: f64, y: f64, axis: Axis) -> Opening {
        Opening { id, kind: OpeningKind::Door, x, y, axis, width: 0.9, from: 0, to: Some(1) }
    }

    fn window(id: u32, x: f64, y: f64, axis: Axis) -> Opening {
        Opening { id, kind: OpeningKind::Window, x, y, axis, width: 1.0, from: 0, to: None }
    }

    #[test]
    fn no_piece_blocks_the_door() {
        let floor = room_with(vec![door(0, 3.0, 0.0, Axis::Horizontal)], vec![]);
        let mut rng = Rng::new(2);
        let pieces = furnish(&mut rng, &floor);
        for p in &pieces {
            assert!(!blocks_door(&floor.rooms[0], &p.rect, &floor.openings[0]), "{p:?} blocks the door");
        }
    }

    #[test]
    fn no_piece_blocks_the_window() {
        let floor = room_with(vec![], vec![window(0, 3.0, 0.0, Axis::Horizontal)]);
        let mut rng = Rng::new(3);
        let pieces = furnish(&mut rng, &floor);
        for p in &pieces {
            assert!(
                !p.kind.is_tall() || !blocks_window(&p.rect, &floor.openings[0]),
                "{p:?} blocks the window"
            );
        }
    }

    #[test]
    fn the_door_zone_is_one_sided() {
        // A door on the room's north wall swings into the room
        // (south), not out through the wall to the north.
        let room = Room { id: 0, program: Program::Bedroom, rect: Rect::new(0.0, 0.0, 6.0, 5.0) };
        let d = door(0, 3.0, 0.0, Axis::Horizontal);
        let zone = door_zone(&room, &d);
        assert!(zone.y >= 0.0, "zone leaks north of the wall: {zone:?}");
        assert!((zone.bottom() - 0.9).abs() < 1e-9, "zone should reach 0.9 m in: {zone:?}");
    }
}
