//! OBJ + MTL writer: the mesh as a Wavefront OBJ document.
//!
//! Coordinates to three decimals, faces 1-based, one material per
//! room program plus wall / ceiling / roof. Deterministic: the same
//! mesh always renders byte-identical text.

use crate::domain::mesh::Mesh;

/// Renders the mesh as an OBJ document. `mtl_name` is the file
/// name of the companion MTL, referenced by the `mtllib` line.
pub fn render_obj(mesh: &Mesh, mtl_name: &str) -> String {
    let mut s = String::new();
    s.push_str("# floorplan\n");
    s.push_str(&format!("mtllib {mtl_name}\n"));
    for v in &mesh.vertices {
        s.push_str(&format!("v {:.3} {:.3} {:.3}\n", v[0], v[1], v[2]));
    }
    for (id, mat) in mesh.material_table.iter().enumerate() {
        let faces: Vec<_> = mesh
            .faces
            .iter()
            .zip(mesh.materials.iter())
            .filter(|(_, m)| **m as usize == id)
            .map(|(f, _)| f)
            .collect();
        if faces.is_empty() {
            continue;
        }
        s.push_str(&format!("usemtl {}\n", mat.name));
        for f in faces {
            s.push_str(&format!("f {} {} {}\n", f[0] + 1, f[1] + 1, f[2] + 1));
        }
    }
    s
}

/// Renders the companion MTL: one material per table entry.
pub fn render_mtl(mesh: &Mesh) -> String {
    let mut s = String::new();
    for mat in &mesh.material_table {
        s.push_str(&format!("newmtl {}\n", mat.name));
        s.push_str(&format!("Kd {:.3} {:.3} {:.3}\n", mat.color[0], mat.color[1], mat.color[2]));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::extrude::extrude;
    use crate::use_cases::generate::generate_house;

    fn mesh() -> Mesh {
        extrude(&generate_house(42, Style::Ranch))
    }

    #[test]
    fn obj_is_deterministic() {
        let m = mesh();
        assert_eq!(render_obj(&m, "house.mtl"), render_obj(&m, "house.mtl"));
        assert_eq!(render_mtl(&m), render_mtl(&m));
    }

    #[test]
    fn obj_faces_reference_real_vertices() {
        let obj = render_obj(&mesh(), "house.mtl");
        let nverts = obj.lines().filter(|l| l.starts_with("v ")).count();
        assert!(nverts > 0);
        for line in obj.lines().filter(|l| l.starts_with("f ")) {
            for idx in line[2..].split_whitespace() {
                let v: usize = idx.parse().unwrap();
                assert!(v >= 1 && v <= nverts, "face index {v} out of range");
            }
        }
    }

    #[test]
    fn obj_references_the_mtl_and_a_material_per_program() {
        let m = mesh();
        let obj = render_obj(&m, "house.mtl");
        let mtl = render_mtl(&m);
        assert!(obj.contains("mtllib house.mtl"));
        for mat in &m.material_table {
            assert!(mtl.contains(&format!("newmtl {}", mat.name)), "no {}", mat.name);
        }
        assert!(obj.contains("usemtl wall"), "no wall material in the OBJ");
    }
}
