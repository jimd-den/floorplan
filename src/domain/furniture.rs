//! Furniture: what a room holds, and the state each piece carries.
//! Pure data; placement lives in `use_cases::furnish`, mesh in
//! `use_cases::extrude`, interaction in `use_cases::interact`.
//! See ADR 0003 and spec §4.2 / §5.

use crate::domain::geometry::Rect;

/// Which way a piece's front faces: into the room, away from the
/// wall it stands against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    North,
    South,
    East,
    West,
}

/// Kinds of furniture. Sizes are the footprint in metres when the
/// piece faces south (width along x, depth along y); facing east
/// or west swaps the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FurnitureKind {
    Bed,
    Nightstand,
    Table,
    Chair,
    Sofa,
    Lamp,
    Desk,
    Tub,
    Sink,
    Shelf,
    Cabinet,
    Counter,
}

impl Facing {
    /// Lower-case name for the JSON writer.
    pub fn name(&self) -> &'static str {
        match self {
            Facing::North => "north",
            Facing::South => "south",
            Facing::East => "east",
            Facing::West => "west",
        }
    }
}

impl FurnitureKind {
    /// Lower-case name for the JSON writer and the material table.
    pub fn name(&self) -> &'static str {
        match self {
            FurnitureKind::Bed => "bed",
            FurnitureKind::Nightstand => "nightstand",
            FurnitureKind::Table => "table",
            FurnitureKind::Chair => "chair",
            FurnitureKind::Sofa => "sofa",
            FurnitureKind::Lamp => "lamp",
            FurnitureKind::Desk => "desk",
            FurnitureKind::Tub => "tub",
            FurnitureKind::Sink => "sink",
            FurnitureKind::Shelf => "shelf",
            FurnitureKind::Cabinet => "cabinet",
            FurnitureKind::Counter => "counter",
        }
    }

    /// Footprint (width, depth) in metres, facing south.
    pub fn size(&self) -> (f64, f64) {
        match self {
            FurnitureKind::Bed => (1.6, 2.1),
            FurnitureKind::Nightstand => (0.5, 0.45),
            FurnitureKind::Table => (1.8, 1.0),
            FurnitureKind::Chair => (0.45, 0.45),
            FurnitureKind::Sofa => (2.0, 0.9),
            FurnitureKind::Lamp => (0.35, 0.35),
            FurnitureKind::Desk => (1.4, 0.7),
            FurnitureKind::Tub => (1.7, 0.8),
            FurnitureKind::Sink => (0.6, 0.5),
            FurnitureKind::Shelf => (1.0, 0.35),
            FurnitureKind::Cabinet => (1.2, 0.6),
            FurnitureKind::Counter => (1.5, 0.6),
        }
    }

    /// The state a fresh piece of this kind starts in.
    pub fn default_state(&self) -> FurnitureState {
        match self {
            FurnitureKind::Lamp => FurnitureState::Off,
            FurnitureKind::Cabinet => FurnitureState::Closed,
            _ => FurnitureState::Fixed,
        }
    }

    /// Pieces the player can move: chairs and tables.
    pub fn is_movable(&self) -> bool {
        matches!(self, FurnitureKind::Chair | FurnitureKind::Table)
    }

    /// Tall pieces that block a window when stood against its
    /// wall. Low pieces (bed, table, sofa, …) sit below the sill
    /// and leave the light alone.
    pub fn is_tall(&self) -> bool {
        matches!(self, FurnitureKind::Shelf | FurnitureKind::Cabinet | FurnitureKind::Counter)
    }
}

/// Interaction state, per kind. A moved piece stores its current
/// position; a lamp its on/off; a cabinet its doors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FurnitureState {
    /// No interaction (bed, tub, sink, sofa, desk, shelf, counter).
    Fixed,
    /// A lamp, on.
    On,
    /// A lamp, off.
    Off,
    /// A cabinet with its doors open.
    Open,
    /// A cabinet with its doors closed.
    Closed,
    /// A moved chair or table at its current (x, y).
    Moved { x: f64, y: f64 },
}

/// One placed piece of furniture.
#[derive(Debug, Clone, PartialEq)]
pub struct Furniture {
    /// Unique across the house; commands address pieces by it.
    pub id: u32,
    pub kind: FurnitureKind,
    pub room_id: u32,
    pub rect: Rect,
    pub facing: Facing,
    pub state: FurnitureState,
}

impl Furniture {
    pub fn new(id: u32, kind: FurnitureKind, room_id: u32, rect: Rect, facing: Facing) -> Self {
        let state = if kind.is_movable() {
            FurnitureState::Moved { x: rect.x, y: rect.y }
        } else {
            kind.default_state()
        };
        Self { id, kind, room_id, rect, facing, state }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bed_is_1_6_by_2_1() {
        assert_eq!(FurnitureKind::Bed.size(), (1.6, 2.1));
    }

    #[test]
    fn a_lamp_starts_off_and_a_cabinet_starts_closed() {
        assert_eq!(FurnitureKind::Lamp.default_state(), FurnitureState::Off);
        assert_eq!(FurnitureKind::Cabinet.default_state(), FurnitureState::Closed);
        assert_eq!(FurnitureKind::Bed.default_state(), FurnitureState::Fixed);
    }

    #[test]
    fn only_chairs_and_tables_are_movable() {
        assert!(FurnitureKind::Chair.is_movable());
        assert!(FurnitureKind::Table.is_movable());
        assert!(!FurnitureKind::Bed.is_movable());
        assert!(!FurnitureKind::Lamp.is_movable());
    }

    #[test]
    fn a_new_chair_stores_its_placed_position() {
        let f = Furniture::new(
            7,
            FurnitureKind::Chair,
            3,
            Rect::new(1.0, 2.0, 0.45, 0.45),
            Facing::North,
        );
        assert_eq!(f.state, FurnitureState::Moved { x: 1.0, y: 2.0 });
        assert_eq!(f.room_id, 3);
    }

    #[test]
    fn a_new_lamp_starts_off() {
        let f = Furniture::new(
            8,
            FurnitureKind::Lamp,
            3,
            Rect::new(1.0, 2.0, 0.35, 0.35),
            Facing::North,
        );
        assert_eq!(f.state, FurnitureState::Off);
    }
}
