//! glTF 2.0 writer: the mesh as a JSON .gltf with the buffer
//! embedded as base64. One primitive per material, positions as
//! f32, indices as u32. The base64 encoder is hand-rolled, std
//! only. Deterministic: the same mesh renders byte-identical JSON.

use crate::domain::mesh::Mesh;

/// Renders the mesh as a glTF 2.0 document.
pub fn render_gltf(mesh: &Mesh) -> String {
    let mut buf: Vec<u8> = vec![];
    for v in &mesh.vertices {
        for &c in v {
            buf.extend_from_slice(&c.to_le_bytes());
        }
    }
    let mut views: Vec<(usize, usize)> = vec![(0, mesh.vertices.len() * 12)];
    let mut prims: Vec<(usize, usize, usize)> = vec![];
    for (id, _) in mesh.material_table.iter().enumerate() {
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
        let off = buf.len();
        for f in &faces {
            for &v in f.iter() {
                buf.extend_from_slice(&v.to_le_bytes());
            }
        }
        let acc = views.len();
        views.push((off, faces.len() * 12));
        prims.push((id, acc, faces.len() * 3));
    }
    let (mn, mx) = bounds(mesh);
    let mut s = String::new();
    s.push_str("{\"asset\":{\"version\":\"2.0\",\"generator\":\"floorplan\"},");
    s.push_str("\"scene\":0,\"scenes\":[{\"nodes\":[0]}],\"nodes\":[{\"mesh\":0}],");
    s.push_str("\"meshes\":[{\"primitives\":[");
    for (i, (mat, acc, _)) in prims.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"attributes\":{{\"POSITION\":0}},\"indices\":{acc},\"material\":{mat}}}"
        ));
    }
    s.push_str("]}],\"materials\":[");
    for (i, mat) in mesh.material_table.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"pbrMetallicRoughness\":{{\"baseColorFactor\":[{:.3},{:.3},{:.3},1.0],\"metallicFactor\":0.0,\"roughnessFactor\":0.9}}}}",
            mat.name, mat.color[0], mat.color[1], mat.color[2]
        ));
    }
    s.push_str("],\"accessors\":[");
    s.push_str(&format!(
        "{{\"bufferView\":0,\"componentType\":5126,\"count\":{},\"type\":\"VEC3\",\"min\":[{:.3},{:.3},{:.3}],\"max\":[{:.3},{:.3},{:.3}]}}",
        mesh.vertices.len(),
        mn[0], mn[1], mn[2],
        mx[0], mx[1], mx[2]
    ));
    for (i, (_, _, count)) in prims.iter().enumerate() {
        s.push_str(&format!(
            ",{{\"bufferView\":{},\"componentType\":5125,\"count\":{},\"type\":\"SCALAR\"}}",
            i + 1,
            count
        ));
    }
    s.push_str("],\"bufferViews\":[");
    for (i, (off, len)) in views.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("{{\"buffer\":0,\"byteOffset\":{off},\"byteLength\":{len}}}"));
    }
    s.push_str(&format!(
        "],\"buffers\":[{{\"byteLength\":{},\"uri\":\"data:application/octet-stream;base64,{}\"}}]}}",
        buf.len(),
        base64_encode(&buf)
    ));
    s
}

/// The min and max vertex of the mesh.
fn bounds(mesh: &Mesh) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::INFINITY; 3];
    let mut mx = [f32::NEG_INFINITY; 3];
    for v in &mesh.vertices {
        for i in 0..3 {
            mn[i] = mn[i].min(v[i]);
            mx[i] = mx[i].max(v[i]);
        }
    }
    (mn, mx)
}

/// Base64, standard alphabet with padding, std only.
pub fn base64_encode(data: &[u8]) -> String {
    const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::extrude::extrude;
    use crate::use_cases::generate::generate_house;

    #[test]
    fn base64_encodes_the_standard_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"M"), "TQ==");
        assert_eq!(base64_encode(b"Ma"), "TWE=");
        assert_eq!(base64_encode(b"Man"), "TWFu");
        assert_eq!(base64_encode(b"floorplan"), "Zmxvb3JwbGFu");
    }

    #[test]
    fn gltf_is_deterministic() {
        let m = extrude(&generate_house(42, Style::Ranch));
        assert_eq!(render_gltf(&m), render_gltf(&m));
    }

    #[test]
    fn gltf_names_its_version_materials_and_buffer_length() {
        let m = extrude(&generate_house(42, Style::Ranch));
        let g = render_gltf(&m);
        assert!(g.contains("\"version\":\"2.0\""));
        assert!(g.contains("\"name\":\"wall\""));
        let expected = m.vertices.len() * 12 + m.faces.len() * 12;
        assert!(g.contains(&format!("\"byteLength\":{expected}")), "buffer length wrong");
        assert!(g.contains("data:application/octet-stream;base64,"));
    }

    #[test]
    fn gltf_has_one_primitive_per_material_present() {
        let m = extrude(&generate_house(42, Style::Ranch));
        let g = render_gltf(&m);
        let present = m.materials.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(g.matches("\"attributes\"").count(), present);
    }
}
