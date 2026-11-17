//! Basement: the below-grade level every house gets.
//!
//! A basement is the main footprint rect dug down: concrete
//! foundation walls from the footing to grade, a concrete floor
//! slab, and a straight stair down from the ground-floor hall.
//! The rect is the main (largest) footprint rect, so its size
//! varies with the seed; the depth is seeded too (2.2, 2.4, or
//! 2.6 m). The stair runs along the basement's long axis, starting
//! at the end nearest the hall, and always fits inside the rect.

use crate::domain::geometry::{Axis, Rect};
use crate::domain::model::Footprint;

/// The stair tread (run per step), in metres.
pub const BASEMENT_TREAD: f64 = 0.25;
/// The stair rise per step, in metres.
pub const BASEMENT_RISE: f64 = 0.2;
/// The foundation wall thickness, in metres.
pub const FOUNDATION_WALL: f64 = 0.2;
/// The basement floor slab thickness, in metres.
pub const BASEMENT_SLAB: f64 = 0.1;

/// A basement level below grade.
#[derive(Debug, Clone, PartialEq)]
pub struct Basement {
    /// The basement footprint: the main (largest) footprint rect.
    pub rect: Rect,
    /// How deep the basement floor sits below grade (y = 0).
    pub depth: f64,
    /// The ground-floor room the stair descends from (the hall).
    pub stair_from: u32,
    /// The axis the stair runs along (the basement's long axis).
    pub stair_axis: Axis,
    /// The stair's top step position (x, y) in plan.
    pub stair_top: (f64, f64),
    /// The stair run length in plan, top step to landing.
    pub stair_run: f64,
}

impl Default for Basement {
    fn default() -> Self {
        Basement {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            depth: 2.4,
            stair_from: 0,
            stair_axis: Axis::Horizontal,
            stair_top: (0.0, 0.0),
            stair_run: 0.0,
        }
    }
}

impl Basement {
    /// The basement for a footprint: the main rect, a seeded depth,
    /// and a stair from the hall. `hall_center` is the hall room's
    /// centre in plan; the stair starts at the rect end nearest it.
    pub fn from_footprint(
        fp: &Footprint,
        seed: u32,
        stair_from: u32,
        hall_center: (f64, f64),
    ) -> Basement {
        let rect = fp
            .rects
            .iter()
            .max_by(|a, b| a.area().partial_cmp(&b.area()).unwrap())
            .copied()
            .unwrap_or(Rect::new(0.0, 0.0, 1.0, 1.0));
        // Depth 2.2, 2.4, or 2.6 m by seed: a whole number of
        // 0.2 m rises, so the stair has an exact step count.
        let depth = 2.2 + (seed % 3) as f64 * 0.2;
        let steps = (depth / BASEMENT_RISE).round();
        let stair_run = steps * BASEMENT_TREAD;
        // The stair runs along the long axis, starting at the end
        // nearest the hall, clamped so the run fits the rect.
        let (stair_axis, stair_top) = if rect.w >= rect.d {
            let lo = rect.x;
            let hi = rect.right() - stair_run;
            let start = if hall_center.0 <= rect.x + rect.w / 2.0 { lo } else { hi };
            let start = start.clamp(lo, hi.max(lo));
            (Axis::Horizontal, (start, rect.y + rect.d / 2.0))
        } else {
            let lo = rect.y;
            let hi = rect.bottom() - stair_run;
            let start = if hall_center.1 <= rect.y + rect.d / 2.0 { lo } else { hi };
            let start = start.clamp(lo, hi.max(lo));
            (Axis::Vertical, (rect.x + rect.w / 2.0, start))
        };
        Basement { rect, depth, stair_from, stair_axis, stair_top, stair_run }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Segment;

    fn footprint() -> Footprint {
        Footprint {
            rects: vec![Rect::new(0.0, 0.0, 12.0, 8.0), Rect::new(0.0, 8.0, 6.0, 3.0)],
            joins: vec![Segment::new(0.0, 8.0, 6.0, 8.0)],
            porch: None,
            garage: None,
        }
    }

    #[test]
    fn the_basement_covers_the_main_rect() {
        let b = Basement::from_footprint(&footprint(), 42, 1, (6.0, 4.0));
        assert_eq!(b.rect, Rect::new(0.0, 0.0, 12.0, 8.0), "basement should be the largest rect");
    }

    #[test]
    fn the_basement_depth_is_seeded_between_two_point_two_and_two_point_six() {
        for seed in [0u32, 1, 2, 3, 42, 100] {
            let b = Basement::from_footprint(&footprint(), seed, 1, (6.0, 4.0));
            assert!(
                (2.2..=2.6).contains(&b.depth),
                "seed {seed} depth {} outside 2.2..=2.6",
                b.depth
            );
            assert!(
                ((b.depth / 0.2).round() - b.depth / 0.2).abs() < 1e-9,
                "depth {} is not a whole number of 0.2 m rises",
                b.depth
            );
        }
    }

    #[test]
    fn the_stair_fits_inside_the_basement_rect() {
        let b = Basement::from_footprint(&footprint(), 42, 1, (6.0, 4.0));
        let (tx, ty) = b.stair_top;
        let (x0, y0) = (b.rect.x, b.rect.y);
        let (x1, y1) = (b.rect.right(), b.rect.bottom());
        match b.stair_axis {
            Axis::Horizontal => {
                assert!(tx >= x0 - 1e-9 && tx + b.stair_run <= x1 + 1e-9, "stair run leaves the rect in x");
                assert!(ty >= y0 - 1e-9 && ty <= y1 + 1e-9, "stair top outside the rect in y");
            }
            Axis::Vertical => {
                assert!(ty >= y0 - 1e-9 && ty + b.stair_run <= y1 + 1e-9, "stair run leaves the rect in y");
                assert!(tx >= x0 - 1e-9 && tx <= x1 + 1e-9, "stair top outside the rect in x");
            }
        }
        assert!(b.stair_run > 2.0, "stair run {} too short to descend", b.stair_run);
    }

    #[test]
    fn the_stair_starts_at_the_end_nearest_the_hall() {
        // Hall at the west end: the stair starts at the west edge.
        let west = Basement::from_footprint(&footprint(), 42, 1, (1.0, 4.0));
        assert!(
            (west.stair_top.0 - 0.0).abs() < 1e-9,
            "hall at west end, stair should start at x=0, got {}",
            west.stair_top.0
        );
        // Hall at the east end: the stair starts at the east edge
        // and runs west (its top is run-length in from the edge).
        let east = Basement::from_footprint(&footprint(), 42, 1, (11.0, 4.0));
        assert!(
            (east.stair_top.0 + east.stair_run - 12.0).abs() < 1e-9,
            "hall at east end, stair should end at x=12, got top {} run {}",
            east.stair_top.0,
            east.stair_run
        );
    }
}
