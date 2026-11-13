//! Integration: determinism, reachability and variety across seeds.

use floorplan::adapters::json::render_json;
use floorplan::adapters::svg::render_svg;
use floorplan::domain::model::{House, OpeningKind, Program};
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::variety::plan_hash;
use std::collections::HashSet;

fn style_for(i: u32) -> Style {
    Style::all()[(i % 6) as usize]
}

#[test]
fn same_seed_is_byte_identical_svg_and_json() {
    for seed in [0u32, 1, 42, 999, u32::MAX] {
        for style in Style::all() {
            let a = generate_house(seed, style);
            let b = generate_house(seed, style);
            assert_eq!(render_svg(&a), render_svg(&b), "svg differs for {seed} {style:?}");
            assert_eq!(render_json(&a), render_json(&b), "json differs for {seed} {style:?}");
        }
    }
}

#[test]
fn every_room_is_reachable_from_the_entry_for_200_seeds() {
    for seed in 0..200u32 {
        let house = generate_house(seed, style_for(seed));
        assert!(reachable(&house), "seed {seed} has unreachable rooms");
    }
}

/// BFS over all rooms: door edges per floor plus the stair edge.
fn reachable(house: &House) -> bool {
    let mut edges: Vec<(u32, u32)> = vec![];
    for floor in &house.floors {
        for o in floor.openings.iter().filter(|o| o.kind == OpeningKind::Door) {
            if let Some(to) = o.to {
                edges.push((o.from, to));
            }
        }
    }
    if let Some((a, b)) = house.stairs {
        edges.push((a, b));
    }
    let start = house.floors[0]
        .rooms
        .iter()
        .find(|r| r.program == Program::Entry)
        .or_else(|| house.floors[0].rooms.iter().find(|r| r.program == Program::Living))
        .map(|r| r.id);
    let Some(start) = start else { return true };
    let mut seen = HashSet::from([start]);
    let mut stack = vec![start];
    while let Some(cur) = stack.pop() {
        for &(a, b) in &edges {
            let next = if a == cur { Some(b) } else if b == cur { Some(a) } else { None };
            if let Some(n) = next
                && !seen.contains(&n)
            {
                seen.insert(n);
                stack.push(n);
            }
        }
    }
    let total: usize = house.floors.iter().map(|f| f.rooms.len()).sum();
    seen.len() == total
}

#[test]
fn a_thousand_seeds_give_a_thousand_distinct_plans() {
    let mut hashes = HashSet::new();
    for seed in 0..1000u32 {
        let house = generate_house(seed, style_for(seed));
        hashes.insert(plan_hash(&house));
    }
    assert_eq!(hashes.len(), 1000, "only {} distinct plans in 1000 seeds", hashes.len());
}

#[test]
fn no_room_exceeds_its_program_cap_for_200_seeds() {
    // M0 re-tiling: bath <= 6 sq m, study <= 12 sq m, every seed.
    for seed in 0..200u32 {
        let house = generate_house(seed, style_for(seed));
        for floor in &house.floors {
            for room in &floor.rooms {
                match room.program {
                    Program::Bath => assert!(
                        room.rect.area() <= 6.0 + 1e-9,
                        "seed {seed} bath room {} is {:.1} sq m",
                        room.id,
                        room.rect.area()
                    ),
                    Program::Study => assert!(
                        room.rect.area() <= 12.0 + 1e-9,
                        "seed {seed} study room {} is {:.1} sq m",
                        room.id,
                        room.rect.area()
                    ),
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn every_habitable_room_has_a_window_for_200_seeds() {
    for seed in 0..200u32 {
        let house = generate_house(seed, style_for(seed));
        for floor in &house.floors {
            let windowed: HashSet<u32> = floor
                .openings
                .iter()
                .filter(|o| o.kind == OpeningKind::Window)
                .map(|o| o.from)
                .collect();
            for room in floor.rooms.iter().filter(|r| r.program.is_habitable()) {
                assert!(
                    windowed.contains(&room.id),
                    "seed {seed} {:?} room {} has no window",
                    room.program,
                    room.id
                );
            }
        }
    }
}
