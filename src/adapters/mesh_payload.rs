//! The binary mesh payload format shared by the WASM exports.
//!
//! All fields are little-endian: magic "FPM1", three counts, then
//! positions, indices, material ids, and the material table. The
//! format is version 1 of the mesh payload; chunking adds no bytes
//! to it, so a chunk mesh is a neighborhood mesh.

use crate::domain::mesh::Mesh;

/// The mesh magic: "FPM1" (floorplan mesh, version 1).
pub const MESH_MAGIC: [u8; 4] = *b"FPM1";

/// Serializes a mesh. The same mesh always yields the same bytes.
pub fn mesh_bytes(mesh: &Mesh) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&MESH_MAGIC);
    out.extend_from_slice(&(mesh.vertices.len() as u32).to_le_bytes());
    out.extend_from_slice(&(mesh.faces.len() as u32).to_le_bytes());
    out.extend_from_slice(&(mesh.material_table.len() as u32).to_le_bytes());
    for v in &mesh.vertices {
        for c in v {
            out.extend_from_slice(&c.to_le_bytes());
        }
    }
    for f in &mesh.faces {
        for i in f {
            out.extend_from_slice(&i.to_le_bytes());
        }
    }
    for m in &mesh.materials {
        out.extend_from_slice(&m.to_le_bytes());
    }
    for mat in &mesh.material_table {
        out.push(mat.name.len() as u8);
        out.extend_from_slice(mat.name.as_bytes());
        for c in mat.color {
            out.extend_from_slice(&c.to_le_bytes());
        }
    }
    out
}
