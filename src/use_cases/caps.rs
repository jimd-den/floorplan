//! Room-size caps: a bath is at most 6 sq m, a study at most 12.
//!
//! The assignment reserves sub-cap leaves for baths (see
//! `repair::assign_bed_bath`); this step splits an oversized bath
//! or study leaf where the leaf allows it — a strip cut across the
//! long side, the capped room keeping a 2.4 m strip, the remainder
//! becoming dining or laundry. A leaf too small in one dimension to
//! split without a sliver is left alone: a 6 sq m bath needs a
//! 2.4 x 2.5 m room, and most leaves cannot host one (see ADR 0002).

use crate::domain::model::{Program, Room};
use crate::use_cases::bsp::Split;
use crate::use_cases::cap_cut::{pick_cut, strip_cuts};

/// The cap for a program, in square metres; None means uncapped.
pub fn cap_for(program: Program) -> Option<f64> {
    match program {
        Program::Bath => Some(6.0),
        Program::Study => Some(12.0),
        _ => None,
    }
}

/// Splits oversized baths and studies down to their cap where the
/// leaf allows a strip cut; records the new split so the rooms stay
/// connected. Leaves that cannot be cut without a sliver are left.
pub fn cap_room_sizes(rooms: &mut Vec<Room>, splits: &mut Vec<Split>, next_room: &mut u32) {
    let original = rooms.len();
    for i in 0..original {
        let Some(cap) = cap_for(rooms[i].program) else { continue };
        if rooms[i].rect.area() <= cap + 1e-9 {
            continue;
        }
        let Some(cuts) = strip_cuts(&rooms[i].rect, cap) else { continue };
        let (small, remainder, split, ext) = pick_cut(rooms, i, &cuts);
        let has_dining = rooms.iter().any(|r| r.program == Program::Dining);
        let program = if ext >= 1.2 && !has_dining { Program::Dining } else { Program::Laundry };
        let id = *next_room;
        *next_room += 1;
        rooms[i].rect = small;
        rooms.push(Room { id, program, rect: remainder });
        splits.push(split);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect;
    use crate::frameworks_drivers::rng::Rng;
    use crate::use_cases::doors::place_doors;
    use crate::use_cases::placement::longest_exterior;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    /// A living room and a bath leaf that can be strip-cut: 2.5 m
    /// wide, 8 m deep, so the bath keeps a 2.5 x 2.4 m strip.
    fn cuttable_floor() -> (Vec<Room>, Vec<Split>) {
        let rooms = vec![
            room(0, Program::Living, Rect::new(0.0, 0.0, 10.0, 8.0)),
            room(1, Program::Bath, Rect::new(10.0, 0.0, 2.5, 8.0)),
        ];
        (rooms, vec![])
    }

    #[test]
    fn bath_cap_is_six_square_metres() {
        assert_eq!(cap_for(Program::Bath), Some(6.0));
        assert_eq!(cap_for(Program::Study), Some(12.0));
        assert_eq!(cap_for(Program::Living), None);
    }

    #[test]
    fn an_oversized_bath_leaf_is_split_into_a_capped_bath_and_a_remainder() {
        let (mut rooms, mut splits) = cuttable_floor();
        let mut next_room = 2;
        cap_room_sizes(&mut rooms, &mut splits, &mut next_room);
        assert_eq!(rooms.len(), 3, "the leaf should split into two rooms");
        assert_eq!(splits.len(), 1, "the cut should be recorded as a split");
        let bath = rooms.iter().find(|r| r.program == Program::Bath).unwrap();
        assert!(bath.rect.area() <= 6.0 + 1e-9, "bath is {} sq m", bath.rect.area());
        let area: f64 = rooms.iter().map(|r| r.rect.area()).sum();
        assert!((area - 100.0).abs() < 1e-9, "area changed: {area}");
    }

    #[test]
    fn an_oversized_study_leaf_is_split_the_same_way() {
        let mut rooms = vec![
            room(0, Program::Living, Rect::new(0.0, 0.0, 10.0, 8.0)),
            room(1, Program::Study, Rect::new(10.0, 0.0, 4.0, 8.0)),
        ];
        let mut splits = vec![];
        let mut next_room = 2;
        cap_room_sizes(&mut rooms, &mut splits, &mut next_room);
        assert_eq!(rooms.len(), 3);
        let study = rooms.iter().find(|r| r.program == Program::Study).unwrap();
        assert!(study.rect.area() <= 12.0 + 1e-9, "study is {} sq m", study.rect.area());
    }

    #[test]
    fn a_leaf_too_small_to_carve_is_left_alone() {
        // 4.9 x 4.2 m: no strip cut yields a 6 sq m bath and a
        // remainder both at least 2.4 m on every side.
        let mut rooms = vec![
            room(0, Program::Living, Rect::new(0.0, 0.0, 10.0, 8.0)),
            room(1, Program::Bath, Rect::new(10.0, 0.0, 4.9, 4.2)),
        ];
        let mut splits = vec![];
        let mut next_room = 2;
        cap_room_sizes(&mut rooms, &mut splits, &mut next_room);
        assert_eq!(rooms.len(), 2, "an uncarvable leaf must not split");
        assert!(splits.is_empty());
    }

    #[test]
    fn the_remainder_keeps_an_exterior_edge_and_becomes_dining() {
        let (mut rooms, mut splits) = cuttable_floor();
        let mut next_room = 2;
        cap_room_sizes(&mut rooms, &mut splits, &mut next_room);
        let remainder = rooms.iter().find(|r| r.id == 2).unwrap();
        assert_eq!(remainder.program, Program::Dining, "remainder should be dining");
        let rects: Vec<Rect> = rooms.iter().map(|r| r.rect).collect();
        let idx = rooms.iter().position(|r| r.id == 2).unwrap();
        assert!(longest_exterior(&rects, idx) >= 1.2, "dining needs a window wall");
    }

    #[test]
    fn the_split_gets_a_door_between_the_capped_room_and_the_remainder() {
        let (mut rooms, mut splits) = cuttable_floor();
        let mut next_room = 2;
        cap_room_sizes(&mut rooms, &mut splits, &mut next_room);
        let mut rng = Rng::new(3);
        let doors = place_doors(&mut rng, &rooms, &splits, &[], 0);
        let remainder = rooms.iter().find(|r| r.id == 2).unwrap().id;
        let bath = rooms.iter().find(|r| r.program == Program::Bath).unwrap().id;
        assert!(
            doors.iter().any(|d| (d.from == bath && d.to == Some(remainder))
                || (d.from == remainder && d.to == Some(bath))),
            "no door connects the bath to its remainder"
        );
    }
}
