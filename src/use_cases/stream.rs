//! stream: the chunk lifecycle for the infinite neighborhood.
//!
//! A pure state machine over chunks: the shell calls `tick` each
//! frame with the player's chunk coordinates. The ring around the
//! player is requested, at most one chunk is generated, at most two
//! are installed, and chunks beyond the keep radius are evicted.
//! Generation is deterministic, so a revisit rebuilds byte-identical
//! content. No clock and no IO live here.

use std::collections::BTreeMap;

use crate::domain::chunk::{Chunk, ChunkState};
use crate::use_cases::chunk::build_chunk;

/// Chunks generated per tick. One keeps generation work inside the
/// 8 ms/frame budget; the rest of the ring waits.
pub const GENERATION_PER_TICK: usize = 1;
/// Chunks installed per tick (FR-3 TARGET: at most 2).
pub const MAX_INSTALLS_PER_TICK: usize = 2;
/// The ring requested around the player, in chunks.
pub const RING_RADIUS: i32 = 1;
/// Chunks farther than this from the player are released.
pub const KEEP_RADIUS: i32 = 2;

/// The streaming session: the chunks in memory and the request queue.
pub struct Streamer {
    world_seed: u32,
    chunks: BTreeMap<(i32, i32), Chunk>,
    queue: Vec<(i32, i32)>,
}

impl Streamer {
    /// A fresh streamer for a world seed. Nothing is requested yet.
    pub fn new(world_seed: u32) -> Self {
        Self { world_seed, chunks: BTreeMap::new(), queue: Vec::new() }
    }

    /// The chunks currently held, in coordinate order.
    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.values()
    }

    /// How many chunks are held.
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// True when no chunk is held.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// The chunk at these coordinates, if it is held.
    pub fn get(&self, cx: i32, cz: i32) -> Option<&Chunk> {
        self.chunks.get(&(cx, cz))
    }

    /// Queues a chunk without generating it. Returns true when the
    /// chunk was not already held or queued.
    pub fn request(&mut self, cx: i32, cz: i32) -> bool {
        if self.chunks.contains_key(&(cx, cz)) {
            return false;
        }
        self.chunks.insert(
            (cx, cz),
            Chunk { cx, cz, state: ChunkState::Requested, neighborhood: None },
        );
        self.queue.push((cx, cz));
        true
    }

    /// Generates at most `GENERATION_PER_TICK` requested chunks.
    pub fn generate_step(&mut self) -> usize {
        let mut generated = 0;
        while generated < GENERATION_PER_TICK {
            let Some(index) = self.queue.iter().position(|key| {
                self.chunks.get(key).is_some_and(|c| c.state == ChunkState::Requested)
            }) else {
                break;
            };
            let (cx, cz) = self.queue.remove(index);
            self.chunks.insert((cx, cz), build_chunk(self.world_seed, cx, cz));
            generated += 1;
        }
        generated
    }

    /// Installs at most `MAX_INSTALLS_PER_TICK` ready chunks.
    pub fn install_step(&mut self) -> usize {
        let mut installed = 0;
        for chunk in self.chunks.values_mut() {
            if installed == MAX_INSTALLS_PER_TICK {
                break;
            }
            if chunk.state == ChunkState::Ready {
                chunk.state = ChunkState::Installed;
                installed += 1;
            }
        }
        installed
    }

    /// One frame: request the ring, generate, install, then evict.
    pub fn tick(&mut self, player: (i32, i32)) {
        self.request_ring(player);
        self.generate_step();
        self.install_step();
        self.evict_far(player);
    }

    /// True when the chunk is installed.
    pub fn installed(&self, cx: i32, cz: i32) -> bool {
        self.get(cx, cz).is_some_and(|c| c.state == ChunkState::Installed)
    }

    /// True when the chunk is ready or installed.
    pub fn ready(&self, cx: i32, cz: i32) -> bool {
        self.get(cx, cz)
            .is_some_and(|c| matches!(c.state, ChunkState::Ready | ChunkState::Installed))
    }

    fn request_ring(&mut self, (px, pz): (i32, i32)) {
        for dz in -RING_RADIUS..=RING_RADIUS {
            for dx in -RING_RADIUS..=RING_RADIUS {
                self.request(px + dx, pz + dz);
            }
        }
    }

    fn evict_far(&mut self, (px, pz): (i32, i32)) {
        self.chunks.retain(|&(cx, cz), _| {
            (cx - px).abs() <= KEEP_RADIUS && (cz - pz).abs() <= KEEP_RADIUS
        });
        self.queue.retain(|key| self.chunks.contains_key(key));
    }
}
