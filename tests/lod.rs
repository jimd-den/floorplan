//! Level of Detail (LOD) tests (spec §7, §10f):
//! - LOD 0: full detail (relief, trim, detailed furniture, roof accessories).
//! - LOD 1: simplified (trim/relief merged, furniture as boxes, bare roof).
//! - LOD 2: massing only (envelope, roof, openings).
//! - Switchable at runtime from the same `House` plan without changing
//!   `plan_hash`.
//! - Distance rule selects LOD 0 near (< 80 m), LOD 1 mid (80–200 m),
//!   and LOD 2 far (> 200 m), with chunk span = 400 m.

use floorplan::domain::chunk::CHUNK_SPAN;
use floorplan::domain::mesh::Lod;
use floorplan::domain::style::Style;
use floorplan::use_cases::extrude::{extrude, extrude_lod};
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::variety::plan_hash;

#[test]
fn lod_zero_one_two_extrude_from_same_plan_with_unchanged_plan_hash() {
    for style in Style::all() {
        let house = generate_house(42, style);
        let before = plan_hash(&house);
        let m0 = extrude_lod(&house, Lod::Full);
        let m1 = extrude_lod(&house, Lod::Simplified);
        let m2 = extrude_lod(&house, Lod::Massing);
        let after = plan_hash(&house);
        assert_eq!(before, after, "extrude_lod mutated or changed plan hash");
        assert_eq!(m0.vertices, extrude(&house).vertices, "Lod::Full must match default extrude");
        assert_eq!(m0.validate(), Ok(()));
        assert_eq!(m1.validate(), Ok(()));
        assert_eq!(m2.validate(), Ok(()));
        assert!(
            m0.faces.len() > m1.faces.len(),
            "{style:?}: LOD 0 ({}) should have more faces than LOD 1 ({})",
            m0.faces.len(),
            m1.faces.len()
        );
        assert!(
            m1.faces.len() > m2.faces.len(),
            "{style:?}: LOD 1 ({}) should have more faces than LOD 2 ({})",
            m1.faces.len(),
            m2.faces.len()
        );
    }
}

#[test]
fn distance_rule_selects_high_lod_near_and_low_lod_far() {
    assert_eq!(CHUNK_SPAN, 400.0);
    assert_eq!(Lod::for_distance(0.0), Lod::Full);
    assert_eq!(Lod::for_distance(75.0), Lod::Full);
    assert_eq!(Lod::for_distance(120.0), Lod::Simplified);
    assert_eq!(Lod::for_distance(250.0), Lod::Massing);
}
