//! Re-tiling targets: which over-cap room to fix, and where.
//!
//! The loop in `floor` rebuilds a floor with one more alcove per
//! round; this module finds the next room that needs one. A room
//! qualifies when its footprint rect admits an alcove and that
//! rect is not already pinned — one rect holds one pin.

use crate::domain::geometry::Rect;
use crate::domain::model::{Program, Room};
use crate::use_cases::alcove::alcove_fits_n;
use crate::use_cases::caps::cap_for;

/// One alcove to add: the footprint rect (by index) and the
/// program its alcove leaf is pinned to.
pub struct RetileTarget {
    pub rect: usize,
    pub program: Program,
}

/// The rooms on a floor that exceed their program cap, in order.
pub fn over_cap_rooms(rooms: &[Room]) -> Vec<&Room> {
    rooms
        .iter()
        .filter(|r| {
            cap_for(r.program).is_some_and(|cap| r.rect.area() > cap + 1e-9)
        })
        .collect()
}

/// The footprint rect containing `room`, if any. Rooms from a cap
/// split stay inside their rect; the garage room is in none.
pub fn containing_rect(rects: &[Rect], room: &Room) -> Option<usize> {
    rects.iter().position(|r| {
        room.rect.x >= r.x - 1e-9
            && room.rect.y >= r.y - 1e-9
            && room.rect.right() <= r.right() + 1e-9
            && room.rect.bottom() <= r.bottom() + 1e-9
    })
}

/// The next re-tiling target: the first over-cap room whose rect
/// admits one more alcove strip. A rect can hold several strips,
/// one per pin, so a rect already pinned for one bath can take a
/// second. When the room's own rect cannot take a strip — a thin
/// wing that can never hold a bath leaf — a different rect that
/// admits an alcove is pinned instead: the pinned leaf satisfies
/// the bath requirement, so the bounded bath count leaves this
/// room a non-bath program. Rooms in a footprint with no
/// alcove-admitting rect at all are skipped here; they are the
/// genuine residual.
pub fn find_retile_target(
    rooms: &[Room],
    rects: &[Rect],
    pinned: &[(usize, Program)],
) -> Option<RetileTarget> {
    for room in over_cap_rooms(rooms) {
        let Some(rect) = containing_rect(rects, room) else { continue };
        let strips = pinned.iter().filter(|&&(r, _)| r == rect).count() + 1;
        if alcove_fits_n(&rects[rect], strips) {
            return Some(RetileTarget { rect, program: room.program });
        }
        for (i, r) in rects.iter().enumerate() {
            let strips_i = pinned.iter().filter(|&&(rr, _)| rr == i).count() + 1;
            if alcove_fits_n(r, strips_i) {
                return Some(RetileTarget { rect: i, program: room.program });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect as R;
    use crate::domain::model::Program as P;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    #[test]
    fn an_over_cap_bath_in_a_big_rect_is_a_target() {
        let rects = vec![R::new(0.0, 0.0, 10.0, 8.0)];
        let rooms = vec![room(0, P::Bath, R::new(0.0, 0.0, 5.0, 8.0))];
        let target = find_retile_target(&rooms, &rects, &[]).expect("a target is expected");
        assert_eq!(target.rect, 0);
        assert_eq!(target.program, P::Bath);
    }

    #[test]
    fn a_room_in_a_thin_rect_is_skipped_when_no_rect_admits_an_alcove() {
        let rects = vec![R::new(0.0, 0.0, 20.5, 4.7)];
        let rooms = vec![room(0, P::Bath, R::new(0.0, 0.0, 4.7, 4.8))];
        assert!(find_retile_target(&rooms, &rects, &[]).is_none());
    }

    #[test]
    fn a_thin_rect_bath_pins_a_different_rect_that_admits_an_alcove() {
        // The bath sits in a thin wing (20.5 x 4.7, no alcove),
        // but the main rect (10 x 8) admits one: pin there.
        let rects = vec![R::new(0.0, 0.0, 20.5, 4.7), R::new(0.0, 4.7, 10.0, 8.0)];
        let rooms = vec![room(0, P::Bath, R::new(0.0, 0.0, 5.0, 4.7))];
        let target = find_retile_target(&rooms, &rects, &[]).expect("a target is expected");
        assert_eq!(target.rect, 1);
        assert_eq!(target.program, P::Bath);
    }

    #[test]
    fn an_already_pinned_rect_takes_a_second_strip_when_it_fits() {
        let rects = vec![R::new(0.0, 0.0, 19.5, 11.3)];
        let rooms = vec![room(0, P::Bath, R::new(0.0, 0.0, 5.0, 8.0))];
        let target = find_retile_target(&rooms, &rects, &[(0, P::Bath)])
            .expect("a second strip should fit");
        assert_eq!(target.rect, 0);
    }

    #[test]
    fn a_rect_full_of_strips_is_skipped() {
        let rects = vec![R::new(0.0, 0.0, 7.0, 7.0)];
        let rooms = vec![room(0, P::Bath, R::new(0.0, 0.0, 5.0, 7.0))];
        assert!(find_retile_target(&rooms, &rects, &[(0, P::Bath)]).is_none());
    }

    #[test]
    fn a_room_within_its_cap_is_not_a_target() {
        let rects = vec![R::new(0.0, 0.0, 10.0, 8.0)];
        let rooms = vec![room(0, P::Bath, R::new(0.0, 0.0, 2.4, 2.5))];
        assert!(find_retile_target(&rooms, &rects, &[]).is_none());
    }
}
