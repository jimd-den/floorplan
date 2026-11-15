//! Pitched roofs (spec §3, §6): gable, hip, gambrel, and mansard.
//!
//! Each footprint rectangle gets its own pitched roof at its own
//! storey height (main body at the top storey via `floor::upper_rects`,
//! single-storey wings and the garage at one storey), with a 0.4 m
//! eave overhang, fascia/gutters, and a chimney stack on the main
//! body. The porch gets its own roof in `use_cases::porch`.

use crate::domain::geometry::Rect;
use crate::domain::mesh::Mesh;
use crate::domain::model::House;
use crate::domain::style::RoofForm;
use crate::use_cases::floor::upper_rects;
use crate::use_cases::materials::{ROOF, SIDING};
use crate::use_cases::roof_detail::{add_gambrel, add_hip, add_mansard, add_roof_accessories};
use crate::use_cases::walls::STOREY_H;

/// Eave overhang past the walls, in metres.
pub const EAVE: f32 = 0.4;
/// Default roof rise per metre of half-span: a 45° pitch at 0.5.
pub const PITCH: f32 = 0.5;

/// The roof form selected for `house`.
pub fn roof_form_for(house: &House) -> RoofForm {
    RoofForm::for_house(house.style, house.seed)
}

/// The roof pitch (rise per metre of half-span) for `house`:
/// always `> 0.0`, varying deterministically by seed and style.
pub fn roof_pitch_for(house: &House) -> f32 {
    let step = (house.seed % 4) as f32 * 0.05;
    0.42 + step
}

/// The highest roof vertex height in `mesh`.
pub fn roof_ridge_height(mesh: &Mesh) -> f32 {
    mesh.faces
        .iter()
        .zip(mesh.materials.iter())
        .filter(|(_, m)| **m == ROOF)
        .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
        .fold(0.0f32, f32::max)
}

/// Adds the pitched roof and eave/chimney accessories to `mesh`.
pub fn add_roof(mesh: &mut Mesh, house: &House) {
    add_roof_with_accessories(mesh, house, true);
}

/// Adds the pitched roof, with optional fascia/chimney accessories
/// (omitted at coarse LOD or in unit tests of bare slope counts).
pub fn add_roof_with_accessories(mesh: &mut Mesh, house: &House, accessories: bool) {
    let top = house.floors.len() as f32 * STOREY_H;
    let upper = upper_rects(&house.footprint, house.style);
    let form = roof_form_for(house);
    let pitch = roof_pitch_for(house);
    for (idx, r) in house.footprint.rects.iter().enumerate() {
        let wall_top = if upper.contains(r) { top } else { STOREY_H };
        if idx == 0 {
            add_form(mesh, *r, wall_top, form, pitch);
            if accessories {
                add_roof_accessories(mesh, house, *r, wall_top, pitch);
            }
        } else {
            add_gable(mesh, *r, wall_top, pitch);
        }
    }
    if let Some(garage) = house.footprint.garage {
        add_gable(mesh, garage, STOREY_H, pitch);
    }
}

fn add_form(mesh: &mut Mesh, r: Rect, wall_top: f32, form: RoofForm, pitch: f32) {
    match form {
        RoofForm::Gable => add_gable(mesh, r, wall_top, pitch),
        RoofForm::Hip => add_hip(mesh, r, wall_top, pitch),
        RoofForm::Gambrel => add_gambrel(mesh, r, wall_top, pitch),
        RoofForm::Mansard => add_mansard(mesh, r, wall_top, pitch),
    }
}

