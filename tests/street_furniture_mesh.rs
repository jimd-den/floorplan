//! Street furniture meshes: a lamp is a pole, an arm, and a head; a
//! sign is a post and a plate.

use floorplan::domain::curve::Pt;
use floorplan::domain::mesh::Mesh;
use floorplan::domain::street_furniture::{RoadSign, SignKind, StreetLamp};
use floorplan::use_cases::neighborhood_mesh::neighborhood_material_table;
use floorplan::use_cases::street_furniture_mesh::{
    LAMP_GLASS_MATERIAL, LAMP_MATERIAL, SIGN_FACE_MATERIAL, SIGN_POST_MATERIAL, add_lamps,
    add_signs,
};

fn mesh() -> Mesh {
    Mesh::new(neighborhood_material_table())
}

fn lamp(rotation: f64) -> StreetLamp {
    StreetLamp { position: Pt::new(0.0, 0.0), rotation }
}

fn count(mesh: &Mesh, material: u32) -> usize {
    mesh.materials.iter().filter(|&&m| m == material).count()
}

#[test]
fn a_lamp_is_a_pole_an_arm_and_a_head() {
    let mut m = mesh();
    add_lamps(&mut m, &[lamp(0.0)]);
    assert_eq!(m.vertices.len(), 24, "three boxes");
    assert_eq!(m.faces.len(), 36, "twelve triangles per box");
    assert_eq!(count(&m, LAMP_MATERIAL), 24, "pole and arm");
    assert_eq!(count(&m, LAMP_GLASS_MATERIAL), 12, "head");
    assert_eq!(m.validate(), Ok(()));
}

#[test]
fn the_lamp_head_hangs_over_the_road_side_of_the_pole() {
    // Rotation -90 faces east: the arm and head are east of the pole.
    let mut m = mesh();
    add_lamps(&mut m, &[lamp(-90.0)]);
    let head: Vec<f32> = m
        .faces
        .iter()
        .enumerate()
        .filter(|(i, _)| m.materials[*i] == LAMP_GLASS_MATERIAL)
        .flat_map(|(_, f)| f.iter().map(|&v| m.vertices[v as usize][0]))
        .collect();
    let min = head.iter().cloned().fold(f32::MAX, f32::min);
    assert!(!head.is_empty(), "the lamp has no head");
    assert!(min > 1.5, "the head starts at x {min:.2}, not over the road");
}

#[test]
fn a_lamp_pole_stands_on_the_ground() {
    let mut m = mesh();
    add_lamps(&mut m, &[lamp(0.0)]);
    let lowest = m.vertices.iter().map(|v| v[1]).fold(f32::MAX, f32::min);
    assert_eq!(lowest, 0.0, "the pole floats or sinks");
}

#[test]
fn a_sign_is_a_post_and_a_plate() {
    let mut m = mesh();
    let sign = RoadSign {
        position: Pt::new(0.0, 0.0),
        rotation: 0.0,
        kind: SignKind::Stop,
        text: "STOP".into(),
    };
    add_signs(&mut m, &[sign]);
    assert_eq!(m.vertices.len(), 16, "two boxes");
    assert_eq!(m.faces.len(), 24, "twelve triangles per box");
    assert_eq!(count(&m, SIGN_POST_MATERIAL), 12);
    assert_eq!(count(&m, SIGN_FACE_MATERIAL), 12);
    assert_eq!(m.validate(), Ok(()));
}

#[test]
fn the_plate_is_thin_across_the_direction_it_faces() {
    let mut m = mesh();
    let sign = RoadSign {
        position: Pt::new(0.0, 0.0),
        rotation: 0.0,
        kind: SignKind::SpeedLimit,
        text: "25".into(),
    };
    add_signs(&mut m, &[sign]);
    let face: Vec<[f32; 3]> = m
        .faces
        .iter()
        .enumerate()
        .filter(|(i, _)| m.materials[*i] == SIGN_FACE_MATERIAL)
        .flat_map(|(_, f)| f.iter().map(|&v| m.vertices[v as usize]))
        .collect();
    // Rotation 0 faces south, so the plate is thin along z and wide
    // along x.
    let xs: Vec<f32> = face.iter().map(|v| v[0]).collect();
    let zs: Vec<f32> = face.iter().map(|v| v[2]).collect();
    let wide = xs.iter().cloned().fold(f32::MIN, f32::max) - xs.iter().cloned().fold(f32::MAX, f32::min);
    let thin = zs.iter().cloned().fold(f32::MIN, f32::max) - zs.iter().cloned().fold(f32::MAX, f32::min);
    assert!(wide > 0.8, "plate only {wide:.2} m wide");
    assert!(thin < 0.1, "plate {thin:.2} m thick along its facing");
}
