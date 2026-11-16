//! Flat ribbons: ground surfaces that face up.
//!
//! Streets, sidewalks, lots, walkways, driveways, and centerlines
//! are all flat quads at a fixed height. A quad whose plan corners
//! wind clockwise (positive signed area, x east, y south) faces
//! down, so the winding is checked here and reversed when needed
//! — never assumed, after the wall-winding lesson.

use crate::domain::curve::Pt;
use crate::domain::mesh::Mesh;
use crate::domain::polygon::signed_area;

/// Adds one flat quad at height `y`, winding corrected to face up.
pub fn add_flat_quad(mesh: &mut Mesh, a: Pt, b: Pt, c: Pt, d: Pt, y: f32, material: u32) {
    let (a3, b3, c3, d3) = (pt3(a, y), pt3(b, y), pt3(c, y), pt3(d, y));
    if signed_area(&[a, b, c, d]) < 0.0 {
        mesh.add_quad(a3, b3, c3, d3, material);
    } else {
        mesh.add_quad(a3, d3, c3, b3, material);
    }
}

fn pt3(p: Pt, y: f32) -> [f32; 3] {
    [p.x as f32, y, p.y as f32]
}

/// Adds a flat ribbon between two polylines at height `y`: one
/// quad per segment, all facing up. The two polylines run the
/// same direction and have the same point count.
pub fn add_ribbon(mesh: &mut Mesh, left: &[Pt], right: &[Pt], y: f32, material: u32) {
    for (l, r) in left.windows(2).zip(right.windows(2)) {
        add_flat_quad(mesh, l[0], l[1], r[1], r[0], y, material);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::materials::material_table;

    /// The y component of a triangle's normal; positive faces up.
    fn normal_y(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
        let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
        u[2] * v[0] - u[0] * v[2]
    }

    fn quad_up(mesh: &Mesh) -> bool {
        mesh.faces.iter().all(|f| {
            normal_y(mesh.vertices[f[0] as usize], mesh.vertices[f[1] as usize], mesh.vertices[f[2] as usize]) > 0.0
        })
    }

    #[test]
    fn a_quad_faces_up_whichever_order_the_corners_arrive_in() {
        let (a, b, c, d) = (Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), Pt::new(10.0, 5.0), Pt::new(0.0, 5.0));
        for (p, q, r, s) in [(a, b, c, d), (a, d, c, b)] {
            let mut mesh = Mesh::new(material_table());
            add_flat_quad(&mut mesh, p, q, r, s, 0.0, 0);
            assert!(quad_up(&mesh), "quad {p:?} {q:?} {r:?} {s:?} faces down");
        }
    }

    #[test]
    fn a_ribbon_faces_up() {
        let left = vec![Pt::new(0.0, 0.0), Pt::new(10.0, 2.0), Pt::new(20.0, 2.0)];
        let right = vec![Pt::new(0.0, 8.0), Pt::new(10.0, 10.0), Pt::new(20.0, 10.0)];
        let mut mesh = Mesh::new(material_table());
        add_ribbon(&mut mesh, &left, &right, 0.0, 0);
        assert_eq!(mesh.faces.len(), 4, "two quads");
        assert!(quad_up(&mesh), "ribbon faces down");
    }
}