/// One gable over a rectangle: two slope quads and two gable
/// triangles, all `EAVE` past the rectangle.
pub(crate) fn add_gable(mesh: &mut Mesh, r: Rect, wall_top: f32, pitch: f32) {
    let (x0, z0, x1, z1) = (
        r.x as f32 - EAVE,
        r.y as f32 - EAVE,
        r.right() as f32 + EAVE,
        r.bottom() as f32 + EAVE,
    );
    if (x1 - x0) >= (z1 - z0) {
        let ridge_z = (z0 + z1) / 2.0;
        let ridge_y = wall_top + (z1 - z0) / 2.0 * pitch;
        mesh.add_quad([x0, wall_top, z1], [x1, wall_top, z1], [x1, ridge_y, ridge_z], [x0, ridge_y, ridge_z], ROOF);
        mesh.add_quad([x1, wall_top, z0], [x0, wall_top, z0], [x0, ridge_y, ridge_z], [x1, ridge_y, ridge_z], ROOF);
        mesh.add_triangle([x0, wall_top, z0], [x0, wall_top, z1], [x0, ridge_y, ridge_z], SIDING);
        mesh.add_triangle([x1, wall_top, z1], [x1, wall_top, z0], [x1, ridge_y, ridge_z], SIDING);
    } else {
        // Ridge along z. The vertex order is reversed relative to
        // the branch above: traversed the same way, these quads and
        // gables wind inward (normals point down into the house),
        // and a front-face-only renderer culls the whole roof.
        let ridge_x = (x0 + x1) / 2.0;
        let ridge_y = wall_top + (x1 - x0) / 2.0 * pitch;
        mesh.add_quad([x0, wall_top, z1], [ridge_x, ridge_y, z1], [ridge_x, ridge_y, z0], [x0, wall_top, z0], ROOF);
        mesh.add_quad([x1, wall_top, z0], [ridge_x, ridge_y, z0], [ridge_x, ridge_y, z1], [x1, wall_top, z1], ROOF);
        mesh.add_triangle([x0, wall_top, z0], [ridge_x, ridge_y, z0], [x1, wall_top, z0], SIDING);
        mesh.add_triangle([x1, wall_top, z1], [ridge_x, ridge_y, z1], [x0, wall_top, z1], SIDING);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::mesh::Mesh;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;
    use crate::use_cases::materials::material_table;

    fn roofed(seed: u32, style: Style) -> (House, Mesh) {
        let house = generate_house(seed, style);
        let mut mesh = Mesh::new(material_table());
        add_roof_with_accessories(&mut mesh, &house, false);
        (house, mesh)
    }

    fn count(mesh: &Mesh, material: u32) -> usize {
        mesh.materials.iter().filter(|m| **m == material).count()
    }

    #[test]
    fn a_rectangle_gable_is_two_slopes_and_two_gables() {
        let (_, mesh) = roofed(0, Style::Townhouse);
        assert_eq!(count(&mesh, ROOF), 4, "two slopes as quads");
        assert_eq!(count(&mesh, SIDING), 2, "two gable triangles");
        assert_eq!(mesh.validate(), Ok(()));
    }

    #[test]
    fn each_footprint_rectangle_gets_its_own_gable() {
        for seed in (0..50u32).step_by(12) {
            for style in [Style::Ranch, Style::Modern, Style::Craftsman] {
                let house = generate_house(seed, style);
                if house.footprint.rects.len() < 2 || roof_form_for(&house) != RoofForm::Gable {
                    continue;
                }
                let mut mesh = Mesh::new(material_table());
                add_roof_with_accessories(&mut mesh, &house, false);
                let rects = house.footprint.rects.len() + house.footprint.garage.is_some() as usize;
                assert_eq!(count(&mesh, ROOF), 4 * rects);
                assert_eq!(count(&mesh, SIDING), 2 * rects);
                return;
            }
        }
        panic!("no multi-rectangle gable footprint in 50 seeds");
    }

    #[test]
    fn a_two_storey_main_body_roofs_at_the_top_storey() {
        let (house, mesh) = roofed(7, Style::Colonial);
        assert_eq!(house.floors.len(), 2);
        let roof_top = mesh.vertices.iter().map(|v| v[1]).fold(0.0f32, f32::max);
        assert!(roof_top > 2.0 * STOREY_H, "the main roof does not rise above the top storey");
    }

    #[test]
    fn the_roof_is_deterministic() {
        let (_, a) = roofed(42, Style::Modern);
        let (_, b) = roofed(42, Style::Modern);
        assert_eq!(a.vertices, b.vertices);
    }
}
