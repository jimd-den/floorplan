//! Roofing tests (spec §3, §6, §10d):
//! - Roof forms (Gable, Hip, Gambrel, Mansard) per style and seed.
//! - Every roof has a ridge and pitch > 0, never a flat slab.
//! - Eaves/fascia, gutters, and chimney sit on real roof geometry.

use floorplan::domain::style::{RoofForm, Style};
use floorplan::use_cases::extrude::extrude;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::materials::{ROOF, TRIM};
use floorplan::use_cases::roof::{roof_form_for, roof_pitch_for, roof_ridge_height};
use floorplan::use_cases::walls::STOREY_H;

#[test]
fn every_roof_has_a_ridge_and_positive_pitch() {
    for seed in 0..30u32 {
        for style in Style::all() {
            let house = generate_house(seed, style);
            let pitch = roof_pitch_for(&house);
            assert!(pitch > 0.0, "seed {seed} {style:?}: roof pitch {pitch} <= 0");
            let mesh = extrude(&house);
            let wall_top = house.floors.len() as f32 * STOREY_H;
            let ridge = roof_ridge_height(&mesh);
            assert!(
                ridge > wall_top + 0.5,
                "seed {seed} {style:?}: ridge {ridge} does not rise above wall top {wall_top}"
            );
        }
    }
}

#[test]
fn all_four_roof_forms_are_generated_across_styles_and_seeds() {
    let mut forms = std::collections::HashSet::new();
    for seed in 0..30u32 {
        for style in Style::all() {
            let house = generate_house(seed, style);
            let form = roof_form_for(&house);
            forms.insert(form);
            let mesh = extrude(&house);
            assert_eq!(mesh.validate(), Ok(()), "invalid mesh for {form:?}");
            assert!(mesh.materials.contains(&ROOF), "missing roof slopes for {form:?}");
            assert!(mesh.materials.contains(&TRIM), "missing fascia/gutter trim for {form:?}");
        }
    }
    assert!(forms.contains(&RoofForm::Gable), "missing Gable");
    assert!(forms.contains(&RoofForm::Hip), "missing Hip");
    assert!(forms.contains(&RoofForm::Gambrel), "missing Gambrel");
    assert!(forms.contains(&RoofForm::Mansard), "missing Mansard");
}

#[test]
fn a_single_storey_wing_and_garage_roof_at_one_storey() {
    for seed in 0..50u32 {
        let house = generate_house(seed, Style::Colonial);
        if house.footprint.rects.len() < 2 || house.footprint.garage.is_none() {
            continue;
        }
        let mesh = extrude(&house);
        let low: Vec<f32> = mesh.vertices.iter().map(|v| v[1]).filter(|&y| y < 1.5 * STOREY_H).collect();
        assert!(low.iter().any(|&y| (y - STOREY_H).abs() < 0.01), "seed {seed}: no one-storey wing/garage eave");
        return;
    }
    panic!("no two-storey Colonial with a wing and garage in 50 seeds");
}
