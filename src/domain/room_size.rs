//! Room-size rules: the study's numbers, encoded once.
//!
//! The source of truth is the room-size study
//! (`research_notes/residential-room-sizes-standards-20261117-0244/report.md`):
//! the IRC 2021 legal floor (R304.1/R304.2), the aspect bands,
//! and the hall/entry classification. Every number here is a
//! conversion of a study figure (feet/inches to metres); nothing
//! is invented. Only the hard floor, the aspect hard cap, and
//! the room-sized classification are enforced — the typical
//! sizes and target bands are the reference table the
//! enforcement is measured against.

use crate::domain::geometry::Rect;

/// Habitable-room floor area, IRC R304.1: 70 sq ft = 6.5 sq m.
pub const HABITABLE_MIN_AREA: f64 = 6.5;

/// Habitable-room least horizontal dimension, IRC R304.2: 7 ft.
pub const HABITABLE_MIN_SIDE: f64 = 2.13;

/// Short side below which a furnished room cannot hold
/// furniture on both long walls: 8 ft 6 in.
pub const FURNISHED_MIN_SHORT_SIDE: f64 = 2.59;

/// Aspect hard cap: no room is emitted above 2.0:1.
pub const ASPECT_HARD_CAP: f64 = 2.0;

/// Hallway minimum width, IRC R311.6: 36 in.
pub const HALL_MIN_WIDTH: f64 = 0.91;

/// Hallway comfortable width: 42 in.
pub const HALL_WIDTH: f64 = 1.07;

/// Width at which a passage starts to read as a room: 48 in.
pub const HALL_GENEROUS_WIDTH: f64 = 1.22;

/// Area under which a space is circulation, not a room: 40 sq ft.
pub const CIRCULATION_MAX_AREA: f64 = 3.7;

/// Foyer minimum area: 48 sq ft (the typical 6x8 ft foyer).
pub const FOYER_MIN_AREA: f64 = 4.5;

/// Foyer minimum width: 5 ft.
pub const FOYER_MIN_WIDTH: f64 = 1.52;

/// Grand foyer area: 100 sq ft.
pub const GRAND_FOYER_AREA: f64 = 9.3;

/// Length divided by width, always >= 1.0.
pub fn aspect(rect: &Rect) -> f64 {
    rect.w.max(rect.d) / rect.w.min(rect.d)
}

/// The shorter side of a rect, in metres.
pub fn short_side(rect: &Rect) -> f64 {
    rect.w.min(rect.d)
}

/// True when a rect meets the IRC habitable floor: area >= 6.5
/// sq m and every side >= 2.13 m.
pub fn meets_habitable_floor(rect: &Rect) -> bool {
    rect.area() >= HABITABLE_MIN_AREA - 1e-9 && short_side(rect) >= HABITABLE_MIN_SIDE - 1e-9
}

/// What a hall/entry-shaped space reads as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CirculationKind {
    /// A passage: short side <= 1.22 m or area < 3.7 sq m.
    Circulation,
    /// An entry room: short side >= 1.52 m and area >= 4.5 sq m.
    Foyer,
    /// A grand entry: area >= 9.3 sq m, short side under 2.13 m.
    GrandFoyer,
    /// Dimensionally a habitable room: area >= 6.5 sq m and short
    /// side >= 2.13 m. Must not be emitted as a hall or entry.
    RoomSized,
}

/// Classifies a hall/entry rect by width and area (study §4,
/// an inference, not code). Room-sized is checked first: a
/// space that is dimensionally a habitable room is room-sized
/// whatever its area alone would say.
pub fn classify_circulation(rect: &Rect) -> CirculationKind {
    if is_room_sized(rect) {
        CirculationKind::RoomSized
    } else if rect.area() >= GRAND_FOYER_AREA - 1e-9 {
        CirculationKind::GrandFoyer
    } else if short_side(rect) >= FOYER_MIN_WIDTH - 1e-9
        && rect.area() >= FOYER_MIN_AREA - 1e-9
    {
        CirculationKind::Foyer
    } else {
        CirculationKind::Circulation
    }
}

