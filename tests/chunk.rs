//! Chunk integration tests: derivation, determinism, distinctness,
//! translation, and the neighborhood invariants per chunk.

use floorplan::domain::chunk::{ChunkState, DEFAULT_CHUNK_BLOCKS};
use floorplan::domain::neighborhood::Neighborhood;
use floorplan::use_cases::chunk::{CHUNK_SPAN, build_chunk, chunk_origin, chunk_seed};
use floorplan::use_cases::neighborhood::generate_neighborhood;
use floorplan::use_cases::variety::neighborhood_hash;
use std::collections::HashSet;

fn content(seed: u32, cx: i32, cz: i32) -> Neighborhood {
    build_chunk(seed, cx, cz).neighborhood.expect("a built chunk has content")
}

#[test]
fn chunk_zero_zero_is_the_fixed_neighborhood() {
    let fixed = generate_neighborhood(42, DEFAULT_CHUNK_BLOCKS);
    assert_eq!(content(42, 0, 0), fixed);
}

#[test]
fn a_built_chunk_starts_ready() {
    assert_eq!(build_chunk(42, 1, 0).state, ChunkState::Ready);
}

#[test]
fn the_chunk_seed_depends_on_the_coordinates() {
    let seeds: HashSet<u32> = [(0, 0), (1, 0), (0, 1), (-1, 2), (7, -3)]
        .iter()
        .map(|&(cx, cz)| chunk_seed(42, cx, cz))
        .collect();
    assert_eq!(seeds.len(), 5, "coordinates must give distinct seeds");
    assert_eq!(chunk_seed(42, 0, 0), 42, "the origin keeps the world seed");
}

#[test]
fn chunk_content_is_deterministic() {
    assert_eq!(content(42, 2, -1), content(42, 2, -1));
    assert_ne!(content(42, 2, -1), content(43, 2, -1));
}

#[test]
fn distinct_chunks_have_distinct_content_hashes() {
    let coords = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1), (2, 3), (-4, 5), (9, -2)];
    let mut hashes = HashSet::new();
    for (cx, cz) in coords {
        assert!(
            hashes.insert(neighborhood_hash(&content(42, cx, cz))),
            "chunk ({cx},{cz}) repeats an earlier neighborhood"
        );
    }
}

#[test]
fn a_chunk_is_translated_to_its_origin() {
    let (x, z) = chunk_origin(1, -2);
    assert_eq!((x, z), (CHUNK_SPAN, -2.0 * CHUNK_SPAN));
    let n = content(42, 1, -2);
    let min_x = n.streets.iter().map(|s| s.rect.x).fold(f64::MAX, f64::min);
    let min_y = n.streets.iter().map(|s| s.rect.y).fold(f64::MAX, f64::min);
    assert!((min_x - x).abs() < 1e-9, "chunk not translated east");
    assert!((min_y - z).abs() < 1e-9, "chunk not translated south");
}

#[test]
fn every_chunk_keeps_the_neighborhood_invariants() {
    for (cx, cz) in [(0, 0), (1, 0), (0, 1), (-2, 3)] {
        let n = content(42, cx, cz);
        let mut seen: HashSet<usize> = HashSet::new();
        for h in &n.houses {
            assert!(seen.insert(h.lot), "chunk ({cx},{cz}): two houses on one lot");
            let lot = n.lots[h.lot].rect;
            assert!(
                h.rect.x >= lot.x - 1e-9
                    && h.rect.right() <= lot.right() + 1e-9
                    && h.rect.y >= lot.y - 1e-9
                    && h.rect.bottom() <= lot.bottom() + 1e-9,
                "chunk ({cx},{cz}): house outside its lot"
            );
        }
        assert_eq!(n.lots.len(), n.houses.len(), "chunk ({cx},{cz}): lot/house mismatch");
        for lot in &n.lots {
            let s = &n.streets[lot.street];
            let touches = (lot.rect.y - s.rect.bottom()).abs() < 1e-9
                || (lot.rect.bottom() - s.rect.y).abs() < 1e-9
                || (lot.rect.x - s.rect.right()).abs() < 1e-9
                || (lot.rect.right() - s.rect.x).abs() < 1e-9;
            assert!(touches, "chunk ({cx},{cz}): lot without street frontage");
        }
        for i in 0..n.houses.len() {
            for j in i + 1..n.houses.len() {
                assert!(
                    !n.houses[i].rect.overlaps(&n.houses[j].rect),
                    "chunk ({cx},{cz}): houses overlap"
                );
            }
        }
    }
}

#[test]
fn a_neighborhood_fits_inside_one_chunk_span() {
    for seed in 0..100u32 {
        let n = generate_neighborhood(seed, DEFAULT_CHUNK_BLOCKS);
        let right = n
            .streets
            .iter()
            .map(|s| &s.rect)
            .chain(n.lots.iter().map(|l| &l.rect))
            .map(|r| r.right())
            .fold(0.0f64, f64::max);
        let bottom = n
            .streets
            .iter()
            .map(|s| &s.rect)
            .chain(n.lots.iter().map(|l| &l.rect))
            .map(|r| r.bottom())
            .fold(0.0f64, f64::max);
        assert!(right <= CHUNK_SPAN, "seed {seed}: width {right} exceeds the chunk span");
        assert!(bottom <= CHUNK_SPAN, "seed {seed}: depth {bottom} exceeds the chunk span");
    }
}
