//! Regression: `Mesh::add_box` triangles must face outward.
//!
//! The web shell renders front faces only, so an inward-wound
//! box culls its near face and lets interior furniture draw in
//! front of a solid wall. The software renderer never culls,
//! which is why this shipped unseen.

use floorplan::domain::mesh::{Box3, Material, Mesh};

fn table() -> Vec<Material> {
    vec![Material { name: "wall", color: [0.8, 0.8, 0.8] }]
}

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

#[test]
fn box_faces_point_outward_from_the_box_centre() {
    let mut mesh = Mesh::new(table());
    mesh.add_box(Box3::new([0.0, 0.0, 0.0], [2.0, 2.7, 3.0]), 0);

    let centre = [1.0, 1.35, 1.5];
    assert_eq!(mesh.faces.len(), 12);
    for (i, face) in mesh.faces.iter().enumerate() {
        let a = mesh.vertices[face[0] as usize];
        let b = mesh.vertices[face[1] as usize];
        let c = mesh.vertices[face[2] as usize];
        let normal = cross(sub(b, a), sub(c, a));
        let centroid = [
            (a[0] + b[0] + c[0]) / 3.0,
            (a[1] + b[1] + c[1]) / 3.0,
            (a[2] + b[2] + c[2]) / 3.0,
        ];
        let outward = sub(centroid, centre);
        assert!(
            dot(normal, outward) > 0.0,
            "face {i} {face:?} points inward: normal {normal:?}, outward {outward:?}"
        );
    }
}
