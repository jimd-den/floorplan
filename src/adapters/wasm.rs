//! WASM adapter: the core as a wasm32 module with no crates and
//! no wasm-bindgen. The web shell calls raw `extern "C"` exports
//! and reads the results out of linear memory.
//!
//! Memory protocol (see ADR 0004 for the full contract):
//! - `fp_alloc(size)` returns a pointer the caller owns; free it
//!   with `fp_free(ptr, size)` when done.
//! - Every payload export returns a pointer to its bytes and
//!   records the length in `fp_last_len()`; a null pointer means
//!   the call was refused (invalid argument) and the length is 0.
//! - `fp_version()` returns the protocol version (1).
//! - JSON payloads are the same text the CLI writers emit.
//! - Mesh payloads are the binary format `mesh_bytes` documents.
//!
//! The payload builders are plain Rust and tested natively; only
//! the `extern "C"` exports are wasm-only. The module is built
//! with `-C panic=abort`, so a bug traps instead of unwinding.

use crate::adapters::json::render_json;
use crate::adapters::neighborhood_json::render_neighborhood_json;
use crate::domain::mesh::Mesh;
use crate::domain::style::Style;
use crate::use_cases::extrude::extrude;
use crate::use_cases::generate::generate_house;
use crate::use_cases::neighborhood::generate_neighborhood;
use crate::use_cases::neighborhood_mesh::neighborhood_mesh;

/// The protocol version the shell checks against.
pub const PROTOCOL_VERSION: u32 = 1;

/// The mesh magic: "FPM1" (floorplan mesh, version 1).
pub const MESH_MAGIC: [u8; 4] = *b"FPM1";

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

/// The mesh binary format, all little-endian:
/// - magic "FPM1" (4 bytes)
/// - u32 vert_count, u32 face_count, u32 material_count
/// - f32[vert_count * 3] positions (x, y, z)
/// - u32[face_count * 3] indices
/// - u32[face_count] material ids
/// - material table: per material, u8 name_len, name bytes,
///   f32[3] colour
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

#[cfg(target_arch = "wasm32")]
mod exports {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static LAST_LEN: AtomicU32 = AtomicU32::new(0);

    /// Moves a payload into WASM linear memory and returns its
    /// pointer; records its length for `fp_last_len`. A refused
    /// call returns a null pointer and length 0.
    fn leak_payload(result: Result<Vec<u8>, String>) -> *mut u8 {
        match result {
            Ok(bytes) => {
                LAST_LEN.store(bytes.len() as u32, Ordering::Relaxed);
                Box::leak(bytes.into_boxed_slice()).as_mut_ptr()
            }
            Err(_) => {
                LAST_LEN.store(0, Ordering::Relaxed);
                std::ptr::null_mut()
            }
        }
    }

    /// Allocates `size` bytes in linear memory. The caller owns
    /// the pointer and frees it with `fp_free`.
    #[unsafe(no_mangle)]
    pub extern "C" fn fp_alloc(size: usize) -> *mut u8 {
        Box::leak(vec![0u8; size].into_boxed_slice()).as_mut_ptr()
    }

    /// Frees a pointer previously returned by `fp_alloc` or a
    /// payload export. `size` must be the size that was allocated.
    #[unsafe(no_mangle)]
    pub extern "C" fn fp_free(ptr: *mut u8, size: usize) {
        if ptr.is_null() || size == 0 {
            return;
        }
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, size));
        }
    }

    /// The length of the payload the last export returned, in
    /// bytes. 0 after a refused call.
    #[unsafe(no_mangle)]
    pub extern "C" fn fp_last_len() -> u32 {
        LAST_LEN.load(Ordering::Relaxed)
    }

    /// The protocol version (1).
    #[unsafe(no_mangle)]
    pub extern "C" fn fp_version() -> u32 {
        PROTOCOL_VERSION
    }

    /// The house JSON for `seed` and a style index (see
    /// `style_for_index`). Null on an invalid style index.
    #[unsafe(no_mangle)]
    pub extern "C" fn fp_house_json(seed: u32, style: u32) -> *mut u8 {
        leak_payload(house_json(seed, style))
    }

    /// The neighborhood JSON for `seed` and a (columns, rows)
    /// block count.
    #[unsafe(no_mangle)]
    pub extern "C" fn fp_neighborhood_json(seed: u32, cols: u32, rows: u32) -> *mut u8 {
        leak_payload(neighborhood_json(seed, cols, rows))
    }

    /// The house mesh for `seed` and a style index, in the
    /// binary format `mesh_bytes` documents. Null on an invalid
    /// style index.
    #[unsafe(no_mangle)]
    pub extern "C" fn fp_house_mesh(seed: u32, style: u32) -> *mut u8 {
        leak_payload(house_mesh_bytes(seed, style))
    }

    /// The neighborhood mesh for `seed` and a (columns, rows)
    /// block count, in the binary format `mesh_bytes` documents.
    #[unsafe(no_mangle)]
    pub extern "C" fn fp_neighborhood_mesh(seed: u32, cols: u32, rows: u32) -> *mut u8 {
        leak_payload(neighborhood_mesh_bytes(seed, cols, rows))
    }
}
