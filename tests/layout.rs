//! Layout planner tests: the three families plan distinct street
//! networks, and the grid plan matches the original builder.

use floorplan::domain::geometry::Rect;
use floorplan::domain::layout::Layout;
use floorplan::frameworks_drivers::rng::Rng;
use floorplan::use_cases::layout::plan_layout;

#[test]
fn the_grid_plan_matches_the_old_builder() {
    let mut rng = Rng::new(7);
    let plan = plan_layout(&mut rng, Layout::Grid, (3, 3), 80.0);
    assert_eq!(plan.streets.len(), 8);
    assert_eq!(plan.streets[0].rect, Rect::new(0.0, 0.0, 8.0, 272.0));
    assert!(!plan.streets[0].horizontal);
    assert_eq!(plan.streets[4].rect, Rect::new(0.0, 0.0, 272.0, 8.0));
    assert!(plan.streets[4].horizontal);
    let b = &plan.blocks[0];
    assert_eq!(b.rect, Rect::new(8.0, 8.0, 80.0, 80.0));
    assert_eq!((b.north, b.south, b.west, b.east), (4, 5, 0, 1));
    assert!(b.court.is_none());
}

#[test]
fn the_grid_plan_consumes_no_rng_draws() {
    let mut rng = Rng::new(7);
    let first = Rng::new(7).next_u64();
    plan_layout(&mut rng, Layout::Grid, (2, 2), 80.0);
    assert_eq!(rng.next_u64(), first, "grid planning drew from the rng");
}

#[test]
fn the_offset_plan_staggers_alternate_rows() {
    let mut rng = Rng::new(7);
    let plan = plan_layout(&mut rng, Layout::Offset, (2, 2), 80.0);
    let row0: Vec<f64> = plan
        .streets
        .iter()
        .filter(|s| !s.horizontal && s.rect.y == 0.0)
        .map(|s| s.rect.x)
        .collect();
    let row1: Vec<f64> = plan
        .streets
        .iter()
        .filter(|s| !s.horizontal && s.rect.y == 88.0)
        .map(|s| s.rect.x)
        .collect();
    assert_eq!(row0, vec![0.0, 88.0, 176.0]);
    assert_eq!(row1, vec![40.0, 128.0, 216.0], "odd rows not shifted by half a block");
    assert_eq!(plan.blocks[2].rect.x, 48.0, "row-1 block not shifted");
}

#[test]
fn the_cul_de_sac_plan_has_one_or_two_courts() {
    let mut rng = Rng::new(7);
    let plan = plan_layout(&mut rng, Layout::CulDeSac, (2, 2), 80.0);
    let courts: Vec<_> = plan.blocks.iter().filter(|b| b.court.is_some()).collect();
    assert!((1..=2).contains(&courts.len()), "{} courts", courts.len());
    assert_eq!(plan.streets.len(), 6 + 2 * courts.len());
    for b in courts {
        let court = b.court.as_ref().unwrap();
        let stub = &plan.streets[court.stub];
        let bulb = &plan.streets[court.bulb];
        assert!(stub.rect.x >= b.rect.x && stub.rect.right() <= b.rect.right());
        assert!(bulb.rect.x >= b.rect.x && bulb.rect.right() <= b.rect.right());
        assert!(bulb.rect.y >= b.rect.y && bulb.rect.bottom() <= b.rect.bottom());
    }
}
