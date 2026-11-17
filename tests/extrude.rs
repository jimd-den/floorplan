//! Extrusion: a generated house becomes a valid 3D mesh.
//!
//! These are integration tests — each builds a whole house with
//! `generate_house` and extrudes it — so they live here, not in
//! the `#[cfg(test)]` module of `use_cases::extrude`. Moved out
//! when that file passed the line ceiling.

use floorplan::domain::basement::BASEMENT_SLAB;
use floorplan::domain::style::Style;
use floorplan::use_cases::extrude::{ROOF, STAIRS, extrude};
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::materials::{CEILING, CONCRETE};
use floorplan::use_cases::walls::STOREY_H;

#[test]
fn extrude_is_deterministic() {
    let a = extrude(&generate_house(42, Style::Ranch));
    let b = extrude(&generate_house(42, Style::Ranch));
    assert_eq!(a.vertices, b.vertices);
    assert_eq!(a.faces, b.faces);
    assert_eq!(a.materials, b.materials);
}

#[test]
fn every_extruded_house_is_a_valid_mesh() {
    for seed in [0u32, 7, 42, 106, 199] {
        for style in Style::all() {
            let mesh = extrude(&generate_house(seed, style));
            assert_eq!(mesh.validate(), Ok(()), "seed {seed} {style:?}");
        }
    }
}

#[test]
fn a_two_storey_house_has_stairs_reaching_the_upper_floor() {
    let house = generate_house(7, Style::Colonial);
    let mesh = extrude(&house);
    let stair_faces: Vec<_> = mesh
        .faces
        .iter()
        .zip(mesh.materials.iter())
        .filter(|(_, m)| **m == STAIRS)
        .collect();
    assert!(!stair_faces.is_empty(), "no stairs geometry");
    let top = stair_faces
        .iter()
        .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
        .fold(0.0f32, f32::max);
    assert!((top - STOREY_H).abs() < 0.01, "stairs top at {top}, not {STOREY_H}");
}

#[test]
fn a_one_storey_house_has_no_stairs() {
    // A one-storey house has no stair UP to another floor. It
    // still has the basement stair down from the hall, which is
    // entirely at or below floor level — so the assertion is on
    // the stair's height, not its presence.
    let mesh = extrude(&generate_house(7, Style::Ranch));
    let top = mesh
        .faces
        .iter()
        .zip(mesh.materials.iter())
        .filter(|(_, m)| **m == STAIRS)
        .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(top <= 0.0, "a stair rises above floor level to {top}");
}

#[test]
fn the_top_floor_has_an_interior_ceiling_and_a_pitched_roof() {
    let mesh = extrude(&generate_house(7, Style::Colonial));
    assert!(mesh.materials.contains(&ROOF), "no pitched roof");
    assert!(mesh.materials.contains(&CEILING), "no interior ceiling");
    let roof_top = mesh.faces.iter().zip(mesh.materials.iter())
        .filter(|(_, m)| **m == ROOF)
        .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
        .fold(0.0f32, f32::max);
    assert!(roof_top > STOREY_H, "the roof does not rise above the top storey");
}

#[test]
fn the_mesh_carries_a_concrete_foundation() {
    let house = generate_house(42, Style::Modern);
    let mesh = extrude(&house);
    assert!(mesh.material_table.iter().any(|m| m.name == "concrete"), "no concrete material");
    let ys: Vec<f32> = mesh
        .faces
        .iter()
        .zip(mesh.materials.iter())
        .filter(|(_, m)| **m == CONCRETE)
        .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
        .collect();
    assert!(!ys.is_empty(), "no concrete faces");
    let low = ys.iter().cloned().fold(f32::INFINITY, f32::min);
    let high = ys.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    // The base slab bottom is 0.3 m below floor level; the
    // basement slab bottom is deeper still, so the slab is
    // asserted by presence, not by being the lowest concrete.
    assert!(
        ys.iter().any(|&y| (y + 0.3).abs() < 0.01),
        "no concrete at the slab bottom (-0.3 m)"
    );
    let expect_low = -(house.basement.depth as f32) - BASEMENT_SLAB as f32;
    assert!(
        (low - expect_low).abs() < 0.01,
        "deepest concrete {low} is not the basement slab bottom {expect_low}"
    );
    // The slab top is floor level (y=0); the stem wall rises 0.2 m
    // above grade, so it — not the slab — is the highest concrete.
    assert!(ys.iter().any(|&y| y.abs() < 0.01), "no concrete at the slab top (floor level)");
    assert!((high - 0.2).abs() < 0.01, "the highest concrete {high} is not the stem wall top (0.2 m)");
}

#[test]
fn an_exposed_ground_ceiling_is_an_interior_ceiling() {
    // A Colonial with a garage: no upper room sits above the
    // garage, so its ceiling slab at 2.7 m is an interior
    // ceiling, and the pitched roof is above the top storey.
    for seed in 0..50u32 {
        let house = generate_house(seed, Style::Colonial);
        if house.footprint.garage.is_none() {
            continue;
        }
        let mesh = extrude(&house);
        let ys: Vec<f32> = mesh.faces.iter().zip(mesh.materials.iter())
            .filter(|(_, m)| **m == CEILING)
            .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
            .collect();
        assert!(ys.iter().any(|&y| (y - 2.7).abs() < 0.01), "seed {seed}: no ceiling at 2.7 m");
        return;
    }
    panic!("no Colonial with a garage in 50 seeds");
}
