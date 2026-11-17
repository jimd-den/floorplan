//! Demoting a residual over-cap bath: a bath the re-tiling
//! loop could not fix (no rect admits an alcove for it) must
//! not be emitted over its 6 sq m cap. Laundry is uncapped,
//! so the room fits. Only when another bath already fits —
//! a house must keep a bath, even an oversized one, over
//! losing its only bath.

use crate::domain::model::{Program, Room};
use crate::use_cases::caps::cap_for;

/// Demotes every over-cap bath to Laundry, when at least one
/// bath already fits its cap. A house with no fitting bath
/// keeps its baths as they are.
pub fn demote_over_cap_baths(rooms: &mut [Room]) {
    let has_fitting_bath = rooms.iter().any(|r| {
        r.program == Program::Bath
            && cap_for(Program::Bath).is_some_and(|cap| r.rect.area() <= cap + 1e-9)
    });
    if !has_fitting_bath {
        return;
    }
    for room in rooms.iter_mut() {
        if room.program == Program::Bath
            && cap_for(Program::Bath).is_some_and(|cap| room.rect.area() > cap + 1e-9)
        {
            room.program = Program::Laundry;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    #[test]
    fn a_residual_over_cap_bath_becomes_laundry_when_another_bath_fits() {
        // Seed 60 Ranch: a 9.35 sq m second bath in a rect too
        // shallow for an alcove, beside a 6.0 sq m bath. The
        // re-tiling loop cannot fix it, so it must not be
        // emitted over its cap.
        let mut rooms = vec![
            room(0, Program::Bath, Rect::new(0.0, 0.0, 2.5, 3.7)),
            room(1, Program::Bath, Rect::new(2.5, 0.0, 2.4, 2.5)),
        ];
        demote_over_cap_baths(&mut rooms);
        assert_eq!(rooms[0].program, Program::Laundry);
        assert_eq!(rooms[1].program, Program::Bath);
    }

    #[test]
    fn the_only_bath_is_kept_even_when_over_cap() {
        // A house must keep a bath over losing its only one.
        let mut rooms = vec![room(0, Program::Bath, Rect::new(0.0, 0.0, 2.5, 3.7))];
        demote_over_cap_baths(&mut rooms);
        assert_eq!(rooms[0].program, Program::Bath);
    }
}
