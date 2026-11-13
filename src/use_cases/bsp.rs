//! BSP subdivision: recursively splits a rectangle into rooms.
//! Splits alternate by aspect ratio, positions are quantized to 0.1 m,
//! and no leaf ever drops below `MIN_SIDE` — no sliver rooms.

use crate::domain::geometry::{Axis, Rect, Segment};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::footprint::MIN_SIDE;
use crate::use_cases::quant;

/// One cut: the axis, the coordinate, and the full cut segment.
#[derive(Debug, Clone, PartialEq)]
pub struct Split {
    pub axis: Axis,
    pub at: f64,
    pub line: Segment,
}

/// The result of subdividing a rectangle: the leaves and every cut.
#[derive(Debug, Clone, PartialEq)]
pub struct Subdivision {
    pub leaves: Vec<Rect>,
    pub splits: Vec<Split>,
}

/// Subdivides `rect` into up to `target` leaves. Stops early when no
/// leaf can be split without breaking the minimum size.
pub fn subdivide(rng: &mut Rng, rect: Rect, target: usize) -> Subdivision {
    let mut leaves = vec![rect];
    let mut splits = vec![];
    while leaves.len() < target {
        let Some(idx) = largest_splittable(&leaves) else { break };
        let leaf = leaves.remove(idx);
        let axis = pick_axis(rng, &leaf);
        let (a, b, split) = cut(rng, &leaf, axis);
        leaves.push(a);
        leaves.push(b);
        splits.push(split);
    }
    Subdivision { leaves, splits }
}

/// Index of the largest leaf that can still split, if any.
fn largest_splittable(leaves: &[Rect]) -> Option<usize> {
    leaves
        .iter()
        .enumerate()
        .filter(|(_, leaf)| can_split(leaf))
        .max_by(|(_, a), (_, b)| a.area().partial_cmp(&b.area()).unwrap())
        .map(|(i, _)| i)
}

/// A leaf can split when one side is long enough for two minimum rooms.
fn can_split(leaf: &Rect) -> bool {
    leaf.w >= 2.0 * MIN_SIDE || leaf.d >= 2.0 * MIN_SIDE
}

/// Splits along the long side when the leaf is clearly oblong, else
/// random — but only among axes the leaf can actually split on.
fn pick_axis(rng: &mut Rng, leaf: &Rect) -> Axis {
    let vertical = leaf.w >= 2.0 * MIN_SIDE;
    let horizontal = leaf.d >= 2.0 * MIN_SIDE;
    match (vertical, horizontal) {
        (true, true) => {
            let ratio = (leaf.w / leaf.d).max(leaf.d / leaf.w);
            if ratio > 1.4 {
                if leaf.w > leaf.d { Axis::Vertical } else { Axis::Horizontal }
            } else if rng.chance(0.5) {
                Axis::Vertical
            } else {
                Axis::Horizontal
            }
        }
        (true, false) => Axis::Vertical,
        (false, true) => Axis::Horizontal,
        (false, false) => Axis::Vertical,
    }
}

/// Cuts one leaf in two at a quantized position, recording the cut.
fn cut(rng: &mut Rng, leaf: &Rect, axis: Axis) -> (Rect, Rect, Split) {
    let (lo, hi) = match axis {
        Axis::Vertical => (leaf.x + MIN_SIDE, leaf.right() - MIN_SIDE),
        Axis::Horizontal => (leaf.y + MIN_SIDE, leaf.bottom() - MIN_SIDE),
    };
    let steps = ((hi - lo) * 10.0).round() as i64;
    let at = quant(lo + rng.range_i64(0, steps + 1) as f64 * 0.1);
    let (a, b) = leaf.split(axis, at);
    let line = match axis {
        Axis::Vertical => Segment::new(at, leaf.y, at, leaf.bottom()),
        Axis::Horizontal => Segment::new(leaf.x, at, leaf.right(), at),
    };
    (a, b, Split { axis, at, line })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
