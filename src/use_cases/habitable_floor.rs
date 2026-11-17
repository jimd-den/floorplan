//! The habitable-floor choke (ADR 0014): a habitable program
//! on a sub-floor leaf is a code violation, so the leaf becomes
//! Laundry, which needs no window and has no area minimum.
//! Baths are already non-habitable, so they are never touched.
//! The choke runs at the end of program assignment, after the
//! pins are re-applied, so it also catches a pinned Study on a
//! sub-floor leaf.

use crate::domain::geometry::Rect;
use crate::domain::model::{Program, Room};
use crate::domain::room_size::meets_habitable_floor;

/// A habitable program on a sub-floor leaf becomes Laundry.
pub(crate) fn enforce_habitable_floor(leaves: &[Rect], out: &mut [Program]) {
    for (i, program) in out.iter_mut().enumerate() {
        if program.is_habitable() && !meets_habitable_floor(&leaves[i]) {
            *program = Program::Laundry;
        }
    }
}

/// The same choke over rooms (not leaves): a habitable program
/// on a sub-floor rect becomes Laundry. Runs after the window
/// repair in `build_floor`, which can swap a habitable program
/// onto a sub-floor leaf after the assignment-time choke ran
/// (seed 3 Modern: a Study swapped onto a 6.48 sq m leaf).
pub(crate) fn enforce_habitable_floor_rooms(rooms: &mut [Room]) {
    for room in rooms.iter_mut() {
        if room.program.is_habitable() && !meets_habitable_floor(&room.rect) {
            room.program = Program::Laundry;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sub_floor_study_becomes_laundry() {
        // The Colonial studies at 5.76 and 6.00 sq m: below the
        // 6.5 sq m IRC floor, so they cannot stay Study.
        let leaves = vec![
            Rect::new(0.0, 0.0, 4.0, 4.0),
            Rect::new(4.0, 0.0, 2.4, 2.4),
            Rect::new(4.0, 2.4, 2.4, 2.5),
        ];
        let mut out = vec![Program::Living, Program::Study, Program::Study];
        enforce_habitable_floor(&leaves, &mut out);
        assert_eq!(out[1], Program::Laundry, "a 5.76 sq m study is below the floor");
        assert_eq!(out[2], Program::Laundry, "a 6.00 sq m study is below the floor");
        assert_eq!(out[0], Program::Living, "the living room is untouched");
    }

    #[test]
    fn a_small_bath_is_not_touched_by_the_floor() {
        let leaves = vec![Rect::new(0.0, 0.0, 2.4, 2.5)];
        let mut out = vec![Program::Bath];
        enforce_habitable_floor(&leaves, &mut out);
        assert_eq!(out[0], Program::Bath, "a bath has no area minimum");
    }

    #[test]
    fn a_swapped_in_sub_floor_study_becomes_laundry() {
        // Seed 3 Modern: the window repair swapped a Study onto
        // a 2.4 x 2.7 (6.48 sq m) leaf after the assignment-time
        // choke ran, so it stayed a sub-floor Study.
        let mut rooms = vec![Room {
            id: 11,
            program: Program::Study,
            rect: Rect::new(15.7, 6.6, 2.4, 2.7),
        }];
        enforce_habitable_floor_rooms(&mut rooms);
        assert_eq!(rooms[0].program, Program::Laundry);
    }
}
