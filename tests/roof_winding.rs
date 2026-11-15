//! Regression: pitched-roof triangles must face outward, for
//! every roof form and both footprint orientations.
//!
//! The web shell renders front faces only. The gable and hip
//! branches whose ridge runs along z (footprint deeper than wide)
//! wound their slopes and end faces inward, so a front-face-only
//! renderer culled those roofs and showed the flat ceiling slab
//! underneath — the "missing roofs" report. The software renderer
//! never culls, which is why this shipped unseen, same as
//! `mesh_winding.rs`.

use floorplan::domain::mesh::Mesh;
use floorplan::domain::style::{RoofForm, Style};
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::materials::{ROOF, SIDING, material_table};
use floorplan::use_cases::roof::{add_roof_with_accessories, roof_form_for};

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A bare roof (no fascia/chimney boxes) over a single-rectangle,
/// garage-less house, so one interior point serves every face.
fn bare_single_rect_roof(seed: u32, style: Style) -> Option<(RoofForm, Mesh)> {
    let house = generate_house(seed, style);
    if house.footprint.rects.len() != 1 || house.footprint.garage.is_some() {
        return None;
    }
    let mut mesh = Mesh::new(material_table());
    add_roof_with_accessories(&mut mesh, &house, false);
    Some((roof_form_for(&house), mesh))
}

fn bounds(mesh: &Mesh) -> ([f32; 3], [f32; 3]) {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in &mesh.vertices {
        for i in 0..3 {
            lo[i] = lo[i].min(v[i]);
            hi[i] = hi[i].max(v[i]);
        }
    }
    (lo, hi)
}

fn assert_faces_outward(seed: u32, style: Style, mesh: &Mesh) {
    let (lo, hi) = bounds(mesh);
    // Inside the roof prism: footprint centre, just above the eave.
    let inside = [(lo[0] + hi[0]) / 2.0, lo[1] + 0.05, (lo[2] + hi[2]) / 2.0];
    for (i, face) in mesh.faces.iter().enumerate() {
        let mat = mesh.materials[i];
        assert!(mat == ROOF || mat == SIDING, "unexpected roof material {mat}");
        let a = mesh.vertices[face[0] as usize];
        let b = mesh.vertices[face[1] as usize];
        let c = mesh.vertices[face[2] as usize];
        let normal = cross(sub(b, a), sub(c, a));
        let centroid = [
            (a[0] + b[0] + c[0]) / 3.0,
            (a[1] + b[1] + c[1]) / 3.0,
            (a[2] + b[2] + c[2]) / 3.0,
        ];
        assert!(
            dot(normal, sub(centroid, inside)) > 0.0,
            "seed {seed} {style:?}: face {i} points inward: normal {normal:?}"
        );
        if mat == ROOF {
            // No roof face points downward. (Mansard crown ends
            // are vertical ROOF faces with a horizontal normal —
            // those are fine; a negative y is the bug signature.)
            let len = (dot(normal, normal)).sqrt();
            assert!(
                normal[1] / len > -1e-4,
                "seed {seed} {style:?}: slope face {i} points down: normal {normal:?}"
            );
        }
    }
}

#[test]
fn roof_faces_point_outward_for_every_form_and_orientation() {
    // Coverage: [form][deep?] — deep = ridge-along-z class (eaved
    // footprint deeper than wide). Only gable and hip branch on
    // orientation, so only they need both cells; gambrel and
    // mansard run one orientation-independent path (and a wide
    // single-rect townhouse mansard may not exist at all).
    let mut covered = [[false; 2]; 4];
    // Deep single-rect hips are rare (Colonial seed 357 is the
    // first); the sweep stops as soon as every cell is covered.
    for seed in 0..400u32 {
        for style in Style::all() {
            let Some((form, mesh)) = bare_single_rect_roof(seed, style) else {
                continue;
            };
            let (lo, hi) = bounds(&mesh);
            let deep = (hi[2] - lo[2]) > (hi[0] - lo[0]);
            covered[form as usize][deep as usize] = true;
            assert_faces_outward(seed, style, &mesh);
        }
        if covered[0].iter().all(|c| *c) && covered[1].iter().all(|c| *c)
            && covered[2].iter().any(|c| *c) && covered[3].iter().any(|c| *c)
        {
            break;
        }
    }
    for form in [RoofForm::Gable, RoofForm::Hip] {
        assert!(covered[form as usize][0], "no wide {form:?} roof exercised");
        assert!(covered[form as usize][1], "no deep {form:?} roof exercised");
    }
    for form in [RoofForm::Gambrel, RoofForm::Mansard] {
        assert!(covered[form as usize].iter().any(|c| *c), "no {form:?} roof exercised");
    }
}
