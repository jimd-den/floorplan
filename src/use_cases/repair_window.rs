//! Window repair: keep habitable rooms off windowless leaves.
//! Swaps programs only, never rooms. Split out of `repair` so
//! both files stay under the 200-line ceiling.

use crate::domain::geometry::Rect;
use crate::domain::model::{Program, Room};
use crate::use_cases::placement::windowless;

/// A swap is cap-safe when neither program lands on a leaf that
/// breaks its cap: a Bath needs its new leaf at most 6 sq m, a
/// Study at most 12. Hall and Laundry are uncapped.
fn swap_is_cap_safe(rooms: &[Room], i: usize, j: usize) -> bool {
    let bath_ok = rooms[j].program != Program::Bath || rooms[i].rect.area() <= 6.0 + 1e-9;
    let study_ok = rooms[i].program != Program::Study || rooms[j].rect.area() <= 12.0 + 1e-9;
    bath_ok && study_ok
}

/// After the garage is attached, a habitable room can lose its only
/// exterior edge. Swap its program with a bath, hall or laundry
/// that can still hold a window. Swaps programs only, never rooms.
/// Pinned room ids are never swapped: the alcove leaf keeps its
/// program. The swap is cap-safe, so it never creates an over-cap
/// bath or study. When no cap-safe partner exists, a windowless
/// Study or Dining is demoted to Laundry — a non-habitable program
/// that needs no window — rather than leaving a habitable room blind.
pub(crate) fn repair_interior_rooms(rooms: &mut [Room], pinned: &[u32]) {
    let rects: Vec<Rect> = rooms.iter().map(|r| r.rect).collect();
    for i in 0..rooms.len() {
        if pinned.contains(&rooms[i].id) {
            continue;
        }
        if rooms[i].program.is_habitable() && windowless(&rects, i) {
            let partner = (0..rooms.len()).find(|&j| {
                !pinned.contains(&rooms[j].id)
                    && !windowless(&rects, j)
                    && matches!(rooms[j].program, Program::Bath | Program::Hall | Program::Laundry)
                    && swap_is_cap_safe(rooms, i, j)
            });
            if let Some(j) = partner {
                let (a, b) = (rooms[i].program, rooms[j].program);
                rooms[i].program = b;
                rooms[j].program = a;
            } else if matches!(rooms[i].program, Program::Study | Program::Dining) {
                rooms[i].program = Program::Laundry;
            }
        }
    }
}

/// Habitable rooms need an exterior wall for a window. A leaf that
/// cannot hold a window swaps its program with a bath, hall or
/// laundry that can. Kitchen adjacency can lose in that rare case;
/// the window guarantee is the harder rule. Pinned leaves are
/// never swapped: the alcove leaf keeps its program.
pub(crate) fn repair_interior(leaves: &[Rect], out: &mut [Program], pinned: &[usize]) {
    for i in 0..leaves.len() {
        if pinned.contains(&i) {
            continue;
        }
        if out[i].is_habitable()
            && windowless(leaves, i)
            && let Some(j) = (0..leaves.len()).find(|&j| {
                !pinned.contains(&j)
                    && !windowless(leaves, j)
                    && matches!(out[j], Program::Bath | Program::Hall | Program::Laundry)
            })
        {
            out.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

#[test]
fn the_repair_never_swaps_a_pinned_leaf() {
    // A 3x3 grid of 4x3 leaves: the centre leaf is interior
    // (windowless). It is Living; leaf 0 is the only Bath on
    // an exterior wall, so without the pin the repair swaps
    // them — with the pin it must not.
    let leaves = vec![
        Rect::new(0.0, 0.0, 4.0, 3.0),
        Rect::new(4.0, 0.0, 4.0, 3.0),
        Rect::new(8.0, 0.0, 4.0, 3.0),
        Rect::new(0.0, 3.0, 4.0, 3.0),
        Rect::new(4.0, 3.0, 4.0, 3.0),
        Rect::new(8.0, 3.0, 4.0, 3.0),
        Rect::new(0.0, 6.0, 4.0, 3.0),
        Rect::new(4.0, 6.0, 4.0, 3.0),
        Rect::new(8.0, 6.0, 4.0, 3.0),
    ];
    let mut plain = vec![Program::Bath; 9];
    plain[4] = Program::Living;
    for i in [1, 2, 3, 5, 6, 7, 8] {
        plain[i] = Program::Bedroom;
    }
    let mut unpinned = plain.clone();
    repair_interior(&leaves, &mut unpinned, &[]);
    assert_eq!(unpinned[0], Program::Living, "without the pin the swap should happen");
    assert_eq!(unpinned[4], Program::Bath);
    let mut pinned = plain.clone();
    repair_interior(&leaves, &mut pinned, &[0]);
    assert_eq!(pinned[0], Program::Bath, "the pinned bath was swapped away");
    assert_eq!(pinned[4], Program::Living, "the windowless room should keep its program");
}
}
