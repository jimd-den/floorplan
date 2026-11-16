//! Wall blocking: which plan points a wall blocks, and where its
//! door gaps are. A shared wall between two rooms blocks movement
//! except inside a door-gap span; windows are not gaps, their
//! sills block. The walk check reads this, never the raw walls.

use crate::domain::geometry::{Axis, Segment, shared_edge};
use crate::domain::model::{Floor, OpeningKind};

/// A wall: a shared edge between two rooms, and the door gaps cut
/// into it (each a span along the wall).
pub(crate) struct Wall {
    segment: Segment,
    gaps: Vec<(f64, f64)>,
}

/// Every shared wall on the floor, with its door gaps.
pub(crate) fn walls(floor: &Floor) -> Vec<Wall> {
    let mut out = vec![];
    for (i, a) in floor.rooms.iter().enumerate() {
        for b in floor.rooms.iter().skip(i + 1) {
            if let Some(segment) = shared_edge(&a.rect, &b.rect) {
                let gaps = gaps_on(&segment, floor);
                out.push(Wall { segment, gaps });
            }
        }
    }
    out
}

/// The door gaps cut into a wall segment: each door on the wall's
/// line contributes its span, clipped to the segment. Windows are
/// not gaps — their sills block.
fn gaps_on(segment: &Segment, floor: &Floor) -> Vec<(f64, f64)> {
    let mut gaps = vec![];
    for o in floor.openings.iter().filter(|o| o.kind != OpeningKind::Window) {
        if (o.axis == Axis::Horizontal) != segment.is_horizontal() {
            continue;
        }
        let (line, c) = if o.axis == Axis::Horizontal { (o.y, o.x) } else { (o.x, o.y) };
        let wall_line = if segment.is_horizontal() { segment.y1 } else { segment.x1 };
        if (line - wall_line).abs() > 1e-9 {
            continue;
        }
        let (lo, hi) = if segment.is_horizontal() {
            (segment.x1.min(segment.x2), segment.x1.max(segment.x2))
        } else {
            (segment.y1.min(segment.y2), segment.y1.max(segment.y2))
        };
        let (a, b) = (c - o.width / 2.0, c + o.width / 2.0);
        if a < hi && b > lo {
            gaps.push((a.max(lo), b.min(hi)));
        }
    }
    gaps
}

/// True when the point is inside a wall (within half the wall
/// thickness) and outside every door gap on that wall.
pub(crate) fn blocked(walls: &[Wall], x: f64, y: f64) -> bool {
    for w in walls {
        let (perp, along, lo, hi) = if w.segment.is_horizontal() {
            (y - w.segment.y1, x, w.segment.x1.min(w.segment.x2), w.segment.x1.max(w.segment.x2))
        } else {
            (x - w.segment.x1, y, w.segment.y1.min(w.segment.y2), w.segment.y1.max(w.segment.y2))
        };
        if perp.abs() > 0.075 || along < lo || along > hi {
            continue;
        }
        if w.gaps.iter().any(|&(a, b)| along >= a && along <= b) {
            continue;
        }
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect;
    use crate::domain::model::{Opening, Program, Room};
    use crate::domain::skeleton::Skeleton;

    fn floor_with(openings: Vec<Opening>) -> Floor {
        Floor {
            level: 0,
            rooms: vec![
                Room { id: 0, program: Program::Living, rect: Rect::new(0.0, 0.0, 4.0, 4.0) },
                Room { id: 1, program: Program::Kitchen, rect: Rect::new(4.0, 0.0, 4.0, 4.0) },
            ],
            openings,
            furniture: vec![],
            open_doors: vec![],
            skeleton: Skeleton::default(),
        }
    }

    #[test]
    fn a_point_in_a_wall_is_blocked() {
        let floor = floor_with(vec![]);
        let walls = walls(&floor);
        assert!(blocked(&walls, 4.0, 2.0), "the shared wall must block");
        assert!(!blocked(&walls, 2.0, 2.0), "open floor must not block");
    }

    #[test]
    fn a_point_in_a_door_gap_is_not_blocked() {
        let door = Opening {
            id: 0,
            kind: OpeningKind::Door,
            x: 4.0,
            y: 2.0,
            axis: Axis::Vertical,
            width: 0.9,
            from: 0,
            to: Some(1),
        };
        let floor = floor_with(vec![door]);
        let walls = walls(&floor);
        assert!(!blocked(&walls, 4.0, 2.0), "the door gap must let a person through");
        assert!(blocked(&walls, 4.0, 0.5), "outside the gap the wall still blocks");
    }

    #[test]
    fn a_window_gap_is_not_a_gap() {
        let window = Opening {
            id: 0,
            kind: OpeningKind::Window,
            x: 4.0,
            y: 2.0,
            axis: Axis::Vertical,
            width: 1.2,
            from: 0,
            to: Some(1),
        };
        let floor = floor_with(vec![window]);
        let walls = walls(&floor);
        assert!(blocked(&walls, 4.0, 2.0), "a window sill must block");
    }
}
