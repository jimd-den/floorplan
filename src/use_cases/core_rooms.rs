//! Protecting the core rooms during assignment: the bedroom
//! reserve and the kitchen's habitable preference (ADR 0015).
//!
//! The assignment spends leaves on optional programs (dining,
//! study, laundry) before the bed/bath rule runs, so a scarce
//! floor can lose its bedroom — or its kitchen, when the only
//! neighbours of living are sub-floor strip leaves. These two
//! helpers keep a habitable leaf for the bedroom and send the
//! kitchen to a habitable leaf.

use crate::domain::geometry::Rect;
use crate::domain::room_size::meets_habitable_floor;
use crate::use_cases::placement::{
    adjacent, adjacent_or_nearest, largest, largest_exterior, nearest, windowless,
};

/// True when removing `idx` from `ids` still leaves a leaf
/// that can hold a bedroom with a window — the bedroom
/// reserve. Optional programs must not take the last such
/// leaf.
pub(crate) fn bedroom_reserve_holds(leaves: &[Rect], ids: &[usize], idx: usize) -> bool {
    ids.iter()
        .copied()
        .filter(|&i| i != idx)
        .any(|i| meets_habitable_floor(&leaves[i]) && !windowless(leaves, i))
}

/// The kitchen leaf: the largest windowed leaf next to living
/// that meets the habitable floor; when no neighbour is
/// habitable, the nearest habitable leaf instead of a
/// sub-floor neighbour the choke would turn into laundry;
/// when no leaf is habitable, the old rule (largest windowed
/// neighbour, else nearest).
pub(crate) fn pick_kitchen(leaves: &[Rect], living: usize, ids: &[usize]) -> usize {
    let near = adjacent(leaves, living, ids);
    let habitable_near: Vec<usize> = near
        .iter()
        .copied()
        .filter(|&i| meets_habitable_floor(&leaves[i]))
        .collect();
    if !habitable_near.is_empty() {
        return largest_exterior(leaves, &habitable_near)
            .unwrap_or_else(|| largest(leaves, &habitable_near));
    }
    let habitable: Vec<usize> = ids
        .iter()
        .copied()
        .filter(|&i| meets_habitable_floor(&leaves[i]))
        .collect();
    if habitable.is_empty() {
        largest_exterior(leaves, &near).unwrap_or_else(|| adjacent_or_nearest(leaves, living, ids))
    } else {
        nearest(leaves, living, &habitable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reserve_fails_on_the_last_habitable_leaf() {
        // One habitable windowed leaf (index 0) and two
        // sub-floor leaves: taking 0 leaves nothing for a
        // bedroom. With a second habitable leaf, the reserve
        // holds.
        let leaves = vec![
            Rect::new(0.0, 0.0, 4.0, 4.0),
            Rect::new(4.0, 0.0, 2.4, 2.5),
            Rect::new(0.0, 4.0, 2.4, 2.5),
        ];
        let ids = vec![0, 1, 2];
        assert!(!bedroom_reserve_holds(&leaves, &ids, 0));
        assert!(bedroom_reserve_holds(&leaves, &ids, 1));
        let leaves = vec![
            Rect::new(0.0, 0.0, 4.0, 4.0),
            Rect::new(4.0, 0.0, 4.0, 4.0),
            Rect::new(0.0, 4.0, 2.4, 2.5),
        ];
        assert!(bedroom_reserve_holds(&leaves, &ids, 0));
    }

    #[test]
    fn the_kitchen_prefers_a_habitable_leaf_over_a_sub_floor_neighbour() {
        // Living is leaf 0; its neighbours (1, 2) are 6.24 sq
        // m strip leaves. The habitable leaf 3 sits elsewhere.
        let leaves = vec![
            Rect::new(0.0, 0.0, 4.0, 4.0),
            Rect::new(4.0, 0.0, 2.6, 2.4),
            Rect::new(0.0, 4.0, 2.6, 2.4),
            Rect::new(6.6, 0.0, 4.0, 4.0),
        ];
        assert_eq!(pick_kitchen(&leaves, 0, &[1, 2, 3]), 3);
    }
}
