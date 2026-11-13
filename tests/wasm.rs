//! WASM payload tests: the builders the `extern "C"` exports
//! call return the same JSON the CLI emits, and the mesh bytes
//! parse back to the mesh they came from.

use floorplan::adapters::json::render_json;
use floorplan::adapters::neighborhood_json::render_neighborhood_json;
use floorplan::adapters::wasm::{
    MESH_MAGIC, PROTOCOL_VERSION, chunk_json, chunk_mesh_bytes, daylight_json, house_json,
    house_lights_json, house_lights_toggled_json, house_mesh_bytes, light_toggle_state, mesh_bytes,
    neighborhood_json, neighborhood_mesh_bytes,
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

#[test]
fn protocol_version_two_adds_chunks_without_breaking_v1() {
    assert_eq!(PROTOCOL_VERSION, 2);
    let v1 = neighborhood_json(42, 2, 2).unwrap();
    let chunk = chunk_json(42, 0, 0).unwrap();
    let text = String::from_utf8(chunk).unwrap();
    assert!(text.contains("\"chunk\":[0,0]"));
    assert!(
        text.contains(&String::from_utf8(v1).unwrap()),
        "chunk (0,0) must embed the fixed neighborhood verbatim"
    );
}

#[test]
fn chunk_mesh_for_the_origin_is_the_fixed_neighborhood_mesh() {
    assert_eq!(chunk_mesh_bytes(42, 0, 0).unwrap(), neighborhood_mesh_bytes(42, 2, 2).unwrap());
}

#[test]
fn a_chunk_mesh_parses_back_and_is_deterministic() {
    let bytes = chunk_mesh_bytes(42, 1, 0).unwrap();
    let (vs, fs, ms, table) = parse_mesh(&bytes);
    assert!(!vs.is_empty());
    assert!(!fs.is_empty());
    assert!(fs.iter().flatten().all(|&i| (i as usize) < vs.len()));
    assert!(ms.iter().all(|&m| (m as usize) < table.len()));
    assert_eq!(bytes, chunk_mesh_bytes(42, 1, 0).unwrap());
    assert_ne!(bytes, chunk_mesh_bytes(42, 2, 0).unwrap());
}

#[test]
fn house_lights_carry_the_flashlight_and_rooms() {
    let text = String::from_utf8(house_lights_json(42, 3).unwrap()).unwrap();
    assert!(text.contains("\"kind\":\"flashlight\""), "no flashlight");
    assert!(text.contains("\"kind\":\"ceiling\""), "no ceiling light");
    assert!(text.contains("\"kind\":\"window\""), "no window light");
}

#[test]
fn a_toggled_light_crosses_the_boundary() {
    let toggled = String::from_utf8(house_lights_toggled_json(42, 3, 1000).unwrap()).unwrap();
    let entry = toggled
        .split("{\"id\":1000,")
        .nth(1)
        .expect("room 0 has no ceiling light")
        .split('}')
        .next()
        .unwrap();
    assert!(entry.contains("\"state\":\"on\""), "the switch did not reach the light");
    assert!(house_lights_toggled_json(42, 3, 99999).is_err());
}

#[test]
fn daylight_json_has_day_and_night() {
    assert!(String::from_utf8(daylight_json(720, true)).unwrap().contains("\"phase\":\"day\""));
    assert!(String::from_utf8(daylight_json(0, false)).unwrap().contains("\"phase\":\"night\""));
}

#[test]
fn a_switch_code_toggles() {
    assert_eq!(light_toggle_state(0), Some(1));
    assert_eq!(light_toggle_state(1), Some(0));
    assert_eq!(light_toggle_state(3), Some(3));
    assert_eq!(light_toggle_state(9), None);
}
