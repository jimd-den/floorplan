//! BSP subdivision behaviour, as integration tests so the
//! implementation file stays under the 200-line ceiling.

use floorplan::domain::geometry::Rect;
use floorplan::frameworks_drivers::rng::Rng;
use floorplan::use_cases::bsp::subdivide;
use floorplan::use_cases::footprint::MIN_SIDE;

#[test]
fn splits_a_rect_into_the_target_leaf_count() {
    let mut rng = Rng::new(5);
    let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 12.0, 9.0), 4);
    assert_eq!(sub.leaves.len(), 4);
}

#[test]
fn leaves_tile_the_rect_exactly() {
    let mut rng = Rng::new(5);
    let rect = Rect::new(0.0, 0.0, 12.0, 9.0);
    let sub = subdivide(&mut rng, rect, 5);
    let area: f64 = sub.leaves.iter().map(Rect::area).sum();
    assert!((area - rect.area()).abs() < 1e-6, "area {area}");
    for (i, a) in sub.leaves.iter().enumerate() {
        for b in sub.leaves.iter().skip(i + 1) {
            assert!(!a.overlaps(b));
        }
    }
}

#[test]
fn no_leaf_is_a_sliver() {
    let mut rng = Rng::new(9);
    let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 12.0, 9.0), 6);
    for leaf in &sub.leaves {
        assert!(leaf.w >= MIN_SIDE - 1e-9, "w {}", leaf.w);
        assert!(leaf.d >= MIN_SIDE - 1e-9, "d {}", leaf.d);
    }
}

#[test]
fn stops_early_when_no_leaf_can_split() {
    let mut rng = Rng::new(2);
    let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 5.0, 5.0), 8);
    assert!(sub.leaves.len() < 8);
    for leaf in &sub.leaves {
        assert!(leaf.w >= MIN_SIDE - 1e-9 && leaf.d >= MIN_SIDE - 1e-9);
    }
}

#[test]
fn one_split_is_recorded_per_extra_leaf() {
    let mut rng = Rng::new(5);
    let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 12.0, 9.0), 4);
    assert_eq!(sub.splits.len(), sub.leaves.len() - 1);
}

#[test]
fn split_positions_are_quantized_to_a_tenth() {
    let mut rng = Rng::new(11);
    let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 15.0, 11.0), 6);
    for split in &sub.splits {
        let q = (split.at * 10.0).round() / 10.0;
        assert!((split.at - q).abs() < 1e-9, "at {}", split.at);
    }
}

#[test]
fn no_leaf_lands_in_the_dead_zone() {
    // A leaf in (6.0, 6.5) sq m is dead: too big for a Bath
    // (cap 6.0), too small for a habitable room (floor 6.5).
    // Every BSP leaf must be bath-takable or habitable-capable.
    let rects = [
        Rect::new(0.0, 0.0, 2.4, 12.5),
        Rect::new(0.0, 0.0, 2.5, 12.5),
        Rect::new(0.0, 0.0, 2.6, 10.0),
        Rect::new(0.0, 0.0, 2.6, 12.5),
        Rect::new(0.0, 0.0, 2.4, 8.1),
        Rect::new(0.0, 0.0, 2.5, 7.0),
        Rect::new(0.0, 0.0, 4.0, 7.0),
        Rect::new(0.0, 0.0, 2.4, 5.4),
    ];
    for (i, rect) in rects.iter().enumerate() {
        for seed in 0..20u32 {
            let mut rng = Rng::new(seed * 100 + i as u32);
            let sub = subdivide(&mut rng, *rect, 5);
            for leaf in &sub.leaves {
                let a = leaf.area();
                assert!(
                    a <= 6.0 + 1e-9 || a >= 6.5 - 1e-9,
                    "rect {rect:?} seed {seed}: leaf {leaf:?} is {a:.2} sq m, in the dead zone"
                );
            }
        }
    }
}
