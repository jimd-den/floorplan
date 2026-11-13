//! WASM payload tests: the builders the `extern "C"` exports
//! call return the same JSON the CLI emits, and the mesh bytes
//! parse back to the mesh they came from.

use floorplan::adapters::json::render_json;
use floorplan::adapters::neighborhood_json::render_neighborhood_json;
use floorplan::adapters::wasm::{
    MESH_MAGIC, house_json, house_mesh_bytes, mesh_bytes, neighborhood_json, neighborhood_mesh_bytes,
};
use floorplan::domain::style::Style;
use floorplan::use_cases::extrude::extrude;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::neighborhood::generate_neighborhood;
use floorplan::use_cases::neighborhood_mesh::neighborhood_mesh;

/// A parsed mesh payload: vertices, faces, material ids, and the
/// material table (name, colour).
type ParsedMesh = (Vec<[f32; 3]>, Vec<[u32; 3]>, Vec<u32>, Vec<(String, [f32; 3])>);

/// Reads the binary mesh format back. Panics on malformed bytes,
/// like a shell would.
fn parse_mesh(bytes: &[u8]) -> ParsedMesh {
    assert_eq!(&bytes[0..4], &MESH_MAGIC, "bad magic");
    let u32_at = |i: usize| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
    let f32_at = |i: usize| f32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
    let verts = u32_at(4) as usize;
    let faces = u32_at(8) as usize;
    let materials = u32_at(12) as usize;
    let mut off = 16;
    let mut vs = Vec::with_capacity(verts);
    for _ in 0..verts {
        vs.push([f32_at(off), f32_at(off + 4), f32_at(off + 8)]);
        off += 12;
    }
    let mut fs = Vec::with_capacity(faces);
    for _ in 0..faces {
        fs.push([u32_at(off), u32_at(off + 4), u32_at(off + 8)]);
        off += 12;
    }
    let mut ms = Vec::with_capacity(faces);
    for _ in 0..faces {
        ms.push(u32_at(off));
        off += 4;
    }
    let mut table = Vec::with_capacity(materials);
    for _ in 0..materials {
        let len = bytes[off] as usize;
        off += 1;
        let name = String::from_utf8(bytes[off..off + len].to_vec()).unwrap();
        off += len;
        table.push((name, [f32_at(off), f32_at(off + 4), f32_at(off + 8)]));
        off += 12;
    }
    assert_eq!(off, bytes.len(), "trailing bytes in mesh payload");
    (vs, fs, ms, table)
}

#[test]
fn house_json_matches_the_cli_json() {
    for (index, style) in Style::all().iter().enumerate() {
        let bytes = house_json(42, index as u32).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert_eq!(text, render_json(&generate_house(42, *style)));
    }
}

#[test]
fn neighborhood_json_matches_the_cli_json() {
    let bytes = neighborhood_json(42, 2, 2).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert_eq!(text, render_neighborhood_json(&generate_neighborhood(42, (2, 2))));
}

#[test]
fn house_mesh_bytes_parse_back_to_the_house_mesh() {
    let house = generate_house(42, Style::Modern);
    let mesh = extrude(&house);
    let bytes = house_mesh_bytes(42, 3).unwrap();
    let (vs, fs, ms, table) = parse_mesh(&bytes);
    assert_eq!(vs.len(), mesh.vertices.len());
    assert_eq!(fs.len(), mesh.faces.len());
    assert_eq!(ms.len(), mesh.materials.len());
    assert_eq!(vs[0], mesh.vertices[0]);
    assert_eq!(fs[0], mesh.faces[0]);
    assert_eq!(ms[0], mesh.materials[0]);
    assert_eq!(table.len(), mesh.material_table.len());
    assert_eq!(table[0].0, mesh.material_table[0].name);
    assert_eq!(table[0].1, mesh.material_table[0].color);
}

#[test]
fn neighborhood_mesh_bytes_parse_back_to_the_neighborhood_mesh() {
    let n = generate_neighborhood(42, (2, 2));
    let mesh = neighborhood_mesh(&n);
    let bytes = neighborhood_mesh_bytes(42, 2, 2).unwrap();
    let (vs, fs, ms, table) = parse_mesh(&bytes);
    assert_eq!(vs.len(), mesh.vertices.len());
    assert_eq!(fs.len(), mesh.faces.len());
    assert_eq!(ms.len(), mesh.materials.len());
    assert_eq!(table.len(), mesh.material_table.len());
    assert!(ms.iter().all(|&m| (m as usize) < table.len()), "material id out of range");
    assert!(fs.iter().flatten().all(|&i| (i as usize) < vs.len()), "face index out of range");
}

#[test]
fn mesh_bytes_are_deterministic() {
    assert_eq!(mesh_bytes(&extrude(&generate_house(42, Style::Modern))), mesh_bytes(&extrude(&generate_house(42, Style::Modern))));
    assert_eq!(house_mesh_bytes(7, 0).unwrap(), house_mesh_bytes(7, 0).unwrap());
}

#[test]
fn an_invalid_style_index_is_refused() {
    assert!(house_json(42, 6).is_err());
    assert!(house_mesh_bytes(42, 99).is_err());
    assert!(house_json(42, 5).is_ok(), "Townhouse is index 5");
}
