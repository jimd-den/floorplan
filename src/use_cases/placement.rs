//! Placement helpers: picking leaves by size, adjacency and
//! distance. Pure functions over the leaf list; the assignment rules
//! in `programs` are built from these.

use crate::domain::geometry::{Rect, shared_edge};
use crate::domain::model::Program;

/// Takes `idx` out of `remaining` and stamps its program.
pub(crate) fn take(remaining: &mut Vec<usize>, out: &mut [Program], idx: usize, program: Program) {
    remaining.retain(|&i| i != idx);
    out[idx] = program;
}

/// Index of the largest leaf among `ids` (first wins ties).
pub(crate) fn largest(leaves: &[Rect], ids: &[usize]) -> usize {
    let mut best = ids[0];
    for &i in &ids[1..] {
        if leaves[i].area() > leaves[best].area() {
            best = i;
        }
    }
    best
}

/// Index of the smallest leaf among `ids`.
pub(crate) fn smallest(leaves: &[Rect], ids: &[usize]) -> usize {
    *ids.iter()
        .min_by(|a, b| leaves[**a].area().partial_cmp(&leaves[**b].area()).unwrap())
        .unwrap()
}

/// Leaves among `ids` sharing a wall with `idx`.
pub(crate) fn adjacent(leaves: &[Rect], idx: usize, ids: &[usize]) -> Vec<usize> {
    ids.iter()
        .copied()
        .filter(|&i| i != idx && shared_edge(&leaves[idx], &leaves[i]).is_some())
        .collect()
}

/// Nearest leaf among `ids` to `idx`, by centroid distance.
pub(crate) fn nearest(leaves: &[Rect], idx: usize, ids: &[usize]) -> usize {
    let c = leaves[idx].center();
    *ids.iter()
        .min_by(|a, b| dist(c, &leaves[**a]).partial_cmp(&dist(c, &leaves[**b])).unwrap())
        .unwrap()
}

/// Farthest leaf among `ids` from `idx`, by centroid distance.
pub(crate) fn farthest(leaves: &[Rect], idx: usize, ids: &[usize]) -> usize {
    let c = leaves[idx].center();
    let mut best = ids[0];
    for &i in &ids[1..] {
        if dist(c, &leaves[i]) > dist(c, &leaves[best]) {
            best = i;
        }
    }
    best
}

pub(crate) fn dist(c: (f64, f64), r: &Rect) -> f64 {
    let o = r.center();
    ((c.0 - o.0).powi(2) + (c.1 - o.1).powi(2)).sqrt()
}

/// The front leaf: touches the southern edge, nearest the centre-x.
pub(crate) fn front_leaf(leaves: &[Rect], ids: &[usize]) -> usize {
    let max_bottom = ids.iter().map(|&i| leaves[i].bottom()).fold(f64::MIN, f64::max);
    let cx = (leaves.iter().map(|r| r.x).fold(f64::MAX, f64::min)
        + leaves.iter().map(|r| r.right()).fold(f64::MIN, f64::max))
        / 2.0;
    *ids.iter()
        .filter(|&&i| (leaves[i].bottom() - max_bottom).abs() < 1e-9)
        .min_by(|a, b| {
            (leaves[**a].center().0 - cx)
                .abs()
                .partial_cmp(&(leaves[**b].center().0 - cx).abs())
                .unwrap()
        })
        .unwrap()
}

/// Largest leaf adjacent to `idx`, else the nearest one.
pub(crate) fn adjacent_or_nearest(leaves: &[Rect], idx: usize, ids: &[usize]) -> usize {
    let adj = adjacent(leaves, idx, ids);
    if adj.is_empty() {
        nearest(leaves, idx, ids)
    } else {
        largest(leaves, &adj)
    }
}

/// Smallest leaf adjacent to `idx`, else the nearest one.
pub(crate) fn smallest_adjacent_or_nearest(leaves: &[Rect], idx: usize, ids: &[usize]) -> usize {
    let adj = adjacent(leaves, idx, ids);
    if adj.is_empty() {
        nearest(leaves, idx, ids)
    } else {
        smallest(leaves, &adj)
    }
}

/// The longest contiguous exterior stretch of a leaf's edges.
pub(crate) fn longest_exterior(leaves: &[Rect], idx: usize) -> f64 {
    let leaf = leaves[idx];
    leaf.edges()
        .iter()
        .map(|edge| {
            let covered: f64 = leaves
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != idx)
                .filter_map(|(_, other)| shared_edge(&leaf, other))
                .filter_map(|s| edge.overlap(&s))
                .map(|o| o.len())
                .sum();
            edge.len() - covered
        })
        .fold(0.0, f64::max)
}

/// True when a leaf cannot hold a window: its longest exterior
/// stretch is under 1.2 m (a 0.6 m window with 0.3 m margins).
pub(crate) fn windowless(leaves: &[Rect], idx: usize) -> bool {
    longest_exterior(leaves, idx) < 1.2
}

/// Largest leaf among `ids` that can hold a window, if any.
/// Index of the largest leaf among `ids` that can hold a window.
/// Ties keep the first leaf, matching `largest`.
pub(crate) fn largest_exterior(leaves: &[Rect], ids: &[usize]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for &i in ids {
        if windowless(leaves, i) {
            continue;
        }
        if best.is_none_or(|b| leaves[i].area() > leaves[b].area()) {
            best = Some(i);
        }
    }
    best
}

/// Farthest leaf from `idx` among `ids` that can hold a window.
pub(crate) fn farthest_exterior(leaves: &[Rect], idx: usize, ids: &[usize]) -> Option<usize> {
    let c = leaves[idx].center();
    ids.iter()
        .copied()
        .filter(|&i| !windowless(leaves, i))
        .max_by(|&a, &b| dist(c, &leaves[a]).partial_cmp(&dist(c, &leaves[b])).unwrap())
}

/// Smallest leaf among `ids` that cannot hold a window, if any.
pub(crate) fn smallest_interior(leaves: &[Rect], ids: &[usize]) -> Option<usize> {
    ids.iter().copied().filter(|&i| windowless(leaves, i)).min_by(|&a, &b| {
        leaves[a].area().partial_cmp(&leaves[b].area()).unwrap()
    })
}

/// Leaves adjacent to `a` or to `b`, among `ids`.
pub(crate) fn adjacent_to_either_set(
    leaves: &[Rect],
    a: usize,
    b: usize,
    ids: &[usize],
) -> Vec<usize> {
    let mut adj = adjacent(leaves, a, ids);
    let extra: Vec<usize> = adjacent(leaves, b, ids)
        .into_iter()
        .filter(|i| !adj.contains(i))
        .collect();
    adj.extend(extra);
    adj
}
