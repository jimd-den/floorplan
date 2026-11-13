//! Assignment finishing: bedrooms and baths, and the repair
//! that keeps habitable rooms off windowless leaves.

use crate::domain::geometry::{Rect, shared_edge};
use crate::domain::model::Program;
use crate::use_cases::placement::{dist, windowless};

/// The pinned baths in a pin list: they count toward the
/// one-bath-per-two-bedrooms requirement.
pub(crate) fn pinned_bath_count(pins: &[(usize, Program)]) -> usize {
    pins.iter().filter(|&&(_, p)| p == Program::Bath).count()
}

/// Bedrooms are the farthest large leaves, exterior ones first;
/// baths are the rest, preferring small leaves next to a bedroom.
/// Only the required baths are assigned — one per two bedrooms,
/// minus the baths already pinned by the re-tiling alcove. The
/// rest stay Study, except a leftover too big for a Study, and
/// too squat to strip-cut, which is a Bedroom instead — but only
/// while enough bath candidates remain for the grown bedroom
/// count, so the last bath candidate is never promoted away.
pub(crate) fn assign_bed_bath(
    leaves: &[Rect],
    remaining: &[usize],
    anchor: usize,
    out: &mut [Program],
    pinned_baths: usize,
) {
    let r = remaining.len();
    if r == 0 {
        return;
    }
    if r == 1 {
        out[remaining[0]] = Program::Bedroom;
        return;
    }
    let bedrooms = (2 * r / 3).max(1).min(r - 1);
    let mut by_dist: Vec<usize> = remaining.to_vec();
    // Bedrooms are the farthest large leaves, exterior ones first —
    // and large before far: a big leaf left over becomes an
    // over-cap Study, while a bedroom is uncapped, so the largest
    // exterior leaves are bedrooms and the small ones are left
    // for Bath and Study.
    by_dist.sort_by(|a, b| {
        let ext_a = !windowless(leaves, *a);
        let ext_b = !windowless(leaves, *b);
        ext_b
            .cmp(&ext_a)
            .then(leaves[*b].area().partial_cmp(&leaves[*a].area()).unwrap())
            .then(
                dist(leaves[anchor].center(), &leaves[*b])
                    .partial_cmp(&dist(leaves[anchor].center(), &leaves[*a]))
                    .unwrap(),
            )
    });
    // Bedrooms never take a sub-cap leaf while a larger leaf can hold
    // one: the small leaves are the baths'. Bedrooms are exterior
    // leaves only — a bedroom needs a window. Falls back to all
    // leaves when the floor has no exterior leaf at all.
    let bath_cap = crate::use_cases::caps::cap_for(Program::Bath).unwrap_or(f64::INFINITY);
    let exterior: Vec<usize> =
        by_dist.iter().copied().filter(|&i| !windowless(leaves, i)).collect();
    let big: Vec<usize> =
        exterior.iter().copied().filter(|&i| leaves[i].area() > bath_cap).collect();
    let pool: Vec<usize> = if big.len() >= bedrooms {
        big
    } else if !exterior.is_empty() {
        exterior
    } else {
        by_dist.clone()
    };
    let mut beds: Vec<usize> = pool.iter().take(bedrooms).copied().collect();
    for &i in &beds {
        out[i] = Program::Bedroom;
    }
    let mut rest: Vec<usize> = remaining.iter().copied().filter(|i| !beds.contains(i)).collect();
    // A leftover too big for a Study, and too squat to strip-cut,
    // is a Bedroom, not a Study — but only while enough bath
    // candidates remain for the (grown) bedroom count. The guard
    // keeps the last bath candidate: promoting it would leave a
    // house with bedrooms and no bath.
    let study_cap = crate::use_cases::caps::cap_for(Program::Study).unwrap_or(f64::INFINITY);
    let mut unstudyable: Vec<usize> = rest
        .iter()
        .copied()
        .filter(|&i| {
            leaves[i].area() > study_cap + 1e-9
                && !crate::use_cases::cap_cut::is_cuttable(&leaves[i], study_cap)
                && !windowless(leaves, i)
        })
        .collect();
    unstudyable.sort_by(|a, b| leaves[*b].area().partial_cmp(&leaves[*a].area()).unwrap());
    for (promoted, &u) in unstudyable.iter().enumerate() {
        let beds_now = beds.len() + promoted + 1;
        let needed_now = beds_now.div_ceil(2).saturating_sub(pinned_baths);
        // The promotion must not create an over-cap bath: the
        // baths still needed must come from fixable leaves — a
        // leaf that fits the bath cap, or one a strip cut can
        // carve to it. Pinned baths already count.
        let promoted_set: Vec<usize> =
            unstudyable.iter().take(promoted + 1).copied().collect();
        let fixable = rest
            .iter()
            .filter(|i| !promoted_set.contains(i))
            .filter(|&&i| {
                leaves[i].area() <= bath_cap + 1e-9
                    || crate::use_cases::cap_cut::is_cuttable(&leaves[i], bath_cap)
            })
            .count();
        if fixable < needed_now {
            break;
        }
        beds.push(u);
        out[u] = Program::Bedroom;
    }
    rest.retain(|i| !beds.contains(i));
    // Baths prefer leaves the cap step can fix: a leaf that
    // already fits, or one a strip cut can carve to the cap. A
    // thin-rect leaf that fits neither is the last resort — it
    // stays over-cap, and the re-tiling loop pins another rect.
    rest.sort_by(|a, b| {
        let fix_a = leaves[*a].area() <= bath_cap + 1e-9
            || crate::use_cases::cap_cut::is_cuttable(&leaves[*a], bath_cap);
        let fix_b = leaves[*b].area() <= bath_cap + 1e-9
            || crate::use_cases::cap_cut::is_cuttable(&leaves[*b], bath_cap);
        let adj_a = beds.iter().any(|&bed| shared_edge(&leaves[*a], &leaves[bed]).is_some());
        let adj_b = beds.iter().any(|&bed| shared_edge(&leaves[*b], &leaves[bed]).is_some());
        fix_b
            .cmp(&fix_a)
            .then(adj_b.cmp(&adj_a))
            .then(leaves[*a].area().partial_cmp(&leaves[*b].area()).unwrap())
    });
    let needed = beds.len().div_ceil(2).saturating_sub(pinned_baths);
    for &i in rest.iter().take(needed) {
        out[i] = Program::Bath;
    }
    // A leftover on a windowless leaf cannot be a Study: it
    // becomes Laundry, which needs no window. The window repair
    // cannot fix it later — the only Bath may be the pinned
    // alcove leaf, which is never swapped.
    for &i in &rest {
        if out[i] == Program::Study && windowless(leaves, i) {
            out[i] = Program::Laundry;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baths_keep_the_sub_cap_leaf_when_a_bedroom_would_take_it() {
        // Living at (0,0); a 5 sq m leaf far from living and a 25 sq m
        // leaf next to it. The bedroom sort prefers far + exterior,
        // so without the reservation the bedroom takes the small leaf
        // and the bath gets the 25 sq m one.
        let leaves = vec![
            Rect::new(0.0, 0.0, 4.0, 4.0),
            Rect::new(10.0, 0.0, 2.5, 2.0),
            Rect::new(4.0, 0.0, 5.0, 5.0),
        ];
        let mut out = vec![Program::Study; 3];
        out[0] = Program::Living;
        assign_bed_bath(&leaves, &[1, 2], 0, &mut out, 0);
        assert_eq!(out[1], Program::Bath, "the 5 sq m leaf should be the bath");
        assert_eq!(out[2], Program::Bedroom);
    }
}
