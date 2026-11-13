//! Program assignment: which room is the kitchen, which the bedrooms.
//! Rules, not dice — the assignment is a deterministic function of the
//! leaves, so the variety lives in the leaves themselves:
//!
//! * entry: a front leaf (ground floor, when there is room for one)
//! * living: the largest leaf
//! * kitchen: a leaf adjacent to living, else the nearest one
//! * dining: a leaf adjacent to kitchen or living, when space allows
//! * hall: the stair hall, forced on two-storey floors
//! * bedrooms: the largest leaves farthest from living
//! * baths: small leaves next to bedrooms, at least one per two bedrooms
//! * study / laundry: only when leaves remain after the required set
//!
//! The ground-floor rules live in `ground`, the upper-floor rules in
//! `upper`; this module is the facade over both, plus the pinning
//! the re-tiling alcove needs.

use crate::domain::geometry::Rect;
use crate::domain::model::Program;
use crate::use_cases::ground::assign_ground;
use crate::use_cases::repair_window::repair_interior;
use crate::use_cases::upper::assign_upper;

/// Assigns a program to every leaf. `level` 0 is the ground floor.
pub fn assign_programs(leaves: &[Rect], level: u8, two_storey: bool) -> Vec<Program> {
    assign_programs_pinned(leaves, level, two_storey, &[])
}

/// Assigns programs with pinned leaves: `pins` maps a leaf index
/// to the program that leaf must take. A pinned leaf is removed
/// from the pool before the rules run, so no rule can take it —
/// the re-tiling alcove pins its bath leaf this way. The repair
/// below never swaps a pinned leaf, and the pins are re-applied
/// at the end so the contract holds whatever the repair does.
pub fn assign_programs_pinned(
    leaves: &[Rect],
    level: u8,
    two_storey: bool,
    pins: &[(usize, Program)],
) -> Vec<Program> {
    if leaves.is_empty() {
        return vec![];
    }
    if leaves.len() == 1 {
        return vec![pins
            .first()
            .map(|&(_, p)| p)
            .unwrap_or(if level == 0 { Program::Living } else { Program::Hall })];
    }
    let mut out = if level == 0 {
        assign_ground(leaves, two_storey, pins)
    } else {
        assign_upper(leaves, pins)
    };
    let pinned: Vec<usize> = pins.iter().map(|&(i, _)| i).collect();
    repair_interior(leaves, &mut out, &pinned);
    for &(i, p) in pins {
        out[i] = p;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 12x9 house as six equal 4x4.5 leaves, 3 over 3.
    fn six_leaves() -> Vec<Rect> {
        vec![
            Rect::new(0.0, 0.0, 4.0, 4.5),
            Rect::new(4.0, 0.0, 4.0, 4.5),
            Rect::new(8.0, 0.0, 4.0, 4.5),
            Rect::new(0.0, 4.5, 4.0, 4.5),
            Rect::new(4.0, 4.5, 4.0, 4.5),
            Rect::new(8.0, 4.5, 4.0, 4.5),
        ]
    }

    #[test]
    fn a_pinned_leaf_keeps_its_program() {
        let leaves = six_leaves();
        let got = assign_programs_pinned(&leaves, 0, false, &[(0, Program::Bath)]);
        assert_eq!(got[0], Program::Bath, "the pin must survive the rules");
        assert!(got.contains(&Program::Living), "the floor still needs a living room");
        assert!(got.contains(&Program::Kitchen), "the floor still needs a kitchen");
    }

    #[test]
    fn a_pinned_bath_still_satisfies_the_bath_per_bedroom_rule() {
        use crate::frameworks_drivers::rng::Rng;
        use crate::use_cases::bsp::subdivide;
        for seed in 0..100 {
            let mut rng = Rng::new(seed);
            let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 16.0, 11.0), 8);
            let got = assign_programs_pinned(&sub.leaves, 0, false, &[(0, Program::Bath)]);
            let beds = got.iter().filter(|p| **p == Program::Bedroom).count();
            let baths = got.iter().filter(|p| **p == Program::Bath).count();
            assert!(baths * 2 >= beds, "seed {seed}: {beds} beds, {baths} baths");
        }
    }
}
