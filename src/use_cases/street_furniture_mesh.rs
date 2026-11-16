//! Street furniture as mesh: lamp posts with an arm and a head, sign
//! posts with a plate.
//!
//! The lamp is three boxes (pole, arm, head) and the sign two (post,
//! plate), built in the furniture's local frame and rotated into
//! place, so the arm reaches over the road and the plate faces the
//! driver. The plate carries no text: a mesh has no font, so the
//! shell textures the face from the JSON's `text`.

use crate::domain::curve::Pt;
use crate::domain::geometry::Rect;
use crate::domain::mesh::{Material, Mesh};
use crate::domain::placement::placed_point;
use crate::domain::street_furniture::{LAMP_HEIGHT, LAMP_REACH, RoadSign, StreetLamp};
use crate::use_cases::materials::CONCRETE;

/// The lamp pole and arm material id. The furniture materials sit
/// after the house table (last id CONCRETE) and the three ground
/// materials (street, lot, sidewalk), so the lamp is CONCRETE + 4.
/// Derived, never a literal — appending a house or ground material
/// shifts these with it.
pub const LAMP_MATERIAL: u32 = CONCRETE + 4;
/// The lamp head material id (glass, so the shell can make it glow).
pub const LAMP_GLASS_MATERIAL: u32 = CONCRETE + 5;
/// The sign post material id.
pub const SIGN_POST_MATERIAL: u32 = CONCRETE + 6;
/// The sign plate material id.
pub const SIGN_FACE_MATERIAL: u32 = CONCRETE + 7;

/// The four furniture materials, in id order.
pub fn furniture_materials() -> Vec<Material> {
    vec![
        Material { name: "lamp_post", color: [0.20, 0.21, 0.23] },
        Material { name: "lamp_glass", color: [0.98, 0.93, 0.72] },
        Material { name: "sign_post", color: [0.45, 0.47, 0.49] },
        Material { name: "sign_face", color: [0.86, 0.86, 0.84] },
    ]
}

/// Appends every lamp: pole, arm, and head.
pub fn add_lamps(mesh: &mut Mesh, lamps: &[StreetLamp]) {
    for lamp in lamps {
        let (o, r) = (lamp.position, lamp.rotation);
        let top = LAMP_HEIGHT + 0.2;
        add_box(mesh, o, r, (-0.09, 0.09), (-0.09, 0.09), (0.0, top), LAMP_MATERIAL);
        add_box(mesh, o, r, (-0.06, 0.06), (0.0, LAMP_REACH), (top - 0.12, top - 0.02), LAMP_MATERIAL);
        add_box(
            mesh,
            o,
            r,
            (-0.22, 0.22),
            (LAMP_REACH - 0.45, LAMP_REACH + 0.1),
            (LAMP_HEIGHT - 0.2, LAMP_HEIGHT + 0.05),
            LAMP_GLASS_MATERIAL,
        );
    }
}

/// Appends every sign: post and plate.
pub fn add_signs(mesh: &mut Mesh, signs: &[RoadSign]) {
    for sign in signs {
        let (o, r) = (sign.position, sign.rotation);
        add_box(mesh, o, r, (-0.05, 0.05), (-0.05, 0.05), (0.0, 2.3), SIGN_POST_MATERIAL);
        add_box(mesh, o, r, (-0.45, 0.45), (-0.02, 0.02), (1.55, 2.3), SIGN_FACE_MATERIAL);
    }
}

/// Appends one box in the frame `(origin, rotation)`: `x` across the
/// front, `y` along the front (toward the road or the driver), `z` in
/// height. Winding matches `Mesh::add_box`, because the placement is a
/// proper rotation.
fn add_box(
    mesh: &mut Mesh,
    origin: Pt,
    rotation: f64,
    x: (f64, f64),
    y: (f64, f64),
    z: (f64, f64),
    material: u32,
) {
    let corner = |x: f64, y: f64, h: f64| -> [f32; 3] {
        let p = placed_point(origin, rotation, Rect::new(0.0, 0.0, 0.0, 0.0), Pt::new(x, y));
        [p.x as f32, h as f32, p.y as f32]
    };
    let base = mesh.vertices.len() as u32;
    mesh.vertices.extend([
        corner(x.0, y.0, z.0),
        corner(x.1, y.0, z.0),
        corner(x.1, y.1, z.0),
        corner(x.0, y.1, z.0),
        corner(x.0, y.0, z.1),
        corner(x.1, y.0, z.1),
        corner(x.1, y.1, z.1),
        corner(x.0, y.1, z.1),
    ]);
    // Bottom, top, and the four sides, outward-facing: the same table
    // `Mesh::add_box` uses.
    let tris: [[u32; 3]; 12] = [
        [0, 1, 2],
        [0, 2, 3],
        [4, 6, 5],
        [4, 7, 6],
        [0, 5, 1],
        [0, 4, 5],
        [1, 6, 2],
        [1, 5, 6],
        [2, 7, 3],
        [2, 6, 7],
        [3, 4, 0],
        [3, 7, 4],
    ];
    for t in tris {
        mesh.faces.push([t[0] + base, t[1] + base, t[2] + base]);
        mesh.materials.push(material);
    }
}
