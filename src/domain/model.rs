//! The plan model: House, Floor, Room, Opening, Style. Pure data, no I/O.

use crate::domain::base::Base;
use crate::domain::geometry::{Rect, Segment};
use crate::domain::light::LightSource;
use crate::domain::skeleton::Skeleton;
use crate::domain::style::Style;

/// What a room is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Program {
    Living,
    Kitchen,
    Dining,
    Bedroom,
    Bath,
    Hall,
    Entry,
    Study,
    Laundry,
    Garage,
}

impl Program {
    pub fn name(&self) -> &'static str {
        match self {
            Program::Living => "living",
            Program::Kitchen => "kitchen",
            Program::Dining => "dining",
            Program::Bedroom => "bedroom",
            Program::Bath => "bath",
            Program::Hall => "hall",
            Program::Entry => "entry",
            Program::Study => "study",
            Program::Laundry => "laundry",
            Program::Garage => "garage",
        }
    }

    /// Rooms that need daylight: at least one window each.
    pub fn is_habitable(&self) -> bool {
        matches!(
            self,
            Program::Living
                | Program::Kitchen
                | Program::Dining
                | Program::Bedroom
                | Program::Study
        )
    }
}

/// One room: a program on a rectangle.
#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    pub id: u32,
    pub program: Program,
    pub rect: Rect,
}

/// Kinds of openings in walls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpeningKind {
    Door,
    EntryDoor,
    Window,
}

/// One opening, centred at (x, y) on a wall running along `axis`.
/// `to` is None for exterior openings (entry door, windows).
#[derive(Debug, Clone, PartialEq)]
pub struct Opening {
    pub id: u32,
    pub kind: OpeningKind,
    pub x: f64,
    pub y: f64,
    pub axis: crate::domain::geometry::Axis,
    pub width: f64,
    pub from: u32,
    pub to: Option<u32>,
}

impl Opening {
    /// True when this opening shares wall length with `other`: same
    /// axis, same wall line, and intersecting spans. Touching ends do
    /// not count — two openings may sit side by side.
    pub fn overlaps(&self, other: &Opening) -> bool {
        if self.axis != other.axis {
            return false;
        }
        let (line_a, a0, a1) = self.span();
        let (line_b, b0, b1) = other.span();
        (line_a - line_b).abs() < 1e-9 && a0 < b1 && b0 < a1
    }
    /// The wall line coordinate and the span along it.
    fn span(&self) -> (f64, f64, f64) {
        use crate::domain::geometry::Axis;
        match self.axis {
            Axis::Horizontal => (self.y, self.x - self.width / 2.0, self.x + self.width / 2.0),
            Axis::Vertical => (self.x, self.y - self.width / 2.0, self.y + self.width / 2.0),
        }
    }
}

/// One storey of the house.
#[derive(Debug, Clone, PartialEq)]
pub struct Floor {
    pub level: u8,
    pub rooms: Vec<Room>,
    pub openings: Vec<Opening>,
    /// Placed furniture (spec §4.2); empty until `furnish` runs.
    pub furniture: Vec<crate::domain::furniture::Furniture>,
    /// Ids of doors that are open (spec §4.2); all closed by default.
    pub open_doors: Vec<u32>,
    /// The logical skeleton: the rooms' shared-wall links and the
    /// entry room. Walls and doors are built from it.
    pub skeleton: Skeleton,
}

/// The footprint: the rectangles that tile the house outline, the
/// segments where they join, and optional porch / garage rectangles.
#[derive(Debug, Clone, PartialEq)]
pub struct Footprint {
    pub rects: Vec<Rect>,
    pub joins: Vec<Segment>,
    pub porch: Option<Rect>,
    pub garage: Option<Rect>,
}

/// A whole house.
#[derive(Debug, Clone, PartialEq)]
pub struct House {
    pub seed: u32,
    pub style: Style,
    pub footprint: Footprint,
    /// The foundation slab, built before any room. Its top is floor
    /// level; the walls and floors sit on it.
    pub base: Base,
    pub floors: Vec<Floor>,
    /// (ground hall, upper hall) room ids, for two-storey styles.
    pub stairs: Option<(u32, u32)>,
    /// The house's light sources and their state (SRD-001 FR-22).
    /// Derived after furnishing, so lamps and rooms exist.
    pub lights: Vec<LightSource>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Axis;

    fn opening(id: u32, kind: OpeningKind, x: f64, y: f64, axis: Axis, width: f64) -> Opening {
        Opening { id, kind, x, y, axis, width, from: 0, to: None }
    }

    #[test]
    fn openings_on_the_same_line_overlap_when_their_spans_intersect() {
        let a = opening(0, OpeningKind::EntryDoor, 3.0, 0.0, Axis::Horizontal, 1.0);
        let b = opening(1, OpeningKind::Window, 3.5, 0.0, Axis::Horizontal, 1.8);
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a), "overlap is symmetric");
    }

    #[test]
    fn openings_that_only_touch_do_not_overlap() {
        let a = opening(0, OpeningKind::Window, 2.0, 0.0, Axis::Horizontal, 1.0);
        let b = opening(1, OpeningKind::Window, 3.0, 0.0, Axis::Horizontal, 1.0);
        assert!(!a.overlaps(&b), "end-to-end openings share only a point");
    }

    #[test]
    fn openings_on_different_walls_or_axes_never_overlap() {
        let a = opening(0, OpeningKind::Window, 2.0, 0.0, Axis::Horizontal, 1.2);
        let same_span_other_wall = opening(1, OpeningKind::Window, 2.0, 5.0, Axis::Horizontal, 1.2);
        let crossing = opening(2, OpeningKind::Door, 0.0, 2.0, Axis::Vertical, 0.9);
        assert!(!a.overlaps(&same_span_other_wall));
        assert!(!a.overlaps(&crossing));
    }
}
