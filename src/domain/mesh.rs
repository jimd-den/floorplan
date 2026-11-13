//! A triangle mesh: the single internal 3D representation.
//!
//! Coordinates are metres: x east, y up, z south (plan y). Faces
//! are triangles; `materials[i]` is the material id of `faces[i]`.
//! Both the OBJ and the glTF writer read this one mesh — there is
//! no second extrusion.

/// One material: a name and an RGB colour.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: &'static str,
    pub color: [f32; 3],
}

/// An axis-aligned box: min and max corners, in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Box3 {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Box3 {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// True when the point is inside the box (inclusive).
    pub fn contains(&self, p: [f32; 3]) -> bool {
        (0..3).all(|i| p[i] >= self.min[i] - 1e-6 && p[i] <= self.max[i] + 1e-6)
    }
}

/// A triangle mesh with a per-face material id.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub materials: Vec<u32>,
    pub material_table: Vec<Material>,
}

impl Mesh {
    pub fn new(material_table: Vec<Material>) -> Self {
        Self { vertices: vec![], faces: vec![], materials: vec![], material_table }
    }

    /// Adds a box as 8 vertices and 12 triangles, all in `material`.
    pub fn add_box(&mut self, b: Box3, material: u32) {
        let [x0, y0, z0] = b.min;
        let [x1, y1, z1] = b.max;
        let base = self.vertices.len() as u32;
        self.vertices.extend([
            [x0, y0, z0],
            [x1, y0, z0],
            [x1, y0, z1],
            [x0, y0, z1],
            [x0, y1, z0],
            [x1, y1, z0],
            [x1, y1, z1],
            [x0, y1, z1],
        ]);
        // Bottom, top, and the four sides, outward-facing.
        let tris: [[u32; 3]; 12] = [
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 1, 5],
            [0, 5, 4],
            [1, 2, 6],
            [1, 6, 5],
            [2, 3, 7],
            [2, 7, 6],
            [3, 0, 4],
            [3, 4, 7],
        ];
        for t in tris {
            self.faces.push([t[0] + base, t[1] + base, t[2] + base]);
            self.materials.push(material);
        }
    }

    /// Adds one triangle in `material`.
    pub fn add_triangle(&mut self, a: [f32; 3], b: [f32; 3], c: [f32; 3], material: u32) {
        let base = self.vertices.len() as u32;
        self.vertices.extend([a, b, c]);
        self.faces.push([base, base + 1, base + 2]);
        self.materials.push(material);
    }

    /// Adds a quad as two triangles, all in `material`.
    pub fn add_quad(
        &mut self,
        a: [f32; 3],
        b: [f32; 3],
        c: [f32; 3],
        d: [f32; 3],
        material: u32,
    ) {
        self.add_triangle(a, b, c, material);
        self.add_triangle(a, c, d, material);
    }

    /// Every face indexes a real vertex, has three distinct corners,
    /// and covers a non-zero area; every material id is in the table.
    pub fn validate(&self) -> Result<(), String> {
        for (i, f) in self.faces.iter().enumerate() {
            if f[0] == f[1] || f[1] == f[2] || f[0] == f[2] {
                return Err(format!("face {i} is degenerate: {f:?}"));
            }
            for &v in f {
                if v as usize >= self.vertices.len() {
                    return Err(format!("face {i} indexes missing vertex {v}"));
                }
            }
            if self.materials.get(i).is_none_or(|&m| m as usize >= self.material_table.len()) {
                return Err(format!("face {i} has no material in the table"));
            }
            let (a, b, c) = (self.vertices[f[0] as usize], self.vertices[f[1] as usize], self.vertices[f[2] as usize]);
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            if cross.iter().map(|v| v * v).sum::<f32>().sqrt() < 1e-9 {
                return Err(format!("face {i} has zero area"));
            }
        }
        if self.faces.len() != self.materials.len() {
            return Err("faces and materials differ in length".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> Vec<Material> {
        vec![Material { name: "wall", color: [0.8, 0.8, 0.8] }]
    }

    #[test]
    fn a_box_has_eight_vertices_and_twelve_triangles() {
        let mut mesh = Mesh::new(table());
        mesh.add_box(Box3::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]), 0);
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.faces.len(), 12);
        assert_eq!(mesh.materials.len(), 12);
    }

    #[test]
    fn a_valid_mesh_passes_validation() {
        let mut mesh = Mesh::new(table());
        mesh.add_box(Box3::new([0.0, 0.0, 0.0], [2.0, 2.7, 3.0]), 0);
        assert_eq!(mesh.validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_an_out_of_range_index() {
        let mut mesh = Mesh::new(table());
        mesh.vertices.push([0.0, 0.0, 0.0]);
        mesh.faces.push([0, 1, 2]);
        mesh.materials.push(0);
        assert!(mesh.validate().is_err());
    }

    #[test]
    fn validate_rejects_a_degenerate_face() {
        let mut mesh = Mesh::new(table());
        mesh.vertices.extend([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        mesh.faces.push([0, 0, 1]);
        mesh.materials.push(0);
        assert!(mesh.validate().is_err());
    }

    #[test]
    fn validate_rejects_a_missing_material() {
        let mut mesh = Mesh::new(table());
        mesh.add_box(Box3::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]), 9);
        assert!(mesh.validate().is_err());
    }

    #[test]
    fn a_box_contains_its_interior_point() {
        let b = Box3::new([0.0, 0.0, 0.0], [2.0, 2.7, 3.0]);
        assert!(b.contains([1.0, 1.0, 1.0]));
        assert!(!b.contains([3.0, 1.0, 1.0]));
    }
}
