//! Variety bounds and Clean Architecture dependency-rule verification.
//!
//! Proves that the generator produces millions (in fact billions) of
//! distinct houses and hundreds of millions of distinct neighborhoods,
//! that the hashes are purely structural (seed-free), and that no
//! source file in `domain/` or `use_cases/` imports an outer layer.

use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::neighborhood::generate_neighborhood;
use floorplan::use_cases::variety::{
    house_variety_lower_bound, neighborhood_hash, neighborhood_variety_lower_bound, plan_hash,
};

#[test]
fn house_variety_lower_bound_exceeds_millions() {
    let bound = house_variety_lower_bound();
    assert!(bound >= 1_000_000, "house bound {bound} < 1,000,000");
    assert_eq!(bound, 4_463_424_000);
}

#[test]
fn neighborhood_variety_lower_bound_exceeds_millions() {
    let bound = neighborhood_variety_lower_bound();
    assert!(bound >= 1_000_000, "neighborhood bound {bound} < 1,000,000");
    assert_eq!(bound, 332_474_904);
}

#[test]
fn plan_hash_ignores_the_seed_field() {
    let mut a = generate_house(42, Style::Modern);
    let mut b = a.clone();
    b.seed = 999_999;
    assert_eq!(plan_hash(&a), plan_hash(&b));
    a.floors[0].rooms[0].rect.w += 0.5;
    assert_ne!(plan_hash(&a), plan_hash(&b));
}

#[test]
fn neighborhood_hash_ignores_the_seed_fields() {
    let a = generate_neighborhood(42, (2, 2));
    let mut b = a.clone();
    b.seed = 123_456;
    for h in &mut b.houses {
        h.house.seed = 789_012;
    }
    assert_eq!(neighborhood_hash(&a), neighborhood_hash(&b));
}

#[test]
fn domain_and_use_cases_never_import_outer_layers() {
    for entry in std::fs::read_dir("src/domain").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for line in text.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            assert!(
                !trimmed.contains("crate::use_cases")
                    && !trimmed.contains("crate::adapters")
                    && !trimmed.contains("crate::frameworks_drivers"),
                "{path:?} imports an outer layer: {trimmed}"
            );
        }
    }
    for entry in std::fs::read_dir("src/use_cases").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for line in text.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            assert!(
                !trimmed.contains("crate::adapters"),
                "{path:?} imports adapters: {trimmed}"
            );
        }
    }
}
