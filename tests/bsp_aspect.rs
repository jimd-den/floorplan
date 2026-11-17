//! BSP aspect-cap behaviour, as integration tests so the
//! implementation file stays under the 200-line ceiling.
//! ADR 0014: no room above 2.0:1.

use floorplan::domain::geometry::{Axis, Rect};
use floorplan::domain::room_size::{ASPECT_HARD_CAP, aspect};
use floorplan::frameworks_drivers::rng::Rng;
use floorplan::use_cases::bsp::subdivide;

#[test]
fn no_leaf_exceeds_the_aspect_cap() {
    // ADR 0014: no room above 2.0:1. The leaves a cut
    // produces must all keep aspect <= 2.0.
    let rects = [
        Rect::new(0.0, 0.0, 12.0, 9.0),
        Rect::new(0.0, 0.0, 14.9, 12.8),
        Rect::new(0.0, 0.0, 17.4, 7.2),
        Rect::new(0.0, 0.0, 8.0, 3.9),
        Rect::new(0.0, 0.0, 14.6, 6.7),
        Rect::new(0.0, 0.0, 10.3, 9.1),
    ];
    for (ri, rect) in rects.iter().enumerate() {
        for seed in 0..20 {
            let mut rng = Rng::new(seed);
            let sub = subdivide(&mut rng, *rect, 6);
            for leaf in &sub.leaves {
                assert!(
                    aspect(leaf) <= ASPECT_HARD_CAP + 1e-6,
                    "rect {ri} seed {seed}: leaf {leaf:?} is {:.2}:1",
                    aspect(leaf)
                );
            }
        }
    }
}

#[test]
fn the_colonial_upper_floor_produces_no_strip_leaf() {
    // Regression: Colonial seed 7 floor 1 had a 2.9 x 12.8 m
    // bedroom (4.41:1) and a 2.4 x 10.3 m bedroom (4.29:1) —
    // full-depth strips from vertical cuts.
    for seed in 0..20 {
        let mut rng = Rng::new(seed);
        let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 14.9, 12.8), 9);
        for leaf in &sub.leaves {
            assert!(
                aspect(leaf) <= ASPECT_HARD_CAP + 1e-6,
                "seed {seed}: leaf {leaf:?} is {:.2}:1",
                aspect(leaf)
            );
        }
    }
}

#[test]
fn a_rect_that_admits_no_valid_single_cut_is_split_into_pieces() {
    // A 2.4 x 10.0 m rect cannot be cut into two leaves both
    // under 2.0:1, but ADR 0014 forbids emitting it whole — it
    // must be re-split into aspect-valid pieces (seed 17
    // Townhouse: a 2.9 x 13.9 rest leaf at 4.79:1). This replaces
    // the earlier documented residual, which accepted the whole
    // leaf; the multi-piece split the BSP's single cut could not
    // do is now provided by split_long_rect.
    let mut rng = Rng::new(3);
    let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 2.4, 10.0), 4);
    assert!(sub.leaves.len() > 1);
    for leaf in &sub.leaves {
        assert!(
            aspect(leaf) <= ASPECT_HARD_CAP + 1e-6,
            "leaf {leaf:?} is {:.2}:1",
            aspect(leaf)
        );
    }
}

#[test]
fn post_pass_splits_run_across_the_long_side() {
    // Regression: the post-pass recorded the split axis
    // inverted — a wide leaf (pieces side by side in x) got
    // horizontal splits, corrupting the skeleton so doors were
    // placed on the wrong edges (seed 21 Modern: a Study with
    // no door at all). A wide leaf's piece boundaries are
    // vertical lines; a tall leaf's are horizontal.
    let mut rng = Rng::new(17);
    let wide = subdivide(&mut rng, Rect::new(0.0, 0.0, 13.9, 2.9), 1);
    assert!(wide.leaves.len() > 1);
    for split in &wide.splits {
        assert_eq!(split.axis, Axis::Vertical, "wide leaf split {split:?}");
    }
    let mut rng = Rng::new(17);
    let tall = subdivide(&mut rng, Rect::new(0.0, 0.0, 2.9, 13.9), 1);
    assert!(tall.leaves.len() > 1);
    for split in &tall.splits {
        assert_eq!(split.axis, Axis::Horizontal, "tall leaf split {split:?}");
    }
}

#[test]
fn a_deep_rect_that_cannot_hold_bath_sized_pieces_stays_aspect_valid() {
    // Seed 8 Craftsman: a 13.1 x 5.7 rest. Bath-sized pieces
    // would be 1.05 m wide, under the 2.4 m floor, so the area
    // count cannot be met — but 5 equal pieces are 2.18:1. The
    // split must stay aspect-valid, never emit the invalid
    // area count.
    let mut rng = Rng::new(8);
    let sub = subdivide(&mut rng, Rect::new(2.4, 0.0, 13.1, 5.7), 5);
    assert!(sub.leaves.len() > 1);
    for leaf in &sub.leaves {
        assert!(
            aspect(leaf) <= ASPECT_HARD_CAP + 1e-6,
            "leaf {leaf:?} is {:.2}:1",
            aspect(leaf)
        );
    }
}

#[test]
fn a_long_thin_rest_leaf_is_split_despite_a_target_of_one() {
    // Seed 17 Townhouse: the rest after alcove strips is
    // 2.9 x 13.9 m (4.79:1) with a target of one leaf. The
    // aspect cap is a hard rule, so it is split into
    // aspect-valid pieces even though that exceeds the target.
    let mut rng = Rng::new(17);
    let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 2.9, 13.9), 1);
    assert!(sub.leaves.len() > 1);
    for leaf in &sub.leaves {
        assert!(
            aspect(leaf) <= ASPECT_HARD_CAP + 1e-6,
            "leaf {leaf:?} is {:.2}:1",
            aspect(leaf)
        );
    }
}
