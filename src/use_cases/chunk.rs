//! chunk: a neighborhood chunk derived only from (world seed, cx, cz).
//!
//! Chunk (0,0) is the fixed 2x2 neighborhood, unchanged. Every other
//! chunk is a 2x2-block neighborhood generated from a seed mixed from
//! the world seed and the coordinates, then translated to its chunk
//! origin. Nothing here reads visit order, wall-clock time, or player
//! state, so a revisit rebuilds byte-identical content.

use crate::domain::chunk::{Chunk, ChunkState, DEFAULT_CHUNK_BLOCKS};
use crate::domain::geometry::Rect;
use crate::domain::neighborhood::Neighborhood;
use crate::use_cases::neighborhood::generate_neighborhood;

/// The world-space spacing between chunk origins, in metres. It is
/// larger than any 2x2 neighborhood, so adjacent chunks never touch.
pub const CHUNK_SPAN: f64 = 400.0;

/// The seed a chunk generates from. Chunk (0,0) keeps the world seed
/// so the fixed neighborhood is unchanged; every other chunk mixes
/// the seed with its coordinates.
pub fn chunk_seed(world: u32, cx: i32, cz: i32) -> u32 {
    if cx == 0 && cz == 0 {
        return world;
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in world
        .to_le_bytes()
        .iter()
        .chain(cx.to_le_bytes().iter())
        .chain(cz.to_le_bytes().iter())
    {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    (hash >> 32) as u32
}

/// The world-space origin of a chunk, in metres.
pub fn chunk_origin(cx: i32, cz: i32) -> (f64, f64) {
    (cx as f64 * CHUNK_SPAN, cz as f64 * CHUNK_SPAN)
}

/// Builds a finished chunk. Deterministic in the world seed and the
/// coordinates; chunk (0,0) is exactly `generate_neighborhood(world,
/// DEFAULT_CHUNK_BLOCKS)`.
pub fn build_chunk(world: u32, cx: i32, cz: i32) -> Chunk {
    let seed = chunk_seed(world, cx, cz);
    let neighborhood = generate_neighborhood(seed, DEFAULT_CHUNK_BLOCKS);
    let (dx, dz) = chunk_origin(cx, cz);
    Chunk {
        cx,
        cz,
        state: ChunkState::Ready,
        neighborhood: Some(translate(neighborhood, dx, dz)),
    }
}

/// Translates every world-space rectangle in a neighborhood. A zero
/// offset returns the neighborhood untouched, which keeps chunk (0,0)
/// byte-identical to the fixed neighborhood.
fn translate(mut n: Neighborhood, dx: f64, dz: f64) -> Neighborhood {
    if dx == 0.0 && dz == 0.0 {
        return n;
    }
    for street in &mut n.streets {
        street.rect = shift(street.rect, dx, dz);
    }
    for lot in &mut n.lots {
        lot.rect = shift(lot.rect, dx, dz);
    }
    for house in &mut n.houses {
        house.rect = shift(house.rect, dx, dz);
    }
    n
}

fn shift(rect: Rect, dx: f64, dz: f64) -> Rect {
    Rect::new(rect.x + dx, rect.y + dz, rect.w, rect.d)
}
