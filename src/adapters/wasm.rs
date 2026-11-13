//! WASM adapter: the core as a wasm32 module with no crates and no
//! wasm-bindgen. The web shell calls raw `extern "C"` exports (in
//! `wasm_exports`) and reads the results out of linear memory.
//!
//! Memory protocol (see ADR 0004 and ADR 0007):
//! - `fp_alloc(size)` returns a pointer the caller owns; free it
//!   with `fp_free(ptr, size)` when done.
//! - Every payload export returns a pointer to its bytes and
//!   records the length in `fp_last_len()`; a null pointer means
//!   the call was refused and the length is 0.
//! - `fp_version()` returns the protocol version (2).
//! - JSON payloads are the same text the CLI writers emit.
//! - Mesh payloads are the binary format `mesh_bytes` documents.
//!
//! The payload builders here are plain Rust and tested natively;
//! only the `extern "C"` exports are wasm-only.

use crate::adapters::chunk_json::render_chunk_json;
use crate::adapters::json::render_json;
use crate::adapters::neighborhood_json::render_neighborhood_json;
use crate::domain::style::Style;
use crate::use_cases::chunk::build_chunk;
use crate::use_cases::extrude::extrude;
use crate::use_cases::generate::generate_house;
use crate::use_cases::neighborhood::generate_neighborhood;
use crate::use_cases::neighborhood_mesh::neighborhood_mesh;

pub use crate::adapters::mesh_payload::{MESH_MAGIC, mesh_bytes};

/// The protocol version the shell checks against. Version 2 adds
/// chunk generation by (seed, cx, cz) and light/daylight queries;
/// every version 1 call keeps its semantics.
pub const PROTOCOL_VERSION: u32 = 2;

/// The style index the WASM exports take: `Style::all()` order
/// (0 Ranch, 1 Colonial, 2 Craftsman, 3 Modern, 4 Cottage,
/// 5 Townhouse).
pub fn style_for_index(index: u32) -> Option<Style> {
    Style::all().get(index as usize).copied()
}

/// The house JSON for a seed and style index: the same text the
/// CLI `--json` writer emits.
pub fn house_json(seed: u32, style_index: u32) -> Result<Vec<u8>, String> {
    let Some(style) = style_for_index(style_index) else {
        return Err(format!("style index {style_index} out of range (0..6)"));
    };
    Ok(render_json(&generate_house(seed, style)).into_bytes())
}

/// The neighborhood JSON for a seed and block count: the same
/// text the CLI `neighborhood --json` writer emits.
pub fn neighborhood_json(seed: u32, cols: u32, rows: u32) -> Result<Vec<u8>, String> {
    Ok(render_neighborhood_json(&generate_neighborhood(seed, (cols, rows))).into_bytes())
}

/// The house mesh for a seed and style index, in the binary
/// format `mesh_bytes` documents.
pub fn house_mesh_bytes(seed: u32, style_index: u32) -> Result<Vec<u8>, String> {
    let Some(style) = style_for_index(style_index) else {
        return Err(format!("style index {style_index} out of range (0..6)"));
    };
    Ok(mesh_bytes(&extrude(&generate_house(seed, style))))
}

/// The neighborhood mesh for a seed and block count, in the
/// binary format `mesh_bytes` documents.
pub fn neighborhood_mesh_bytes(seed: u32, cols: u32, rows: u32) -> Result<Vec<u8>, String> {
    Ok(mesh_bytes(&neighborhood_mesh(&generate_neighborhood(seed, (cols, rows)))))
}

/// The JSON for one chunk of the infinite neighborhood: its
/// coordinates, lifecycle state, and the neighborhood document.
/// Chunk (0,0) carries the fixed neighborhood's JSON verbatim.
pub fn chunk_json(seed: u32, cx: i32, cz: i32) -> Result<Vec<u8>, String> {
    render_chunk_json(&build_chunk(seed, cx, cz))
        .map(String::into_bytes)
        .ok_or_else(|| "chunk has no content".to_string())
}

/// The mesh for one chunk, in the binary format `mesh_bytes`
/// documents. Chunk (0,0) is the fixed neighborhood's mesh.
pub fn chunk_mesh_bytes(seed: u32, cx: i32, cz: i32) -> Result<Vec<u8>, String> {
    let chunk = build_chunk(seed, cx, cz);
    let neighborhood = chunk
        .neighborhood
        .as_ref()
        .ok_or_else(|| "chunk has no content".to_string())?;
    Ok(mesh_bytes(&neighborhood_mesh(neighborhood)))
}
