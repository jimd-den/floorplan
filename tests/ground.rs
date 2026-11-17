//! Ground-floor assignment behaviour, as integration tests so
//! the implementation file stays under the 200-line ceiling.

use floorplan::domain::geometry::Rect;
use floorplan::domain::model::Program;
use floorplan::use_cases::programs::assign_programs;

#[test]
fn kitchen_takes_a_habitable_leaf_not_a_sub_floor_neighbour() {
    // ADR 0015 (seeds 142/178 Cottage): the leaves next to
    // living were 6.24 sq m strip leaves, so the kitchen took
    // one and the choke turned it into laundry. The habitable
    // leaves sat elsewhere, not next to living. The kitchen
    // must take the nearest habitable leaf instead.
    let leaves = vec![
        Rect::new(0.0, 0.0, 4.0, 4.0),
        Rect::new(4.0, 0.0, 2.6, 2.4),
        Rect::new(0.0, 4.0, 2.6, 2.4),
        Rect::new(6.6, 0.0, 4.0, 4.0),
    ];
    let got = assign_programs(&leaves, 0, false);
    let kitchen = got.iter().position(|p| *p == Program::Kitchen).expect("a kitchen");
    assert!(
        leaves[kitchen].area() >= 6.5 - 1e-9,
        "kitchen leaf is {:.2} sq m",
        leaves[kitchen].area()
    );
}

#[test]
fn dining_does_not_take_the_last_habitable_leaf() {
    // ADR 0015: dining is optional, a bedroom is not. When the
    // only habitable leaf left after living and kitchen is the
    // dining candidate, dining skips it and the bed/bath rule
    // takes it as a bedroom.
    let leaves = vec![
        Rect::new(0.0, 0.0, 4.0, 4.0),
        Rect::new(4.0, 0.0, 4.0, 4.0),
        Rect::new(8.0, 0.0, 4.0, 4.0),
        Rect::new(0.0, 4.0, 2.4, 2.5),
        Rect::new(2.4, 4.0, 2.4, 2.5),
        Rect::new(4.8, 4.0, 2.4, 2.5),
        Rect::new(7.2, 4.0, 2.4, 2.5),
    ];
    let got = assign_programs(&leaves, 0, false);
    assert_eq!(got[2], Program::Bedroom, "leaf 2 should be the bedroom, got {:?}", got[2]);
    assert!(!got.contains(&Program::Dining), "dining took the last habitable leaf");
}

#[test]
fn the_entry_prefers_a_narrow_southernmost_leaf() {
    // ADR 0015 (seed 77 Townhouse round 1): one bath strip
    // plus a 4.8 m rest. The southernmost rest leaf is 4.8 x
    // 2.6 m (12.48 sq m, room-sized) and the southernmost
    // strip leaf is 2.4 x 2.5 m (6.0 sq m, a foyer). The
    // entry must take the narrow leaf, not the wide one —
    // otherwise the re-tiling loop pins a second strip for
    // it and destroys the rest.
    let leaves = vec![
        Rect::new(0.0, 0.0, 2.4, 2.5),
        Rect::new(0.0, 2.5, 2.4, 2.5),
        Rect::new(0.0, 5.0, 2.4, 2.5),
        Rect::new(0.0, 7.5, 2.4, 2.5),
        Rect::new(0.0, 10.0, 2.4, 2.5),
        Rect::new(2.4, 0.0, 4.8, 2.4),
        Rect::new(2.4, 2.4, 4.8, 2.5),
        Rect::new(2.4, 4.9, 4.8, 2.6),
        Rect::new(2.4, 7.5, 4.8, 2.4),
        Rect::new(2.4, 9.9, 4.8, 2.6),
    ];
    let got = assign_programs(&leaves, 0, false);
    let entry = got.iter().position(|p| *p == Program::Entry).expect("an entry");
    assert!(
        leaves[entry].area() < 6.5 - 1e-9,
        "entry leaf {} is {:.2} sq m (room-sized)",
        entry,
        leaves[entry].area()
    );
}
