//! Shrinking a room-sized entry to a foyer (ADR 0015).
//!
//! When the entry is dimensionally a room but shares its full
//! north edge with a neighbour that can absorb the depth, the
//! shared wall moves south: the entry becomes a 2.4 m foyer
//! from the front and the neighbour grows. The shared split
//! moves with the wall so the skeleton stays coherent. No-op
//! when no neighbour can absorb (a capped bath, a hall, or a
//! neighbour that would break aspect or its cap).

use crate::domain::geometry::{Axis, Rect, Segment, shared_edge};
use crate::domain::model::{Program, Room};
use crate::domain::room_size::{ASPECT_HARD_CAP, aspect, is_room_sized};
use crate::use_cases::bsp::Split;
use crate::use_cases::caps::cap_for;
use crate::use_cases::footprint::MIN_SIDE;

/// Shrinks every room-sized entry that has an absorbent
/// northern neighbour. A neighbour absorbs when it sits
/// directly on the entry (same width, same x), is not
/// circulation, and stays within aspect and its program cap
/// after growing by the freed depth.
pub(crate) fn shrink_room_sized_entry(rooms: &mut [Room], splits: &mut [Split]) {
    for i in 0..rooms.len() {
        if rooms[i].program != Program::Entry || !is_room_sized(&rooms[i].rect) {
            continue;
        }
        let Some(j) = absorbent_neighbour(rooms, i) else { continue };
        let give = rooms[i].rect.d - MIN_SIDE;
        let new_y = rooms[i].rect.y + give;
        // Move the shared split with the wall, so the
        // skeleton still walls it.
        let shared = shared_edge(&rooms[i].rect, &rooms[j].rect).unwrap();
        for split in splits.iter_mut() {
            if split.axis == Axis::Horizontal && split.line == shared {
                split.at = new_y;
                split.line = Segment::new(shared.x1, new_y, shared.x2, new_y);
            }
        }
        rooms[j].rect.d += give;
        rooms[i].rect.y = new_y;
        rooms[i].rect.d = MIN_SIDE;
    }
}

