//! Upper-floor program assignment: the hall is the leaf nearest
//! the centre, the study the farthest from it, bedrooms and baths
//! by the shared bed/bath rule. See `programs` for the full list.

use crate::domain::geometry::Rect;
use crate::domain::model::Program;
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
    let hall = *remaining
        .iter()
        .min_by(|a, b| {
            dist((cx, cy), &leaves[**a])
                .partial_cmp(&dist((cx, cy), &leaves[**b]))
                .unwrap()
        })
        .unwrap();
    take(&mut remaining, &mut out, hall, Program::Hall);
    if remaining.len() >= 4 {
        let study = farthest(leaves, hall, &remaining);
        take(&mut remaining, &mut out, study, Program::Study);
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
}
