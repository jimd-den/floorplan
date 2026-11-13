//! Splitting a room target across the footprint rects by area.

use crate::domain::geometry::Rect;

/// Splits `target` rooms across the footprint rects by area, each
/// rect getting at least one room when the target allows it.
pub(crate) fn distribute(target: usize, rects: &[Rect]) -> Vec<usize> {
    let n = rects.len();
    if target <= n {
        return (0..n).map(|i| if i < target { 1 } else { 0 }).collect();
    }
    let total: f64 = rects.iter().map(|r| r.area()).sum();
    let mut counts = vec![1; n];
    let mut remaining = target - n;
    while remaining > 0 {
        let best = (0..n)
            .max_by(|&a, &b| {
                let fa = target as f64 * rects[a].area() / total - counts[a] as f64;
                let fb = target as f64 * rects[b].area() / total - counts[b] as f64;
                fa.partial_cmp(&fb).unwrap()
            })
            .unwrap();
        counts[best] += 1;
        remaining -= 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooms_split_by_area_and_sum_to_the_target() {
        let rects = vec![Rect::new(0.0, 0.0, 10.0, 10.0), Rect::new(0.0, 0.0, 5.0, 5.0)];
        let counts = distribute(6, &rects);
        assert_eq!(counts.iter().sum::<usize>(), 6);
        assert!(counts.iter().all(|&c| c >= 1));
        assert!(counts[0] > counts[1], "the bigger rect gets more rooms");
    }

    #[test]
    fn a_target_below_the_rect_count_leaves_later_rects_empty() {
        let rects = vec![
            Rect::new(0.0, 0.0, 10.0, 10.0),
            Rect::new(0.0, 0.0, 5.0, 5.0),
            Rect::new(0.0, 0.0, 5.0, 5.0),
        ];
        assert_eq!(distribute(2, &rects), vec![1, 1, 0]);
    }
}
