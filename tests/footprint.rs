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
