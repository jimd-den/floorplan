//! Openings: no two may share wall length.
//!
//! A window must never be cut through the entry door's span, and two
//! windows on collinear exterior stretches must not drift into each
//! other. `Opening::overlaps` is the predicate both the placer and
//! this test use.

use floorplan::domain::model::{Opening, OpeningKind};
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;

fn style_for(i: u32) -> Style {
    Style::all()[(i % 6) as usize]
}

#[test]
fn no_two_openings_overlap_on_the_same_wall_for_300_seeds() {
    for seed in 0..300u32 {
        let house = generate_house(seed, style_for(seed));
        for floor in &house.floors {
            for (i, a) in floor.openings.iter().enumerate() {
                for b in floor.openings.iter().skip(i + 1) {
                    assert!(
                        !a.overlaps(b),
                        "seed {seed} f{} {:?} at {:.2} w{:.2} overlaps {:?} at {:.2} w{:.2}",
                        floor.level,
                        a.kind,
                        a.x,
                        a.width,
                        b.kind,
                        b.x,
                        b.width
                    );
                }
            }
        }
    }
}

#[test]
fn an_entry_door_never_shares_its_span_with_a_window() {
    // The front door is the case that failed first: the entry room's
    // south wall is exterior, and a window used to be cut through it.
    let mut checked = 0;
    for seed in 0..200u32 {
        let house = generate_house(seed, style_for(seed));
        for floor in &house.floors {
            let entries: Vec<&Opening> = floor
                .openings
                .iter()
                .filter(|o| o.kind == OpeningKind::EntryDoor)
                .collect();
            for entry in entries {
                for o in floor.openings.iter().filter(|o| o.kind == OpeningKind::Window) {
                    assert!(
                        !o.overlaps(entry),
                        "seed {seed}: a window overlaps the entry door"
                    );
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 0, "no entry door was compared with a window");
}
