//! Wall relief and casing tests (spec §5, §10c):
//! - Exterior walls have real depth relief (overlapping siding courses,
//!   brick coursing with recessed mortar, stone blocks, stucco trims).
//! - Interior walls have baseboards, chair rails, crown moulding, and
//!   door/window casings.
//! - Every wall face has relief depth > 0 somewhere.

use floorplan::domain::style::Style;
use floorplan::use_cases::extrude::extrude;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::relief::wall_relief_depth;
use floorplan::use_cases::walls::{STOREY_H, wall_segments_toned};

#[test]
fn every_wall_face_has_relief_depth_above_zero() {
    for seed in 0..20u32 {
        for style in Style::all() {
            let house = generate_house(seed, style);
            let mesh = extrude(&house);
            for floor in &house.floors {
                let base = floor.level as f32 * STOREY_H;
                for (seg, exterior) in wall_segments_toned(&floor.rooms) {
                    let depth = wall_relief_depth(&mesh, &seg, base, exterior);
                    assert!(
                        depth > 0.0,
                        "seed {seed} {style:?} floor {} wall {seg:?} (exterior={exterior}) has 0 relief depth",
                        floor.level
                    );
                }
            }
        }
    }
}

#[test]
fn interior_rooms_have_baseboard_chair_rail_and_crown_moulding() {
    let house = generate_house(42, Style::Colonial);
    let mesh = extrude(&house);
    let room = house.floors[0]
        .rooms
        .iter()
        .find(|r| r.program == floorplan::domain::model::Program::Living)
        .unwrap();
    let z = room.rect.y as f32;
    let has_at_y = |y_lo: f32, y_hi: f32| {
        mesh.vertices.iter().any(|v| {
            v[0] >= room.rect.x as f32 - 0.05
                && v[0] <= room.rect.right() as f32 + 0.05
                && v[1] >= y_lo
                && v[1] <= y_hi
                && (v[2] - z).abs() >= 0.085
                && (v[2] - z).abs() <= 0.12
        })
    };
    assert!(has_at_y(0.05, 0.12), "missing baseboard relief");
    assert!(has_at_y(0.85, 0.98), "missing chair rail / wainscoting relief");
    assert!(has_at_y(2.58, 2.70), "missing crown moulding relief");
}
