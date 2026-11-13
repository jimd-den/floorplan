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
