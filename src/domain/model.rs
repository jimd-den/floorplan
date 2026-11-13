//! The plan model: House, Floor, Room, Opening, Style. Pure data, no I/O.

use crate::domain::geometry::{Rect, Segment};
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
    pub floors: Vec<Floor>,
    /// (ground hall, upper hall) room ids, for two-storey styles.
    pub stairs: Option<(u32, u32)>,
}
