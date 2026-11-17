//! roof_frame: derive the roof frame (ridge + rafters) from the
//! footprint and the roof form. The ridge is the frame's top member:
//! it runs along the main rect's long axis, centred on the short
//! axis, and the rafters are spaced along it. The ridge length
//! follows the roof form — a hip ridge is shorter than the long
//! side, so a full-length ridge would poke out of the roof at both
//! ends. The roof frame does not draw the roof; `roof.rs` does.

use crate::domain::framing::{RoofFrame, rafter_positions};
use crate::domain::geometry::{Rect, Segment};
use crate::domain::model::Footprint;
use crate::domain::style::RoofForm;

/// The roof frame for `footprint` under `form`: the ridge along
/// the main (largest) rect's long axis, centred, with rafters
/// spaced along it. The span is the rafter run (ridge to eave,
/// half the short side).
pub fn roof_frame(footprint: &Footprint, form: RoofForm) -> RoofFrame {
    let main = footprint
        .rects
        .iter()
        .max_by(|a, b| a.area().partial_cmp(&b.area()).unwrap())
        .copied()
        .unwrap_or(Rect::new(0.0, 0.0, 1.0, 1.0));
    let (long, short) = if main.w >= main.d { (main.w, main.d) } else { (main.d, main.w) };
    let ridge_len = ridge_length(long, short, form);
    if main.w >= main.d {
        let cx = main.x + main.w / 2.0;
        let ridge =
            Segment::new(cx - ridge_len / 2.0, main.y + main.d / 2.0, cx + ridge_len / 2.0, main.y + main.d / 2.0);
        RoofFrame { ridge, rafters: rafter_positions(ridge_len), span: main.d / 2.0 }
    } else {
        let cy = main.y + main.d / 2.0;
        let ridge =
            Segment::new(main.x + main.w / 2.0, cy - ridge_len / 2.0, main.x + main.w / 2.0, cy + ridge_len / 2.0);
        RoofFrame { ridge, rafters: rafter_positions(ridge_len), span: main.w / 2.0 }
    }
}

/// The ridge length for a roof of `form` on a rect with long side
/// `long` and short side `short`. Gable and gambrel ridges span the
/// full long side; a hip ridge is shortened by the short side (the
/// hip slopes meet the ridge at each end); a mansard ridge is the
/// flat deck's long edge, shortened by the curb inset on the eave
/// bounds (see `roof_detail::add_mansard`).
fn ridge_length(long: f64, short: f64, form: RoofForm) -> f64 {
    match form {
        RoofForm::Gable | RoofForm::Gambrel => long,
        RoofForm::Hip => (long - short).max(0.0),
        RoofForm::Mansard => {
            let eave = crate::use_cases::roof::EAVE as f64;
            let inset = ((short + 2.0 * eave) * 0.22).max(0.4);
            (long + 2.0 * eave - 2.0 * inset).max(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;

    fn main_rect(seed: u32, style: Style) -> Rect {
        let house = generate_house(seed, style);
        *house
            .footprint
            .rects
            .iter()
            .max_by(|a, b| a.area().partial_cmp(&b.area()).unwrap())
            .unwrap()
    }

    #[test]
    fn the_roof_frame_has_a_ridge_and_rafters() {
        let house = generate_house(42, Style::Ranch);
        let roof = roof_frame(&house.footprint, RoofForm::for_house(Style::Ranch, 42));
        assert!(roof.ridge.len() > 0.0, "no ridge");
        assert!(!roof.rafters.is_empty(), "no rafters");
        assert!(roof.span > 0.0, "no rafter span");
    }

    #[test]
    fn hip_roof_ridge_is_shorter_than_the_long_side() {
        // A hip roof's ridge runs between the two hip ends, so it
        // is long-minus-short, not the full long side — the full
        // length would poke out of the roof at both ends.
        let house = generate_house(42, Style::Modern);
        let roof = roof_frame(&house.footprint, RoofForm::Hip);
        let main = main_rect(42, Style::Modern);
        let (long, short) = if main.w >= main.d { (main.w, main.d) } else { (main.d, main.w) };
        let ridge_len = roof.ridge.len();
        assert!(
            (ridge_len - (long - short)).abs() < 1e-6,
            "hip ridge {ridge_len} should be long-minus-short {}",
            long - short
        );
    }

    #[test]
    fn gable_roof_ridge_spans_the_long_side() {
        // A gable ridge runs the full length between the gable ends.
        let house = generate_house(36, Style::Modern);
        let roof = roof_frame(&house.footprint, RoofForm::Gable);
        let main = main_rect(36, Style::Modern);
        let long = if main.w >= main.d { main.w } else { main.d };
        let ridge_len = roof.ridge.len();
        assert!(
            (ridge_len - long).abs() < 1e-6,
            "gable ridge {ridge_len} should span the long side {long}"
        );
    }
}
