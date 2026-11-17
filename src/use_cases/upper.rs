//! Upper-floor program assignment: the hall is the leaf nearest
//! the centre, the study the farthest from it, bedrooms and baths
//! by the shared bed/bath rule. See `programs` for the full list.

use crate::domain::geometry::Rect;
use crate::domain::model::Program;
use crate::use_cases::core_rooms::bedroom_reserve_holds;
use crate::use_cases::placement::{dist, farthest, take};
use crate::use_cases::repair::{assign_bed_bath, pinned_bath_count};

pub(crate) fn assign_upper(leaves: &[Rect], pins: &[(usize, Program)]) -> Vec<Program> {
    let n = leaves.len();
    let mut out = vec![Program::Study; n];
    let mut remaining: Vec<usize> = (0..n).collect();
    for &(i, p) in pins {
        take(&mut remaining, &mut out, i, p);
    }
    if remaining.is_empty() {
        return out;
    }
    let cx = (leaves.iter().map(|r| r.x).fold(f64::MAX, f64::min)
        + leaves.iter().map(|r| r.right()).fold(f64::MIN, f64::max))
        / 2.0;
    let cy = (leaves.iter().map(|r| r.y).fold(f64::MAX, f64::min)
        + leaves.iter().map(|r| r.bottom()).fold(f64::MIN, f64::max))
        / 2.0;
    // ADR 0014: a pinned hall is the anchor — the nearest-centre
    // rule must not assign a second hall, and the study is
    // measured from the pinned leaf.
    let hall = match pins.iter().find(|&&(_, p)| p == Program::Hall) {
        Some(&(i, _)) => i,
        None => {
            let h = *remaining
                .iter()
                .min_by(|a, b| {
                    dist((cx, cy), &leaves[**a])
                        .partial_cmp(&dist((cx, cy), &leaves[**b]))
                        .unwrap()
                })
                .unwrap();
            take(&mut remaining, &mut out, h, Program::Hall);
            h
        }
    };
    if remaining.len() >= 4 {
        // A study must fit the cap, or be a leaf the strip cut
        // can carve to it, and it must meet the habitable floor:
        // a 6.0 sq m alcove leaf is a bath, not a study, and the
        // choke turns a sub-floor study into laundry, wasting
        // the leaf a bath needs. Prefer the farthest fitting
        // leaf; a leaf too big for a study, and too squat to
        // cut, is a bedroom, not a study — the bed/bath rule
        // promotes it. (Ground does the same; the upper floor
        // used to take the farthest leaf whatever its size,
        // which made over-cap studies the re-tiling loop could
        // not fix.)
        let study_cap = crate::use_cases::caps::cap_for(Program::Study).unwrap_or(f64::INFINITY);
        let fitting: Vec<usize> = remaining
            .iter()
            .copied()
            .filter(|&i| {
                leaves[i].area() >= crate::domain::room_size::HABITABLE_MIN_AREA - 1e-9
                    && (leaves[i].area() <= study_cap + 1e-9
                        || crate::use_cases::cap_cut::is_cuttable(&leaves[i], study_cap))
            })
            .collect();
        // ADR 0015 (9): no fitting leaf means no Study — the
        // farthest leaf is an over-cap, uncuttable leaf the
        // bed/bath rule promotes to a Bedroom instead.
        if !fitting.is_empty() {
            let study = farthest(leaves, hall, &fitting);
            if bedroom_reserve_holds(leaves, &remaining, study) {
                take(&mut remaining, &mut out, study, Program::Study);
            }
        }
    }
    assign_bed_bath(leaves, &remaining, hall, &mut out, pinned_bath_count(pins));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::programs::assign_programs;

    #[test]
    fn upper_floor_has_a_hall_bedrooms_and_a_bath() {
        let leaves = vec![
            Rect::new(0.0, 0.0, 6.0, 5.0),
            Rect::new(6.0, 0.0, 6.0, 5.0),
            Rect::new(0.0, 5.0, 6.0, 5.0),
            Rect::new(6.0, 5.0, 6.0, 5.0),
        ];
        let got = assign_programs(&leaves, 1, true);
        assert!(got.contains(&Program::Hall));
        assert!(got.contains(&Program::Bedroom));
        assert!(got.contains(&Program::Bath));
        assert!(!got.contains(&Program::Entry));
        assert!(!got.contains(&Program::Kitchen));
    }

    #[test]
    fn a_pinned_hall_is_the_only_hall_and_the_anchor() {
        // ADR 0014: when the re-tiling alcove pins a hall leaf,
        // the nearest-centre rule must not assign a second hall,
        // and the study (farthest from the hall) is measured
        // from the pinned leaf.
        use crate::use_cases::programs::assign_programs_pinned;
        let leaves = vec![
            Rect::new(0.0, 0.0, 4.0, 5.0),
            Rect::new(4.0, 0.0, 4.0, 5.0),
            Rect::new(8.0, 0.0, 4.0, 5.0),
            Rect::new(0.0, 5.0, 4.0, 5.0),
            Rect::new(4.0, 5.0, 4.0, 5.0),
            Rect::new(8.0, 5.0, 4.0, 5.0),
        ];
        let got = assign_programs_pinned(&leaves, 1, true, &[(0, Program::Hall)]);
        assert_eq!(got[0], Program::Hall);
        assert_eq!(got.iter().filter(|p| **p == Program::Hall).count(), 1);
        assert_eq!(got[5], Program::Study, "the study is farthest from the pinned hall");
    }

    #[test]
    fn no_study_is_assigned_when_no_leaf_fits_it() {
        // Regression (seed 97 Colonial floor 1): every
        // habitable leaf is over the 12 sq m Study cap and
        // uncuttable (the 4.9 x 6.9 m leaf keeps 12.005 sq m),
        // so the fitting set is empty and the farthest-leaf
        // fallback put the Study on a 5.9 x 6.9 m (40.7 sq m)
        // leaf. The Study is skipped; the bed/bath rule
        // promotes the big leaves to Bedrooms instead.
        use crate::use_cases::programs::assign_programs_pinned;
        let leaves = vec![
            Rect::new(0.0, 0.0, 2.4, 2.5),
            Rect::new(2.4, 0.0, 2.4, 2.5),
            Rect::new(4.8, 0.0, 2.4, 2.5),
            Rect::new(7.2, 0.0, 10.8, 5.4),
            Rect::new(7.2, 5.4, 4.9, 6.9),
            Rect::new(12.1, 5.4, 5.9, 6.9),
        ];
        let got = assign_programs_pinned(&leaves, 1, true, &[(0, Program::Hall)]);
        for (i, p) in got.iter().enumerate() {
            if *p == Program::Study {
                assert!(
                    leaves[i].area() <= 12.0 + 1e-9,
                    "leaf {i} is a {:.1} sq m study",
                    leaves[i].area()
                );
            }
        }
    }

    #[test]
    fn the_study_takes_a_fitting_leaf_not_the_farthest_over_cap_one() {
        // Regression (seed 1 Colonial floor 1): the farthest leaf
        // from the hall was 15.6 sq m, too squat to strip-cut, and
        // became an over-cap Study. The Study takes the farthest
        // leaf that fits the cap (or can be cut to it); the
        // over-cap leaf is left for the bed/bath rule, which
        // promotes it to a Bedroom.
        let leaves = vec![
            Rect::new(0.0, 0.0, 2.4, 2.5),
            Rect::new(0.0, 2.5, 2.4, 4.7),
            Rect::new(2.4, 0.0, 2.4, 2.5),
            Rect::new(2.4, 2.5, 2.4, 4.7),
            Rect::new(4.8, 0.0, 2.4, 2.5),
            Rect::new(4.8, 2.5, 2.4, 4.7),
            Rect::new(7.2, 0.0, 2.4, 2.5),
            Rect::new(7.2, 2.5, 2.4, 4.7),
            Rect::new(9.6, 0.0, 4.0, 3.9),
            Rect::new(9.6, 3.9, 4.0, 3.3),
        ];
        let got = assign_programs(&leaves, 1, true);
        for (i, p) in got.iter().enumerate() {
            if *p == Program::Study {
                assert!(
                    leaves[i].area() <= 12.0 + 1e-9,
                    "leaf {i} is a {:.1} sq m study",
                    leaves[i].area()
                );
                assert!(
                    leaves[i].area() >= 6.5 - 1e-9,
                    "leaf {i} is a sub-floor {:.1} sq m study",
                    leaves[i].area()
                );
            }
        }
    }
}
