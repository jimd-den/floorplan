//! Alcove behaviour, as integration tests so the
//! implementation file stays under the 200-line ceiling.

use floorplan::domain::geometry::Rect;
use floorplan::domain::model::Program;
use floorplan::frameworks_drivers::rng::Rng;
use floorplan::use_cases::alcove::{alcove_fits, alcove_fits_n, subdivide_with_alcove};
use floorplan::use_cases::footprint::MIN_SIDE;
#[test]
fn a_rect_big_enough_for_an_alcove_fits() {
    assert!(alcove_fits(&Rect::new(0.0, 0.0, 10.0, 8.0)));
    assert!(alcove_fits(&Rect::new(0.0, 0.0, 4.8, 4.9)));
    assert!(alcove_fits(&Rect::new(0.0, 0.0, 4.9, 4.8)));
}

#[test]
fn a_thin_rect_admits_no_alcove() {
    assert!(!alcove_fits(&Rect::new(0.0, 0.0, 20.5, 4.7)));
    assert!(!alcove_fits(&Rect::new(0.0, 0.0, 4.3, 5.7)));
    assert!(!alcove_fits(&Rect::new(0.0, 0.0, 3.1, 3.4)));
}

#[test]
fn two_strips_need_room_for_both_and_a_rest() {
    // 19.5 wide admits two west strips (4.8) plus a 2.4 rest.
    assert!(alcove_fits_n(&Rect::new(0.0, 0.0, 19.5, 11.3), 2));
    // 7.0 x 7.0 admits one strip (2.4) plus a 2.4 rest, but
    // two strips need 7.2 in one axis for the strips plus rest.
    assert!(alcove_fits_n(&Rect::new(0.0, 0.0, 7.0, 7.0), 1));
    assert!(!alcove_fits_n(&Rect::new(0.0, 0.0, 7.0, 7.0), 2));
}

#[test]
fn the_alcove_leaf_is_bath_sized_and_pinned_to_bath() {
    let mut rng = Rng::new(3);
    let sub = subdivide_with_alcove(
        &mut rng,
        Rect::new(0.0, 0.0, 10.0, 8.0),
        5,
        &[Program::Bath],
    );
    assert!(sub.pins.contains(&(0, Program::Bath)), "no bath pin: {:?}", sub.pins);
    let bath = sub.leaves[0];
    assert!(bath.area() <= 6.0 + 1e-9, "bath leaf is {} sq m", bath.area());
    assert!(bath.w >= MIN_SIDE - 1e-9 && bath.d >= MIN_SIDE - 1e-9);
}

#[test]
fn the_alcove_keeps_the_area_and_bounds_the_leaf_count() {
    // The strips are extra leaves on top of the target, and the
    // rest keeps its area share — so the count lands between
    // the target and the target plus two per strip, never below
    // the target and never inflated beyond the strips.
    let mut rng = Rng::new(3);
    let rect = Rect::new(0.0, 0.0, 10.0, 8.0);
    let sub = subdivide_with_alcove(&mut rng, rect, 5, &[Program::Bath]);
    assert!(
        (5..=7).contains(&sub.leaves.len()),
        "leaf count changed: {}",
        sub.leaves.len()
    );
    let area: f64 = sub.leaves.iter().map(Rect::area).sum();
    assert!((area - rect.area()).abs() < 1e-6, "area changed: {area}");
    for (i, a) in sub.leaves.iter().enumerate() {
        for b in sub.leaves.iter().skip(i + 1) {
            assert!(!a.overlaps(b), "leaves overlap: {a:?} {b:?}");
        }
        assert!(a.w >= MIN_SIDE - 1e-9 && a.d >= MIN_SIDE - 1e-9, "sliver: {a:?}");
    }
}

#[test]
fn the_alcove_records_one_split_per_extra_leaf() {
    let mut rng = Rng::new(3);
    let sub =
        subdivide_with_alcove(&mut rng, Rect::new(0.0, 0.0, 10.0, 8.0), 5, &[Program::Bath]);
    assert_eq!(sub.splits.len(), sub.leaves.len() - 1);
}

#[test]
fn a_study_pin_goes_on_the_alcove_leaf_itself() {
    let mut rng = Rng::new(3);
    let sub = subdivide_with_alcove(
        &mut rng,
        Rect::new(0.0, 0.0, 10.0, 7.0),
        5,
        &[Program::Study],
    );
    assert_eq!(sub.pins, vec![(0, Program::Study)]);
    assert!(sub.leaves[0].area() <= 12.0 + 1e-9);
}

#[test]
fn no_pin_means_no_pins() {
    let mut rng = Rng::new(3);
    let sub = subdivide_with_alcove(&mut rng, Rect::new(0.0, 0.0, 10.0, 8.0), 5, &[]);
    assert!(sub.pins.is_empty());
    assert_eq!(sub.leaves.len(), 5);
}

#[test]
fn a_rect_with_no_alcove_falls_back_to_a_plain_subdivision() {
    let mut rng = Rng::new(3);
    let rect = Rect::new(0.0, 0.0, 20.5, 4.7);
    let sub = subdivide_with_alcove(&mut rng, rect, 4, &[Program::Bath]);
    assert!(sub.pins.is_empty(), "a thin rect must not pin");
    let area: f64 = sub.leaves.iter().map(Rect::area).sum();
    assert!((area - rect.area()).abs() < 1e-6);
}

#[test]
fn two_pins_carve_two_bath_leaves() {
    let mut rng = Rng::new(3);
    let rect = Rect::new(0.0, 0.0, 19.5, 11.3);
    let sub = subdivide_with_alcove(&mut rng, rect, 6, &[Program::Bath, Program::Bath]);
    assert_eq!(sub.pins, vec![(0, Program::Bath), (2, Program::Bath)]);
    assert_eq!(sub.leaves[0], Rect::new(0.0, 0.0, 2.4, 2.5));
    assert_eq!(sub.leaves[2], Rect::new(2.4, 0.0, 2.4, 2.5));
    assert_eq!(sub.splits.len(), sub.leaves.len() - 1);
    let area: f64 = sub.leaves.iter().map(Rect::area).sum();
    assert!((area - rect.area()).abs() < 1e-6);
    for (i, a) in sub.leaves.iter().enumerate() {
        for b in sub.leaves.iter().skip(i + 1) {
            assert!(!a.overlaps(b), "leaves overlap: {a:?} {b:?}");
        }
        assert!(a.w >= MIN_SIDE - 1e-9 && a.d >= MIN_SIDE - 1e-9, "sliver: {a:?}");
    }
}

#[test]
fn two_pins_keep_the_rest_divided() {
    // With a low target (5) and two pins, the old count
    // (target - 2n = 1) left the 14.7 x 11.3 rest as a single
    // 166 sq m room. The area floor gives it four leaves.
    let mut rng = Rng::new(3);
    let rect = Rect::new(0.0, 0.0, 19.5, 11.3);
    let sub = subdivide_with_alcove(&mut rng, rect, 5, &[Program::Bath, Program::Bath]);
    assert!(sub.leaves.len() >= 8, "leaves: {}", sub.leaves.len());
    let largest = sub.leaves.iter().map(Rect::area).fold(0.0, f64::max);
    assert!(largest < 60.0, "largest leaf is {largest:.1} sq m");
}
