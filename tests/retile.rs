//! Re-tiling targets, as integration tests so the
//! implementation file stays under the 200-line ceiling.

use floorplan::domain::geometry::Rect as R;
use floorplan::domain::geometry::Rect;
use floorplan::domain::model::{Program, Program as P, Room};
use floorplan::use_cases::retile::{find_retile_target, over_cap_rooms};

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

#[test]
fn a_room_sized_hall_is_a_target() {
    // ADR 0014: a hall that is dimensionally a habitable
    // room (the Colonial 4.1 x 6.4 m hall) must be re-tiled,
    // not emitted as a hall.
    let rooms = vec![room(0, P::Hall, R::new(0.0, 0.0, 4.1, 6.4))];
    assert_eq!(over_cap_rooms(&rooms).len(), 1);
}

#[test]
fn a_foyer_sized_hall_is_not_a_target() {
    let rooms = vec![room(0, P::Hall, R::new(0.0, 0.0, 2.4, 2.5))];
    assert!(over_cap_rooms(&rooms).is_empty());
}

#[test]
fn a_room_sized_entry_is_a_target() {
    // The Modern 8.0 x 3.9 m entry: 31.2 sq m of entry.
    let rooms = vec![room(0, P::Entry, R::new(0.0, 0.0, 8.0, 3.9))];
    assert_eq!(over_cap_rooms(&rooms).len(), 1);
}

#[test]
fn a_room_sized_hall_pins_an_alcove() {
    let rects = vec![R::new(0.0, 0.0, 14.9, 12.8)];
    let rooms = vec![room(0, P::Hall, R::new(4.8, 5.3, 6.3, 7.5))];
    let target = find_retile_target(&rooms, &rects, &[]).expect("a target is expected");
    assert_eq!(target.rect, 0);
    assert_eq!(target.program, P::Hall);
}

#[test]
fn a_room_sized_entry_outranks_an_over_cap_bath_when_a_bath_already_fits() {
    // ADR 0015 (seed 45 Modern): the floor already has a
    // fitting bath, so the over-cap bath can fall back to
    // demotion to laundry — the room-sized entry must get the
    // strip first, or it stays room-sized forever.
    let rects = vec![R::new(0.0, 0.0, 10.0, 8.0)];
    let rooms = vec![
        room(0, P::Bath, R::new(0.0, 0.0, 2.4, 2.5)),
        room(1, P::Bath, R::new(2.4, 0.0, 5.0, 2.5)),
        room(2, P::Entry, R::new(0.0, 2.5, 7.6, 3.9)),
    ];
    let target = find_retile_target(&rooms, &rects, &[]).expect("a target is expected");
    assert_eq!(target.program, P::Entry);
}

#[test]
fn a_capped_room_is_targeted_before_a_room_sized_hall() {
    // When a rect admits only one more strip, the hard cap
    // (a Bath over 6 sq m) is fixed before a room-sized
    // hall: the cap test must never lose its strip to a hall.
    // (Study is not a retile target: a pinned study alcove
    // becomes laundry via the choke, so the assignment's
    // fitting-leaf rule fixes studies, not pinning.)
    let rects = vec![R::new(0.0, 0.0, 7.0, 7.0)];
    let rooms = vec![
        room(0, P::Hall, R::new(0.0, 0.0, 3.0, 3.0)),
        room(1, P::Bath, R::new(3.0, 0.0, 3.0, 2.5)),
    ];
    let target = find_retile_target(&rooms, &rects, &[]).expect("a target is expected");
    assert_eq!(target.program, P::Bath);
}