/// True when a hall/entry rect is room-sized: area >= 6.5 sq m
/// AND short side >= 2.13 m. The enforcement predicate.
pub fn is_room_sized(rect: &Rect) -> bool {
    rect.area() >= HABITABLE_MIN_AREA - 1e-9 && short_side(rect) >= HABITABLE_MIN_SIDE - 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aspect_is_long_over_short() {
        assert!((aspect(&Rect::new(0.0, 0.0, 2.9, 12.8)) - 4.41).abs() < 0.01);
        assert!((aspect(&Rect::new(0.0, 0.0, 7.2, 9.5)) - 1.32).abs() < 0.01);
        assert!((aspect(&Rect::new(0.0, 0.0, 4.0, 4.0)) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn habitable_floor_is_area_and_short_side() {
        // The Colonial studies: 2.4 x 2.4 (5.76) and 2.4 x 2.5
        // (6.00) are below the 6.5 sq m floor.
        assert!(!meets_habitable_floor(&Rect::new(0.0, 0.0, 2.4, 2.4)));
        assert!(!meets_habitable_floor(&Rect::new(0.0, 0.0, 2.4, 2.5)));
        // 2.4 x 2.7 is 6.48 sq m: still under.
        assert!(!meets_habitable_floor(&Rect::new(0.0, 0.0, 2.4, 2.7)));
        // 2.5 x 2.7 is 6.75 sq m with a 2.5 m short side: passes.
        assert!(meets_habitable_floor(&Rect::new(0.0, 0.0, 2.5, 2.7)));
        // 2.0 x 4.0 is 8.0 sq m but the short side is under 2.13 m.
        assert!(!meets_habitable_floor(&Rect::new(0.0, 0.0, 2.0, 4.0)));
    }

    #[test]
    fn circulation_classification_follows_width_and_area() {
        // A 1.0 m wide passage is circulation whatever its length.
        assert_eq!(
            classify_circulation(&Rect::new(0.0, 0.0, 1.0, 5.0)),
            CirculationKind::Circulation
        );
        // A 1.5 x 2.0 m space is under 3.7 sq m: circulation.
        assert_eq!(
            classify_circulation(&Rect::new(0.0, 0.0, 1.5, 2.0)),
            CirculationKind::Circulation
        );
        // The alcove leaf, 2.4 x 2.5 m (6.0 sq m): a foyer, not
        // room-sized — its area is under 6.5 sq m.
        assert_eq!(
            classify_circulation(&Rect::new(0.0, 0.0, 2.4, 2.5)),
            CirculationKind::Foyer
        );
        // A 2.0 x 5.0 m passage (10 sq m, short side under 2.13):
        // grand foyer, not a room.
        assert_eq!(
            classify_circulation(&Rect::new(0.0, 0.0, 2.0, 5.0)),
            CirculationKind::GrandFoyer
        );
        // The Ranch entry, 3.2 x 3.6 m: room-sized.
        assert_eq!(
            classify_circulation(&Rect::new(0.0, 0.0, 3.2, 3.6)),
            CirculationKind::RoomSized
        );
        // The Colonial hall, 7.2 x 9.5 m: room-sized.
        assert_eq!(
            classify_circulation(&Rect::new(0.0, 0.0, 7.2, 9.5)),
            CirculationKind::RoomSized
        );
        // The Modern entry, 8.0 x 3.9 m: room-sized.
        assert_eq!(
            classify_circulation(&Rect::new(0.0, 0.0, 8.0, 3.9)),
            CirculationKind::RoomSized
        );
    }

    #[test]
    fn is_room_sized_is_area_and_short_side_together() {
        // 2.0 x 3.9 m is 7.8 sq m but its short side is under
        // 2.13 m: not room-sized.
        assert!(!is_room_sized(&Rect::new(0.0, 0.0, 2.0, 3.9)));
        // 2.4 x 2.5 m is 6.0 sq m: under the area floor.
        assert!(!is_room_sized(&Rect::new(0.0, 0.0, 2.4, 2.5)));
        // 2.5 x 2.7 m is 6.75 sq m with a 2.5 m short side.
        assert!(is_room_sized(&Rect::new(0.0, 0.0, 2.5, 2.7)));
    }
}