/// The northern neighbour of room `i`, if it sits directly on
/// the entry (same width and x) and can absorb the freed
/// depth without breaking aspect or its program cap.
fn absorbent_neighbour(rooms: &[Room], i: usize) -> Option<usize> {
    let entry = rooms[i].rect;
    // A 2.4 m foyer is room-sized when the entry is wider
    // than 6.5 / 2.4 = 2.71 m — no depth can fix it.
    if entry.w > 2.7 {
        return None;
    }
    let give = entry.d - MIN_SIDE;
    if give <= 1e-9 {
        return None;
    }
    // ADR 0015 (10): an over-cap bath is demoted to Laundry
    // by the re-tiling loop's exit step when a fitting bath
    // exists, so it is effectively uncapped and can absorb.
    // Without a fitting bath it is kept, and cannot absorb.
    let has_fitting_bath = rooms.iter().any(|r| {
        r.program == Program::Bath
            && cap_for(r.program).is_some_and(|cap| r.rect.area() <= cap + 1e-9)
    });
    rooms.iter().position(|r| {
        r.rect.x == entry.x
            && r.rect.right() == entry.right()
            && (r.rect.bottom() - entry.y).abs() < 1e-9
            && r.program != Program::Hall
            && r.program != Program::Entry
            && {
                let grown = Rect::new(r.rect.x, r.rect.y, r.rect.w, r.rect.d + give);
                aspect(&grown) <= ASPECT_HARD_CAP + 1e-9
                    && (cap_for(r.program).is_none_or(|cap| grown.area() <= cap + 1e-9)
                        || (r.program == Program::Bath
                            && has_fitting_bath
                            && cap_for(r.program)
                                .is_some_and(|cap| r.rect.area() > cap + 1e-9)))
            }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    #[test]
    fn a_room_sized_entry_shrinks_to_a_foyer_and_the_neighbour_grows() {
        // Seed 101 Craftsman: the entry is 2.6 x 3.4 m (8.84
        // sq m) under a 2.6 x 3.12 m study. The study absorbs
        // 1.0 m and stays under its 12 sq m cap at 1.58:1.
        let mut rooms = vec![
            room(0, Program::Entry, Rect::new(7.1, 7.8, 2.6, 3.4)),
            room(1, Program::Study, Rect::new(7.1, 4.68, 2.6, 3.12)),
        ];
        let mut splits = vec![Split {
            axis: Axis::Horizontal,
            at: 7.8,
            line: Segment::new(7.1, 7.8, 9.7, 7.8),
        }];
        shrink_room_sized_entry(&mut rooms, &mut splits);
        assert_eq!(rooms[0].rect, Rect::new(7.1, 8.8, 2.6, 2.4));
        assert!(!is_room_sized(&rooms[0].rect), "the entry is still room-sized");
        assert_eq!(rooms[1].rect, Rect::new(7.1, 4.68, 2.6, 4.12));
        assert_eq!(splits[0].at, 8.8);
        assert_eq!(splits[0].line, Segment::new(7.1, 8.8, 9.7, 8.8));
        let area: f64 = rooms.iter().map(|r| r.rect.area()).sum();
        assert!((area - 16.952).abs() < 1e-9, "area changed: {area}");
    }

    #[test]
    fn an_over_cap_bath_neighbour_absorbs_when_another_bath_fits() {
        // Seed 45 Modern round 2: the entry's northern
        // neighbour is a 2.7 x 3.88 m bath (10.46 sq m, over
        // its 6.0 cap), beside a fitting 2.4 x 2.5 m bath.
        // The re-tiling loop demotes the over-cap bath to
        // Laundry, so it is effectively uncapped and absorbs
        // the entry's depth.
        let mut rooms = vec![
            room(0, Program::Entry, Rect::new(2.4, 8.6, 2.7, 2.9)),
            room(1, Program::Bath, Rect::new(2.4, 4.72, 2.7, 3.88)),
            room(2, Program::Bath, Rect::new(0.0, 4.72, 2.4, 2.5)),
        ];
        let mut splits = vec![Split {
            axis: Axis::Horizontal,
            at: 8.6,
            line: Segment::new(2.4, 8.6, 5.1, 8.6),
        }];
        shrink_room_sized_entry(&mut rooms, &mut splits);
        assert_eq!(rooms[0].rect, Rect::new(2.4, 9.1, 2.7, 2.4));
        assert!(!is_room_sized(&rooms[0].rect), "the entry is still room-sized");
        assert_eq!(rooms[1].rect, Rect::new(2.4, 4.72, 2.7, 4.38));
    }

    #[test]
    fn an_over_cap_bath_neighbour_blocks_when_it_is_the_only_bath() {
        // No fitting bath: the over-cap bath is kept (a house
        // must keep a bath), so it cannot absorb.
        let mut rooms = vec![
            room(0, Program::Entry, Rect::new(2.4, 8.6, 2.7, 2.9)),
            room(1, Program::Bath, Rect::new(2.4, 4.72, 2.7, 3.88)),
        ];
        let mut splits = vec![Split {
            axis: Axis::Horizontal,
            at: 8.6,
            line: Segment::new(2.4, 8.6, 5.1, 8.6),
        }];
        shrink_room_sized_entry(&mut rooms, &mut splits);
        assert_eq!(rooms[0].rect, Rect::new(2.4, 8.6, 2.7, 2.9));
        assert_eq!(rooms[1].rect, Rect::new(2.4, 4.72, 2.7, 3.88));
    }

    #[test]
    fn a_capped_neighbour_blocks_the_shrink() {
        // The neighbour is a bath at its 6.0 sq m cap: growing
        // it would break the cap, so the entry stays.
        let mut rooms = vec![
            room(0, Program::Entry, Rect::new(0.0, 2.5, 2.6, 3.4)),
            room(1, Program::Bath, Rect::new(0.0, 0.0, 2.6, 2.5)),
        ];
        let mut splits = vec![Split {
            axis: Axis::Horizontal,
            at: 2.5,
            line: Segment::new(0.0, 2.5, 2.6, 2.5),
        }];
        shrink_room_sized_entry(&mut rooms, &mut splits);
        assert_eq!(rooms[0].rect, Rect::new(0.0, 2.5, 2.6, 3.4));
        assert_eq!(rooms[1].rect, Rect::new(0.0, 0.0, 2.6, 2.5));
    }

    #[test]
    fn a_seed_45_entry_shrinks_against_a_laundry_neighbour() {
        // Seed 45 Modern: the entry is 2.7 x 2.9 m (7.83 sq
        // m) under a 2.7 x 3.88 m laundry. The laundry is
        // uncapped, so it absorbs 0.5 m and the entry becomes
        // a 2.7 x 2.4 m (6.48 sq m) foyer.
        let mut rooms = vec![
            room(0, Program::Entry, Rect::new(2.4, 8.6, 2.7, 2.9)),
            room(1, Program::Laundry, Rect::new(2.4, 4.72, 2.7, 3.88)),
        ];
        let mut splits = vec![Split {
            axis: Axis::Horizontal,
            at: 8.6,
            line: Segment::new(2.4, 8.6, 5.1, 8.6),
        }];
        shrink_room_sized_entry(&mut rooms, &mut splits);
        assert_eq!(rooms[0].rect, Rect::new(2.4, 9.1, 2.7, 2.4));
        assert!(!is_room_sized(&rooms[0].rect), "the entry is still room-sized");
        assert_eq!(rooms[1].rect, Rect::new(2.4, 4.72, 2.7, 4.38));
    }
}
