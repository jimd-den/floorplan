//! The study's reference table: typical room sizes and aspect
//! target bands per program. These are NOT enforced — the
//! enforced rules (the IRC floor, the 2.0 aspect hard cap, the
//! room-sized classification) live in `room_size.rs`. This
//! table is what the enforcement is measured against: a room
//! that misses its typical size is a finding, not a violation.
//! Every number is a conversion of a study figure; nothing is
//! invented.

use crate::domain::model::Program;

/// Aspect target band (length:width), best general range.
pub const ASPECT_TARGET: f64 = 1.6;

/// The aspect target band (low, high) for a program, from the
/// study: bedrooms/dining 1.0-1.4, living 1.3-1.6, galley
/// kitchen up to 1.7, everything else the general 1.0-1.6.
pub fn aspect_target(program: Program) -> (f64, f64) {
    match program {
        Program::Bedroom | Program::Dining => (1.0, 1.4),
        Program::Living => (1.3, 1.6),
        Program::Kitchen => (1.0, 1.7),
        _ => (1.0, ASPECT_TARGET),
    }
}

/// The study's typical size (width, depth) in metres for a
/// program, or None for a hall — a hall is a width, not a
/// rectangle. Master bedroom is the 12x14 ft queen; the plain
/// Bedroom is the 10x12 ft secondary.
pub fn typical_size(program: Program) -> Option<(f64, f64)> {
    match program {
        Program::Bedroom => Some((3.05, 3.66)),
        Program::Living => Some((3.96, 5.18)),
        Program::Kitchen => Some((3.05, 3.66)),
        Program::Dining => Some((3.35, 4.27)),
        Program::Bath => Some((1.83, 2.74)),
        Program::Study => Some((3.05, 3.35)),
        Program::Laundry => Some((1.83, 2.44)),
        Program::Entry => Some((1.83, 2.44)),
        Program::Garage => Some((6.71, 6.71)),
        Program::Hall => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typical_sizes_come_from_the_study() {
        assert_eq!(typical_size(Program::Bedroom), Some((3.05, 3.66)));
        assert_eq!(typical_size(Program::Living), Some((3.96, 5.18)));
        assert_eq!(typical_size(Program::Kitchen), Some((3.05, 3.66)));
        assert_eq!(typical_size(Program::Dining), Some((3.35, 4.27)));
        assert_eq!(typical_size(Program::Bath), Some((1.83, 2.74)));
        assert_eq!(typical_size(Program::Study), Some((3.05, 3.35)));
        assert_eq!(typical_size(Program::Laundry), Some((1.83, 2.44)));
        assert_eq!(typical_size(Program::Entry), Some((1.83, 2.44)));
        assert_eq!(typical_size(Program::Garage), Some((6.71, 6.71)));
        assert_eq!(typical_size(Program::Hall), None);
    }

    #[test]
    fn aspect_target_bands_come_from_the_study() {
        assert_eq!(aspect_target(Program::Bedroom), (1.0, 1.4));
        assert_eq!(aspect_target(Program::Dining), (1.0, 1.4));
        assert_eq!(aspect_target(Program::Living), (1.3, 1.6));
        assert_eq!(aspect_target(Program::Kitchen), (1.0, 1.7));
        assert_eq!(aspect_target(Program::Study), (1.0, 1.6));
    }
}
