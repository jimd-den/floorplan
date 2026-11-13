//! Ground-floor program assignment. Rules, not dice: entry is a
//! front leaf, living the largest, kitchen next to living, dining
//! next to kitchen or living, the stair hall an interior leaf next
//! to living, bedrooms the farthest large leaves, baths the small
//! leaves next to bedrooms. See `programs` for the full rule list.

use crate::domain::geometry::Rect;
use crate::domain::model::Program;
use crate::use_cases::placement::{
    adjacent, adjacent_or_nearest, adjacent_to_either_set, farthest, farthest_exterior,
    front_leaf, largest, largest_exterior, smallest, smallest_adjacent_or_nearest,
    smallest_interior, take,
};
use crate::use_cases::repair::{assign_bed_bath, pinned_bath_count};

pub(crate) fn assign_ground(
    leaves: &[Rect],
    two_storey: bool,
    pins: &[(usize, Program)],
) -> Vec<Program> {
    let n = leaves.len();
    let mut out = vec![Program::Study; n];
    let mut remaining: Vec<usize> = (0..n).collect();
    for &(i, p) in pins {
        take(&mut remaining, &mut out, i, p);
    }
    if remaining.is_empty() {
        return out;
    }
    if n >= 5 {
        let entry = front_leaf(leaves, &remaining);
        take(&mut remaining, &mut out, entry, Program::Entry);
    }
    // Living wants daylight: the largest exterior leaf. The
    // largest leaf can sit interior (an alcove strip can leave a
    // big central leaf); a windowless living room is worse than a
    // smaller one, and the interior leaf becomes laundry.
    let living = largest_exterior(leaves, &remaining).unwrap_or_else(|| largest(leaves, &remaining));
    take(&mut remaining, &mut out, living, Program::Living);
    // Kitchen wants daylight too: the largest exterior leaf next
    // to living, falling back to the nearest when no neighbour
    // can hold a window.
    let near = adjacent(leaves, living, &remaining);
    let kitchen = largest_exterior(leaves, &near)
        .unwrap_or_else(|| adjacent_or_nearest(leaves, living, &remaining));
    take(&mut remaining, &mut out, kitchen, Program::Kitchen);
    if two_storey && !remaining.is_empty() {
        // The hall is circulation: it takes an interior leaf when
        // one sits next to living, leaving daylight for the rooms.
        let near = adjacent(leaves, living, &remaining);
        let hall = smallest_interior(leaves, &near)
            .unwrap_or_else(|| smallest_adjacent_or_nearest(leaves, living, &remaining));
        take(&mut remaining, &mut out, hall, Program::Hall);
    }
    if remaining.len() >= 4 {
        // Dining wants daylight: the largest exterior leaf next to
        // the kitchen or living, falling back to the largest next to
        // them — or the nearest to the kitchen when nothing is next
        // to either (another footprint rect).
        let near = adjacent_to_either_set(leaves, kitchen, living, &remaining);
        let dining = if near.is_empty() {
            adjacent_or_nearest(leaves, kitchen, &remaining)
        } else {
            largest_exterior(leaves, &near).unwrap_or_else(|| largest(leaves, &near))
        };
        take(&mut remaining, &mut out, dining, Program::Dining);
    }
    if remaining.len() >= 6 {
        // A study is far from living — but it must fit the cap,
        // or be a leaf the strip cut can carve to it. Prefer the
        // farthest exterior leaf that fits; a leaf too big for a
        // study, and too squat to cut, is a bedroom, not a study.
        let study_cap = crate::use_cases::caps::cap_for(Program::Study).unwrap_or(f64::INFINITY);
        let fitting: Vec<usize> = remaining
            .iter()
            .copied()
            .filter(|&i| {
                leaves[i].area() <= study_cap + 1e-9
                    || crate::use_cases::cap_cut::is_cuttable(&leaves[i], study_cap)
            })
            .collect();
        let study = if fitting.is_empty() {
            farthest_exterior(leaves, living, &remaining)
                .unwrap_or_else(|| farthest(leaves, living, &remaining))
        } else {
            farthest_exterior(leaves, living, &fitting)
                .unwrap_or_else(|| farthest(leaves, living, &fitting))
        };
        take(&mut remaining, &mut out, study, Program::Study);
    }
    if remaining.len() >= 5 {
        // Laundry needs no window: it takes an interior leaf first.
        let laundry = smallest_interior(leaves, &remaining)
            .unwrap_or_else(|| smallest(leaves, &remaining));
        take(&mut remaining, &mut out, laundry, Program::Laundry);
    }
    assign_bed_bath(leaves, &remaining, living, &mut out, pinned_bath_count(pins));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::programs::assign_programs;

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
    fn six_leaves_get_the_expected_programs() {
        let got = assign_programs(&six_leaves(), 0, false);
        assert_eq!(
            got,
            vec![
                Program::Living,
                Program::Kitchen,
                Program::Bedroom,
                Program::Bath,
                Program::Entry,
                Program::Bedroom,
            ]
        );
    }

    #[test]
    fn kitchen_is_adjacent_to_living() {
        use crate::domain::geometry::shared_edge;
        let leaves = six_leaves();
        let got = assign_programs(&leaves, 0, false);
        let living = got.iter().position(|p| *p == Program::Living).unwrap();
        let kitchen = got.iter().position(|p| *p == Program::Kitchen).unwrap();
        assert!(shared_edge(&leaves[living], &leaves[kitchen]).is_some());
    }

    #[test]
    fn every_house_has_a_bedroom_and_a_bath() {
        use crate::frameworks_drivers::rng::Rng;
        use crate::use_cases::bsp::subdivide;
        for seed in 0..100 {
            let mut rng = Rng::new(seed);
            let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 14.0, 10.0), 6);
            let got = assign_programs(&sub.leaves, 0, false);
            assert!(got.contains(&Program::Bedroom), "seed {seed}");
            assert!(got.contains(&Program::Bath), "seed {seed}");
        }
    }

    #[test]
    fn at_least_one_bath_per_two_bedrooms() {
        use crate::frameworks_drivers::rng::Rng;
        use crate::use_cases::bsp::subdivide;
        for seed in 0..100 {
            let mut rng = Rng::new(seed);
            let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 16.0, 11.0), 8);
            let got = assign_programs(&sub.leaves, 0, false);
            let beds = got.iter().filter(|p| **p == Program::Bedroom).count();
            let baths = got.iter().filter(|p| **p == Program::Bath).count();
            assert!(baths * 2 >= beds, "seed {seed}: {beds} beds, {baths} baths");
        }
    }
}
