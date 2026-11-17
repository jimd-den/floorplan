//! Door widths: realistic residential sizes, by room, not by
//! opening. A real house has a 0.9 m front door, 0.8 m interior
//! doors, 0.7 m doors to small rooms (a closet, bath, or laundry
//! under 6 m²), and at most one double door — a wide opening
//! between two main rooms — capped at 1.2 m (two 0.6 m leaves).
//! The opening length never sizes a door: a long shared wall
//! does not make a wide doorway. No door is wider than its
//! opening minus the 0.6 m jitter margin, and no door is
//! narrower than 0.6 m.

/// The front door: a single 0.9 m door, whatever the front.
pub const ENTRY_DOOR_WIDTH: f64 = 0.9;
/// A double door: two 0.6 m leaves, capped at 1.2 m.
pub const DOUBLE_DOOR_WIDTH: f64 = 1.2;
/// An opening this long or longer, between two main rooms, may
/// carry the floor's one double door — which goes on the
/// living/dining opening, the one place a real house puts one.
pub const DOUBLE_DOOR_AT: f64 = 3.0;
/// A standard interior door.
pub const STANDARD_DOOR_WIDTH: f64 = 0.8;
/// A door to a small room (closet, bath, laundry).
pub const SMALL_DOOR_WIDTH: f64 = 0.7;
/// A room under this area is small: its door is the narrow one.
pub const SMALL_ROOM_AREA: f64 = 6.0;
/// The narrowest door that still reads as a door.
pub const MIN_DOOR_WIDTH: f64 = 0.6;

/// The width of the entry (front) door: a single 0.9 m door.
pub fn entry_door_width() -> f64 {
    ENTRY_DOOR_WIDTH
}

/// True when a room of this area is small: its door is the narrow
/// one, never a double.
pub fn is_small_room(area: f64) -> bool {
    area < SMALL_ROOM_AREA
}

/// True when an opening of this length between two main rooms may
/// carry the floor's one double door.
pub fn is_double_opening(opening_len: f64, small: bool) -> bool {
    opening_len >= DOUBLE_DOOR_AT && !small
}

/// The width of an interior door in an opening `opening_len`
/// metres long, where `small` says either room it joins is small
/// and `double` says this opening carries the floor's one double
/// door (the living/dining opening). A wide opening to a small
/// room takes a small single — a real house never puts a double
/// door on a closet — and a wide opening that is not the
/// designated double takes a standard single: a long shared wall
/// does not make a wide doorway.
pub fn interior_door_width(opening_len: f64, small: bool, double: bool) -> f64 {
    let want = if double && !small {
        DOUBLE_DOOR_WIDTH
    } else if small {
        SMALL_DOOR_WIDTH
    } else {
        STANDARD_DOOR_WIDTH
    };
    // Never wider than the opening minus its jitter margin, never
    // narrower than a door. Quantized to 0.1 m like every other
    // seeded dimension, so 1.4 - 0.6 reads 0.8, not 0.7999.
    crate::use_cases::quant(want.min(opening_len - 0.6).max(MIN_DOOR_WIDTH))
}

/// True when a door of this width is a double door (two leaves).
pub fn is_double_door(width: f64) -> bool {
    width >= DOUBLE_DOOR_WIDTH - 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_entry_door_is_a_single_nine_tenths_door() {
        assert_eq!(entry_door_width(), 0.9);
        assert!(!is_double_door(entry_door_width()));
    }

    #[test]
    fn the_designated_double_door_is_twelve_tenths() {
        assert_eq!(interior_door_width(6.0, false, true), 1.2, "two 0.6 m leaves");
        assert!(is_double_door(interior_door_width(6.0, false, true)));
    }

    #[test]
    fn a_wide_opening_that_is_not_the_double_gets_a_standard_single() {
        assert_eq!(
            interior_door_width(6.0, false, false),
            0.8,
            "a long shared wall does not make a wide doorway"
        );
        assert!(!is_double_door(interior_door_width(6.0, false, false)));
    }

    #[test]
    fn a_wide_opening_to_a_small_room_gets_a_small_single() {
        assert_eq!(interior_door_width(6.0, true, true), 0.7, "no double on a closet");
        assert!(!is_double_door(interior_door_width(6.0, true, true)));
    }

    #[test]
    fn a_standard_opening_gets_an_eight_tenths_door() {
        assert_eq!(interior_door_width(1.8, false, false), 0.8);
        assert!(!is_double_door(interior_door_width(1.8, false, false)));
    }

    #[test]
    fn a_small_room_gets_a_seven_tenths_door() {
        assert_eq!(interior_door_width(1.8, true, false), 0.7);
    }

    #[test]
    fn a_room_under_six_square_metres_is_small() {
        assert!(is_small_room(5.9));
        assert!(!is_small_room(6.0), "at the threshold it is a main room");
        assert!(!is_small_room(30.0));
    }

    #[test]
    fn a_double_opening_is_wide_and_between_main_rooms() {
        assert!(is_double_opening(3.0, false));
        assert!(!is_double_opening(2.9, false), "under the threshold");
        assert!(!is_double_opening(6.0, true), "never on a small room");
    }

    #[test]
    fn a_tiny_opening_is_clamped_by_the_opening_itself() {
        assert_eq!(interior_door_width(1.0, false, false), 0.6, "the door keeps its 0.6 m margin");
        assert_eq!(interior_door_width(0.9, true, false), 0.6, "never narrower than a door");
    }

    #[test]
    fn the_width_never_exceeds_the_opening_minus_its_margin() {
        for len in [1.0f64, 1.5, 2.0, 2.5, 3.0, 4.0, 6.0, 10.0] {
            for small in [false, true] {
                for double in [false, true] {
                    let cap = (len - 0.6).max(MIN_DOOR_WIDTH);
                    assert!(
                        interior_door_width(len, small, double) <= cap + 1e-9,
                        "len {len} small {small} double {double}: width {} spills past the opening",
                        interior_door_width(len, small, double)
                    );
                }
            }
        }
    }
}
