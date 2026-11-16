//! Door width by opening size: a door is sized from the shared
//! wall it sits in, not from a fixed clamp. A wide opening takes a
//! wide or double door; a narrow opening takes a single narrow
//! door. Widths are realistic throughout: a single leaf is
//! 0.8–1.0 m, a double door is two 0.8 m leaves (1.6 m), and no
//! door is narrower than 0.6 m. The width never exceeds the
//! opening minus 0.6 m, so the door keeps its jitter margin.

/// An opening this long or longer takes a double door.
pub const DOUBLE_DOOR_AT: f64 = 3.0;
/// A double door: two 0.8 m leaves.
pub const DOUBLE_DOOR_WIDTH: f64 = 1.6;
/// An opening this long or longer takes a wide single door.
pub const WIDE_DOOR_AT: f64 = 2.0;
/// A wide single door.
pub const WIDE_DOOR_WIDTH: f64 = 1.0;
/// An opening this long or longer takes a standard single door.
pub const STANDARD_DOOR_AT: f64 = 1.5;
/// A standard single door.
pub const STANDARD_DOOR_WIDTH: f64 = 0.9;
/// A narrow single door.
pub const NARROW_DOOR_WIDTH: f64 = 0.8;
/// The narrowest door that still reads as a door.
pub const MIN_DOOR_WIDTH: f64 = 0.6;

/// The width of a door in an opening `opening_len` metres long.
pub fn door_width(opening_len: f64) -> f64 {
    let want = if opening_len >= DOUBLE_DOOR_AT {
        DOUBLE_DOOR_WIDTH
    } else if opening_len >= WIDE_DOOR_AT {
        WIDE_DOOR_WIDTH
    } else if opening_len >= STANDARD_DOOR_AT {
        STANDARD_DOOR_WIDTH
    } else {
        NARROW_DOOR_WIDTH
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
    fn a_six_metre_opening_gets_a_double_door() {
        assert_eq!(door_width(6.0), 1.6, "a wide opening takes two leaves");
        assert!(is_double_door(door_width(6.0)));
    }

    #[test]
    fn a_two_and_a_half_metre_opening_gets_a_wide_single_door() {
        assert_eq!(door_width(2.5), 1.0);
        assert!(!is_double_door(door_width(2.5)));
    }

    #[test]
    fn a_one_point_eight_metre_opening_gets_a_standard_door() {
        assert_eq!(door_width(1.8), 0.9);
    }

    #[test]
    fn a_one_point_four_metre_opening_gets_a_narrow_door() {
        assert_eq!(door_width(1.4), 0.8);
    }

    #[test]
    fn a_tiny_opening_is_clamped_by_the_opening_itself() {
        assert_eq!(door_width(1.0), 0.6, "the door keeps its 0.6 m margin");
        assert_eq!(door_width(0.9), 0.6, "never narrower than a door");
    }

    #[test]
    fn the_width_never_exceeds_the_opening_minus_its_margin() {
        for len in [1.0f64, 1.5, 2.0, 2.5, 3.0, 4.0, 6.0, 10.0] {
            let cap = (len - 0.6).max(MIN_DOOR_WIDTH);
            assert!(
                door_width(len) <= cap + 1e-9,
                "len {len}: width {} spills past the opening",
                door_width(len)
            );
        }
    }
}
