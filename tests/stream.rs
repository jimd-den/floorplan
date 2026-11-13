//! Streaming integration tests: the chunk lifecycle, metered installs,
//! the ring ahead, eviction, byte-identical revisits, and a bounded
//! working set.

use floorplan::domain::chunk::ChunkState;
use floorplan::use_cases::stream::{
    KEEP_RADIUS, MAX_INSTALLS_PER_TICK, Streamer,
};
use floorplan::use_cases::variety::neighborhood_hash;

fn hash_of(streamer: &Streamer, cx: i32, cz: i32) -> u64 {
    let chunk = streamer.get(cx, cz).expect("chunk not held");
    let n = chunk.neighborhood.as_ref().expect("chunk has no content");
    neighborhood_hash(n)
}

#[test]
fn requesting_a_chunk_does_not_generate_it() {
    let mut streamer = Streamer::new(42);
    assert!(streamer.request(1, 0));
    let chunk = streamer.get(1, 0).expect("requested chunk is held");
    assert_eq!(chunk.state, ChunkState::Requested);
    assert!(chunk.neighborhood.is_none(), "request must not generate");
    assert!(!streamer.request(1, 0), "a queued chunk is not queued twice");
}

#[test]
fn generate_step_builds_one_requested_chunk() {
    let mut streamer = Streamer::new(42);
    streamer.request(1, 0);
    assert_eq!(streamer.generate_step(), 1);
    let chunk = streamer.get(1, 0).unwrap();
    assert_eq!(chunk.state, ChunkState::Ready);
    assert!(chunk.neighborhood.is_some());
}

#[test]
fn installs_are_metered_at_two_per_tick() {
    let mut streamer = Streamer::new(42);
    for (cx, cz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        streamer.request(cx, cz);
    }
    for _ in 0..4 {
        streamer.generate_step();
    }
    assert_eq!(streamer.install_step(), MAX_INSTALLS_PER_TICK);
    let installed = streamer
        .chunks()
        .filter(|c| c.state == ChunkState::Installed)
        .count();
    assert_eq!(installed, 2, "a tick must install at most two chunks");
}

#[test]
fn tick_installs_the_ring_ahead() {
    let mut streamer = Streamer::new(42);
    for _ in 0..10 {
        streamer.tick((0, 0));
    }
    for (cx, cz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        assert!(streamer.installed(cx, cz), "ring chunk ({cx},{cz}) not installed");
    }
}

#[test]
fn the_chunk_about_to_be_entered_is_installed_before_arrival() {
    let mut streamer = Streamer::new(42);
    for _ in 0..10 {
        streamer.tick((0, 0));
    }
    // The player walks east into (1,0). It is already installed
    // before the first tick at the new position.
    assert!(streamer.installed(1, 0));
    streamer.tick((1, 0));
    assert!(streamer.installed(1, 0));
}

#[test]
fn walking_away_evicts_chunks_behind() {
    let mut streamer = Streamer::new(42);
    for _ in 0..4 {
        streamer.tick((0, 0));
    }
    assert!(streamer.get(0, 0).is_some(), "origin should be held");
    streamer.tick((10, 0));
    assert!(streamer.get(0, 0).is_none(), "distant chunk was not evicted");
}

#[test]
fn revisiting_a_chunk_regenerates_identical_content() {
    let mut streamer = Streamer::new(42);
    for _ in 0..10 {
        streamer.tick((0, 0));
    }
    let first = hash_of(&streamer, 0, 0);
    streamer.tick((10, 0));
    assert!(streamer.get(0, 0).is_none(), "chunk should have been evicted");
    for _ in 0..10 {
        streamer.tick((0, 0));
    }
    assert_eq!(hash_of(&streamer, 0, 0), first, "revisit is not byte-identical");
}

#[test]
fn memory_plateaus_after_the_working_set_is_full() {
    let mut streamer = Streamer::new(42);
    for step in 0..40 {
        streamer.tick((step, 0));
    }
    let ring = ((2 * KEEP_RADIUS + 1) * (2 * KEEP_RADIUS + 1)) as usize;
    assert!(
        streamer.len() <= ring,
        "held {} chunks, working set is {ring}",
        streamer.len()
    );
}
