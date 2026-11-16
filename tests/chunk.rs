//! Chunk integration tests: derivation, determinism, distinctness,
//! translation, and the neighborhood invariants per chunk.

use floorplan::domain::chunk::{ChunkState, DEFAULT_CHUNK_BLOCKS};
use floorplan::domain::curve::Pt;
use floorplan::domain::layout::Density;
use floorplan::domain::neighborhood::Neighborhood;
use floorplan::domain::placement::{house_corners, house_extent};
use floorplan::domain::polygon::{point_in_poly, polys_overlap};
use floorplan::use_cases::chunk::{CHUNK_SPAN, build_chunk, chunk_origin, chunk_seed};
use floorplan::use_cases::neighborhood::{generate_neighborhood, generate_neighborhood_density};
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

/// Every point of a neighborhood's geometry: street samples,
/// lot outlines, and house corners. Translation and span are
/// properties of the whole neighborhood, not of any one layer.
fn all_points(n: &Neighborhood) -> Vec<Pt> {
    let mut pts: Vec<Pt> = n.streets.iter().flat_map(|s| s.samples.points.iter().copied()).collect();
    pts.extend(n.lots.iter().flat_map(|l| l.outline.iter().copied()));
    for h in &n.houses {
        pts.extend(house_corners(h.center, h.rotation, house_extent(&h.house)));
    }
    pts
}

fn min_xy(n: &Neighborhood) -> (f64, f64) {
    let pts = all_points(n);
    (
        pts.iter().map(|p| p.x).fold(f64::MAX, f64::min),
        pts.iter().map(|p| p.y).fold(f64::MAX, f64::min),
    )
}

fn max_xy(n: &Neighborhood) -> (f64, f64) {
    let pts = all_points(n);
    (
        pts.iter().map(|p| p.x).fold(f64::MIN, f64::max),
        pts.iter().map(|p| p.y).fold(f64::MIN, f64::max),
    )
}

#[test]
fn a_chunk_is_translated_to_its_origin() {
    let (x, z) = chunk_origin(1, -2);
    assert_eq!((x, z), (CHUNK_SPAN, -2.0 * CHUNK_SPAN));
    // A chunk's seed depends on its coordinates, so the moved
    // chunk is compared against its own untranslated
    // neighborhood, not against chunk (0,0).
    let seed = chunk_seed(42, 1, -2);
    let plain = generate_neighborhood_density(seed, DEFAULT_CHUNK_BLOCKS, Density::from_seed(seed));
    let moved = content(42, 1, -2);
    let (px, py) = min_xy(&plain);
    let (mx, my) = min_xy(&moved);
    assert!((mx - (px + x)).abs() < 1e-9, "chunk not translated east");
    assert!((my - (py + z)).abs() < 1e-9, "chunk not translated south");
}

#[test]
fn every_chunk_keeps_the_neighborhood_invariants() {
    for (cx, cz) in [(0, 0), (1, 0), (0, 1), (-2, 3)] {
        let n = content(42, cx, cz);
        let mut seen: HashSet<usize> = HashSet::new();
        for h in &n.houses {
            assert!(seen.insert(h.lot), "chunk ({cx},{cz}): two houses on one lot");
            let lot = &n.lots[h.lot];
            for c in house_corners(h.center, h.rotation, house_extent(&h.house)) {
                assert!(
                    point_in_poly(c, &lot.outline),
                    "chunk ({cx},{cz}): house corner escapes its lot"
                );
            }
        }
        assert_eq!(n.lots.len(), n.houses.len(), "chunk ({cx},{cz}): lot/house mismatch");
        for lot in &n.lots {
            assert!(lot.street < n.streets.len(), "chunk ({cx},{cz}): lot fronts no street");
            let street = &n.streets[lot.street];
            let s0 = lot.center_s - lot.frontage / 2.0;
            let expected = street.samples.point_at(s0).add(
                street.samples.normal_at(s0).scale(4.0 * lot.side as f64),
            );
            assert!(
                lot.outline[0].dist(expected) < 1e-6,
                "chunk ({cx},{cz}): lot front is off its street's offset curve"
            );
        }
        for i in 0..n.houses.len() {
            for j in i + 1..n.houses.len() {
                let ci = house_corners(n.houses[i].center, n.houses[i].rotation, house_extent(&n.houses[i].house));
                let cj = house_corners(n.houses[j].center, n.houses[j].rotation, house_extent(&n.houses[j].house));
                assert!(
                    !polys_overlap(&ci, &cj),
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
        let (right, bottom) = max_xy(&n);
        assert!(right <= CHUNK_SPAN, "seed {seed}: width {right} exceeds the chunk span");
        assert!(bottom <= CHUNK_SPAN, "seed {seed}: depth {bottom} exceeds the chunk span");
    }
}
