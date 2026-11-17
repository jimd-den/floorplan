//! Footprint behaviour, as integration tests so the
//! implementation file stays under the 200-line ceiling.

use floorplan::domain::geometry::{Rect, Segment};
use floorplan::domain::model::Footprint;
use floorplan::domain::style::{FootprintShape, Style};
use floorplan::frameworks_drivers::rng::Rng;
use floorplan::use_cases::alcove::alcove_fits;
use floorplan::use_cases::footprint::{footprint, footprint_with_shape};
#[test]
fn rectangle_footprint_is_one_rect_with_no_joins() {
    let mut rng = Rng::new(1);
    let fp = footprint_with_shape(&mut rng, Style::Ranch, FootprintShape::Rectangle);
    assert_eq!(fp.rects.len(), 1);
    assert!(fp.joins.is_empty());
}

#[test]
fn l_shape_has_two_rects_and_one_join() {
    let mut rng = Rng::new(1);
    let fp = footprint_with_shape(&mut rng, Style::Ranch, FootprintShape::L);
    assert_eq!(fp.rects.len(), 2);
    assert_eq!(shape_joins(&fp), 1);
}

#[test]
fn u_shape_has_three_rects_and_two_joins() {
    let mut rng = Rng::new(1);
    let fp = footprint_with_shape(&mut rng, Style::Ranch, FootprintShape::U);
    assert_eq!(fp.rects.len(), 3);
    assert_eq!(shape_joins(&fp), 2);
}

#[test]
fn t_shape_has_two_rects_and_one_join() {
    let mut rng = Rng::new(1);
    let fp = footprint_with_shape(&mut rng, Style::Craftsman, FootprintShape::T);
    assert_eq!(fp.rects.len(), 2);
    assert_eq!(shape_joins(&fp), 1);
}

/// Joins between the shape's own rectangles, excluding the garage join.
fn shape_joins(fp: &Footprint) -> usize {
    fp.joins.len() - usize::from(fp.garage.is_some())
}

#[test]
fn every_footprint_has_a_rect_that_admits_a_bath_alcove() {
    // The re-tiling loop needs a bath-capable rect; a
    // footprint without one cannot host a compliant bath.
    for seed in 0..200 {
        let mut rng = Rng::new(seed);
        for style in Style::all() {
            let fp = footprint(&mut rng, style);
            assert!(
                fp.rects.iter().any(alcove_fits),
                "seed {seed} {:?} has no bath-capable rect: {:?}",
                style,
                fp.rects
            );
        }
    }
}

#[test]
fn wings_are_wide_enough_for_an_alcove_strip() {
    // ADR 0015: a wing narrower than 4.8 m can neither hold an
    // alcove strip (2.4 + 2.4 m) nor a foyer entry (3.6 x 2.4
    // is 8.64 sq m, room-sized) — seeds 60 and 80. Wing
    // widths are drawn from 4.8 m, not 3.0 m.
    for shape in [FootprintShape::L, FootprintShape::U, FootprintShape::T] {
        for seed in 0..200 {
            let mut rng = Rng::new(seed);
            for style in Style::all() {
                let fp = footprint_with_shape(&mut rng, style, shape);
                for wing in fp.rects.iter().skip(1) {
                    assert!(
                        wing.w >= 4.8 - 1e-9,
                        "seed {seed} {style:?} {shape:?}: wing is {} m wide",
                        wing.w
                    );
                }
            }
        }
    }
}

#[test]
fn wing_depths_avoid_the_room_sized_strip_rest_range() {
    // ADR 0015 (seed 116 Craftsman T): a 5.58 m stem leaves a
    // 2.4 x 3.08 m strip rest (7.39 sq m) — room-sized, and
    // unsplittable (3.08 < 4.8 m), so no southern leaf can be
    // a foyer. A wing/stem depth in (5.208, 7.3) m leaves a
    // strip rest in (2.708, 4.8) m: room-sized and
    // unsplittable. Wing depths must stay out of that range.
    for shape in [FootprintShape::L, FootprintShape::U, FootprintShape::T] {
        for style in Style::all() {
            for seed in 0..200u32 {
                let mut rng = Rng::new(seed);
                let fp = footprint_with_shape(&mut rng, style, shape);
                for rect in fp.rects.iter().skip(1) {
                    assert!(
                        rect.d <= 5.208 + 1e-9 || rect.d >= 7.3 - 1e-9,
                        "{shape:?} {style:?} seed {seed}: wing depth {:.2} m leaves a room-sized strip rest",
                        rect.d
                    );
                }
            }
        }
    }
}

#[test]
fn footprint_rects_never_overlap() {
    for seed in 0..50 {
        let mut rng = Rng::new(seed);
        for style in Style::all() {
            let fp = footprint(&mut rng, style);
            for (i, a) in fp.rects.iter().enumerate() {
                for b in fp.rects.iter().skip(i + 1) {
                    assert!(!a.overlaps(b), "seed {seed} style {:?}", style);
                }
            }
        }
    }
}

#[test]
fn footprint_dimensions_stay_in_the_style_range() {
    for seed in 0..50 {
        let mut rng = Rng::new(seed);
        for style in Style::all() {
            let spec = style.spec();
            let fp = footprint(&mut rng, style);
            let w = fp.rects.iter().map(|r| r.right()).fold(0.0, f64::max);
            let d = fp.rects.iter().map(|r| r.bottom()).fold(0.0, f64::max);
            assert!(w >= spec.width.0 && w <= spec.width.1, "{w} for {:?}", style);
            assert!(d >= spec.depth.0 && d <= spec.depth.1, "{d} for {:?}", style);
        }
    }
}

#[test]
fn garage_when_present_touches_the_house_with_a_join() {
    for seed in 0..50 {
        let mut rng = Rng::new(seed);
        let fp = footprint(&mut rng, Style::Ranch);
        if let Some(garage) = fp.garage {
            assert_eq!(garage.w, 6.0);
            assert_eq!(garage.d, 6.0);
            let touches = fp
                .rects
                .iter()
                .any(|r| r.edges().iter().any(|e| e.overlap(&garage_edge(&garage)).is_some()));
            assert!(touches, "garage does not touch the house");
            return;
        }
    }
    panic!("no garage appeared in 50 seeds");
}

fn garage_edge(garage: &Rect) -> Segment {
    Segment::new(garage.x, garage.y, garage.x, garage.bottom())
}
