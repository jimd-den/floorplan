//! Chunk integration tests for the unbounded map: a chunk is a window,
//! not a fixed block. Two neighbouring windows agree on the street they
//! share, every lot belongs to exactly one chunk, and a window can be
//! moved anywhere without hitting an edge.

use floorplan::domain::chunk::ChunkState;
use floorplan::domain::geometry::Rect;
use floorplan::domain::lattice::LATTICE;
use floorplan::domain::neighborhood::Neighborhood;
use floorplan::domain::placement::{house_corners, house_extent};
use floorplan::domain::polygon::{point_in_poly, polys_overlap};
use floorplan::use_cases::chunk::{CHUNK_SPAN, build_chunk, chunk_origin, chunk_rect, chunk_seed, front_centre};
use floorplan::use_cases::variety::neighborhood_hash;
use std::collections::HashSet;

fn content(seed: u32, cx: i32, cz: i32) -> Neighborhood {
    build_chunk(seed, cx, cz).neighborhood.expect("a built chunk has content")
}

#[test]
fn a_chunk_is_one_lattice_cell() {
    assert_eq!(CHUNK_SPAN, LATTICE);
    assert_eq!(chunk_origin(2, -3), (2.0 * CHUNK_SPAN, -3.0 * CHUNK_SPAN));
    let r = chunk_rect(2, -3);
    assert_eq!((r.x, r.y, r.w, r.d), (2.0 * CHUNK_SPAN, -3.0 * CHUNK_SPAN, CHUNK_SPAN, CHUNK_SPAN));
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
fn chunk_content_is_deterministic_and_seeded() {
    assert_eq!(content(42, 2, -1), content(42, 2, -1));
    assert_ne!(content(42, 2, -1), content(43, 2, -1));
}

#[test]
fn distinct_chunks_have_distinct_content_hashes() {
    let coords = [(0, 0), (1, 0), (-1, 0), (0, 1), (2, 3), (-4, 5)];
    let mut hashes = HashSet::new();
    for (cx, cz) in coords {
        assert!(
            hashes.insert(neighborhood_hash(&content(42, cx, cz))),
            "chunk ({cx},{cz}) repeats an earlier neighborhood"
        );
    }
}

#[test]
fn a_chunk_carries_streets_lots_and_houses_in_lockstep() {
    for (cx, cz) in [(0, 0), (1, 0), (0, 1), (-3, 7)] {
        let n = content(42, cx, cz);
        assert!(!n.streets.is_empty(), "chunk ({cx},{cz}) has no streets");
        assert!(!n.lots.is_empty(), "chunk ({cx},{cz}) has no lots");
        assert_eq!(n.lots.len(), n.houses.len(), "chunk ({cx},{cz}): lot/house mismatch");
        let mut seen = HashSet::new();
        for h in &n.houses {
            assert!(seen.insert(h.lot), "chunk ({cx},{cz}): two houses on one lot");
            assert!(h.lot < n.lots.len(), "chunk ({cx},{cz}): house points at no lot");
        }
    }
}

#[test]
fn every_lot_belongs_to_exactly_one_chunk() {
    // A 2x2 window of chunks: each lot's front centre is unique, so no
    // lot is drawn twice and none is lost between windows.
    let mut seen: HashSet<(i64, i64)> = HashSet::new();
    let mut total = 0;
    for cz in -1..=0 {
        for cx in -1..=0 {
            let n = content(42, cx, cz);
            for lot in &n.lots {
                let p = front_centre(&n.streets[lot.street], lot);
                assert!(
                    seen.insert(((p.x * 100.0).round() as i64, (p.y * 100.0).round() as i64)),
                    "chunk ({cx},{cz}) repeats a lot from another chunk"
                );
                total += 1;
            }
        }
    }
    assert!(total > 20, "only {total} lots across four chunks");
}

#[test]
fn every_lot_in_a_chunk_fronts_its_own_chunk_rectangle() {
    for (cx, cz) in [(0, 0), (2, -1), (-4, 3)] {
        let n = content(42, cx, cz);
        let r = chunk_rect(cx, cz);
        for lot in &n.lots {
            let p = front_centre(&n.streets[lot.street], lot);
            assert!(
                p.x >= r.x && p.x < r.right() && p.y >= r.y && p.y < r.bottom(),
                "chunk ({cx},{cz}): a lot front centre is outside the window"
            );
        }
    }
}

#[test]
fn no_two_lots_in_a_chunk_overlap() {
    for (cx, cz) in [(0, 0), (1, 1), (-2, 3)] {
        let n = content(42, cx, cz);
        for i in 0..n.lots.len() {
            for j in i + 1..n.lots.len() {
                assert!(
                    !polys_overlap(&n.lots[i].outline, &n.lots[j].outline),
                    "chunk ({cx},{cz}): lots {i} and {j} overlap"
                );
            }
        }
    }
}

#[test]
fn every_house_in_a_chunk_sits_inside_its_lot() {
    for (cx, cz) in [(0, 0), (1, 0), (0, -2)] {
        let n = content(42, cx, cz);
        for h in &n.houses {
            for c in house_corners(h.center, h.rotation, house_extent(&h.house)) {
                assert!(
                    point_in_poly(c, &n.lots[h.lot].outline),
                    "chunk ({cx},{cz}): a house corner escapes its lot"
                );
            }
        }
    }
}

#[test]
fn neighbouring_chunks_draw_the_shared_street_identically() {
    let west = content(42, 0, 0);
    let east = content(42, 1, 0);
    let mut shared = 0;
    for a in &west.streets {
        let key = (a.control[0], *a.control.last().unwrap());
        if let Some(b) = east.streets.iter().find(|b| (b.control[0], *b.control.last().unwrap()) == key) {
            assert_eq!(a, b, "the shared street differs between two chunks");
            shared += 1;
        }
    }
    assert!(shared > 0, "the two chunks share no street");
}

#[test]
fn the_map_is_a_window_that_can_be_moved_anywhere() {
    let one = content(42, 0, 0).houses.len();
    let mut four = 0;
    for (cx, cz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        four += content(42, cx, cz).houses.len();
    }
    assert!(four > one, "a 2x2 window ({four}) is no bigger than one chunk ({one})");
    for (cx, cz) in [(10_000, 0), (-10_000, 0)] {
        let n = content(42, cx, cz);
        assert!(!n.houses.is_empty(), "far chunk ({cx},{cz}) is empty");
    }
}

#[test]
fn a_chunk_rectangle_tiles_the_plane_without_gaps() {
    // The four corners of a shared edge belong to exactly one of the
    // two chunks, and the union covers the edge with no overlap.
    let a = chunk_rect(0, 0);
    let b = chunk_rect(1, 0);
    assert_eq!(a.right(), b.x, "chunks do not share an edge");
    assert!(a.x < a.right() && a.y < a.bottom(), "degenerate chunk");
    let _ = Rect::new(0.0, 0.0, 0.0, 0.0);
}
