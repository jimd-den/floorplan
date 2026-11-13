//! The `extern "C"` surface of the WASM adapter. Compiled only for
//! wasm32; the payload builders it calls are tested natively.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::adapters::wasm::{
    PROTOCOL_VERSION, chunk_json, chunk_mesh_bytes, daylight_json, house_json,
    house_lights_json, house_lights_toggled_json, house_mesh_bytes, light_toggle_state,
    neighborhood_json, neighborhood_mesh_bytes,
};

static LAST_LEN: AtomicU32 = AtomicU32::new(0);

/// Moves a payload into WASM linear memory and returns its pointer;
/// records its length for `fp_last_len`. A refused call returns a
/// null pointer and length 0.
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

/// Allocates `size` bytes in linear memory. The caller owns the
/// pointer and frees it with `fp_free`.
#[unsafe(no_mangle)]
pub extern "C" fn fp_alloc(size: usize) -> *mut u8 {
    Box::leak(vec![0u8; size].into_boxed_slice()).as_mut_ptr()
}

/// Frees a pointer previously returned by `fp_alloc` or a payload
/// export. `size` must be the size that was allocated.
#[unsafe(no_mangle)]
pub extern "C" fn fp_free(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }
    unsafe {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, size));
    }
}

/// The length of the payload the last export returned, in bytes.
/// 0 after a refused call.
#[unsafe(no_mangle)]
pub extern "C" fn fp_last_len() -> u32 {
    LAST_LEN.load(Ordering::Relaxed)
}

/// The protocol version (2).
#[unsafe(no_mangle)]
pub extern "C" fn fp_version() -> u32 {
    PROTOCOL_VERSION
}

/// The house JSON for `seed` and a style index. Null on an invalid
/// style index.
#[unsafe(no_mangle)]
pub extern "C" fn fp_house_json(seed: u32, style: u32) -> *mut u8 {
    leak_payload(house_json(seed, style))
}

/// The neighborhood JSON for `seed` and a (columns, rows) count.
#[unsafe(no_mangle)]
pub extern "C" fn fp_neighborhood_json(seed: u32, cols: u32, rows: u32) -> *mut u8 {
    leak_payload(neighborhood_json(seed, cols, rows))
}

/// The house mesh for `seed` and a style index. Null on an invalid
/// style index.
#[unsafe(no_mangle)]
pub extern "C" fn fp_house_mesh(seed: u32, style: u32) -> *mut u8 {
    leak_payload(house_mesh_bytes(seed, style))
}

/// The neighborhood mesh for `seed` and a (columns, rows) count.
#[unsafe(no_mangle)]
pub extern "C" fn fp_neighborhood_mesh(seed: u32, cols: u32, rows: u32) -> *mut u8 {
    leak_payload(neighborhood_mesh_bytes(seed, cols, rows))
}

/// The JSON for chunk (cx, cz) of the infinite neighborhood. Chunk
/// (0,0) carries the fixed neighborhood's JSON.
#[unsafe(no_mangle)]
pub extern "C" fn fp_chunk_json(seed: u32, cx: i32, cz: i32) -> *mut u8 {
    leak_payload(chunk_json(seed, cx, cz))
}

/// The mesh for chunk (cx, cz) of the infinite neighborhood.
#[unsafe(no_mangle)]
pub extern "C" fn fp_chunk_mesh(seed: u32, cx: i32, cz: i32) -> *mut u8 {
    leak_payload(chunk_mesh_bytes(seed, cx, cz))
}

/// The lights a house owns, as JSON. Null on an invalid style index.
#[unsafe(no_mangle)]
pub extern "C" fn fp_house_lights(seed: u32, style: u32) -> *mut u8 {
    leak_payload(house_lights_json(seed, style))
}

/// The lights after one `ToggleLight` command. Null when the light is
/// missing or broken, or the style index is invalid.
#[unsafe(no_mangle)]
pub extern "C" fn fp_house_lights_toggled(seed: u32, style: u32, light_id: u32) -> *mut u8 {
    leak_payload(house_lights_toggled_json(seed, style, light_id))
}

/// The daylight cycle at `minutes`, held when `held` is non-zero.
#[unsafe(no_mangle)]
pub extern "C" fn fp_daylight(minutes: u32, held: u32) -> *mut u8 {
    leak_payload(Ok(daylight_json(minutes, held != 0)))
}

/// The state a switch produces for a light-state code; -1 when the
/// code is unknown.
#[unsafe(no_mangle)]
pub extern "C" fn fp_light_toggle_state(code: u32) -> i32 {
    light_toggle_state(code).map_or(-1, |next| next as i32)
}
