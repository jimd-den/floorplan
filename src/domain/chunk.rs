//! Chunk: one unit of the infinite neighborhood.
//!
//! A chunk is a 2x2-block neighborhood derived only from the world
//! seed and its coordinates. Chunk (0,0) is the fixed neighborhood
//! the CLI already emits; every other chunk is translated to its
//! own origin. The lifecycle state lives here; the transitions live
//! in `use_cases::stream`.

use crate::domain::neighborhood::Neighborhood;

/// Blocks per chunk. One chunk is one fixed 2x2 neighborhood.
pub const DEFAULT_CHUNK_BLOCKS: (u32, u32) = (2, 2);

/// World-space spacing between chunk origins, in metres (400 m).
pub const CHUNK_SPAN: f64 = 400.0;

/// Where a chunk is in its lifecycle. A chunk is generated off the
/// main path and installed at a metered rate; an evicted chunk is
/// released and will be rebuilt byte-identically on revisit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkState {
    Requested,
    Generating,
    Ready,
    Installed,
    Evicted,
}

impl ChunkState {
    /// Lower-case name for the JSON writer.
    pub fn id(&self) -> &'static str {
        match self {
            ChunkState::Requested => "requested",
            ChunkState::Generating => "generating",
            ChunkState::Ready => "ready",
            ChunkState::Installed => "installed",
            ChunkState::Evicted => "evicted",
        }
    }
}

/// Per-house level of detail (spec §7):
/// - `Full` (LOD 0): relief, trim, multi-part furniture, roof accessories.
/// - `Simplified` (LOD 1): merged trim/relief, single-box furniture, bare roof.
/// - `Massing` (LOD 2): envelope slabs, walls with openings, bare roof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lod {
    Full,
    Simplified,
    Massing,
}

impl Lod {
    /// Distance rule (in metres): `< 80 m` -> `Full` (LOD 0),
    /// `80..=200 m` -> `Simplified` (LOD 1), `> 200 m` -> `Massing` (LOD 2).
    pub fn for_distance(meters: f64) -> Self {
        if meters < 80.0 {
            Lod::Full
        } else if meters <= 200.0 {
            Lod::Simplified
        } else {
            Lod::Massing
        }
    }
}

/// One chunk: its coordinates, lifecycle state, and content. Content
/// is `None` until generation finishes, so a requested chunk holds
/// no mesh or JSON.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub cx: i32,
    pub cz: i32,
    pub state: ChunkState,
    pub neighborhood: Option<Neighborhood>,
}
