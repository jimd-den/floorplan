//! Furniture density: a full set per room, chairs by tables, unique
//! ids, and deterministic furnishing.

use floorplan::domain::furniture::FurnitureKind;
use floorplan::domain::geometry::Rect;
use floorplan::domain::model::{Floor, Program, Room};
use floorplan::domain::style::Style;
use floorplan::frameworks_drivers::rng::Rng;
use floorplan::use_cases::furnish::furnish;
use floorplan::use_cases::generate::generate_house;

fn bedroom_floor() -> Floor {
    Floor {
        level: 0,
        rooms: vec![Room { id: 0, program: Program::Bedroom, rect: Rect::new(0.0, 0.0, 6.0, 5.0) }],
        openings: vec![],
        furniture: vec![],
        open_doors: vec![],
    }
}

#[test]
fn a_bedroom_gets_a_bed_a_nightstand_and_a_cabinet() {
    let floor = bedroom_floor();
    let pieces = furnish(&mut Rng::new(1), &floor);
    let kinds: Vec<FurnitureKind> = pieces.iter().map(|p| p.kind).collect();
    assert!(kinds.contains(&FurnitureKind::Bed), "no bed: {kinds:?}");
    assert!(kinds.contains(&FurnitureKind::Nightstand), "no nightstand: {kinds:?}");
    assert!(kinds.contains(&FurnitureKind::Cabinet), "no cabinet: {kinds:?}");
}

#[test]
fn a_bath_gets_a_tub_and_a_sink() {
    let floor = Floor {
        level: 0,
        rooms: vec![Room { id: 0, program: Program::Bath, rect: Rect::new(0.0, 0.0, 2.4, 2.5) }],
        openings: vec![],
        furniture: vec![],
        open_doors: vec![],
    };
    let pieces = furnish(&mut Rng::new(4), &floor);
    let kinds: Vec<FurnitureKind> = pieces.iter().map(|p| p.kind).collect();
    assert!(kinds.contains(&FurnitureKind::Tub), "no tub: {kinds:?}");
    assert!(kinds.contains(&FurnitureKind::Sink), "no sink: {kinds:?}");
}

#[test]
fn furnishing_is_deterministic() {
    let house = generate_house(42, Style::Modern);
    let pa = furnish(&mut Rng::new(42), &house.floors[0]);
    let pb = furnish(&mut Rng::new(42), &house.floors[0]);
    assert_eq!(pa, pb);
}

#[test]
fn rooms_are_filled_with_a_full_furniture_set() {
    let mut total = 0usize;
    let mut rooms = 0usize;
    for seed in 0..20u32 {
        for style in Style::all() {
            let house = generate_house(seed, style);
            for floor in &house.floors {
                for room in &floor.rooms {
                    if room.program == Program::Garage {
                        continue;
                    }
                    total += floor.furniture.iter().filter(|p| p.room_id == room.id).count();
                    rooms += 1;
                }
            }
        }
    }
    let average = total / rooms;
    assert!(average >= 4, "only {average} pieces per room on average");
}

#[test]
fn a_chair_is_placed_next_to_a_table() {
    let floor = Floor {
        level: 0,
        rooms: vec![Room { id: 0, program: Program::Dining, rect: Rect::new(0.0, 0.0, 6.0, 5.0) }],
        openings: vec![],
        furniture: vec![],
        open_doors: vec![],
    };
    let pieces = furnish(&mut Rng::new(7), &floor);
    let table = pieces.iter().find(|p| p.kind == FurnitureKind::Table).expect("no table");
    assert!(
        pieces.iter().any(|p| {
            p.kind == FurnitureKind::Chair
                && p.rect.overlaps(&Rect::new(
                    table.rect.x - 0.6,
                    table.rect.y - 0.6,
                    table.rect.w + 1.2,
                    table.rect.d + 1.2,
                ))
        }),
        "no chair next to the table"
    );
}

#[test]
fn furniture_ids_are_unique_per_house() {
    for seed in 0..20u32 {
        let house = generate_house(seed, Style::Modern);
        let mut ids: Vec<u32> =
            house.floors.iter().flat_map(|f| f.furniture.iter().map(|p| p.id)).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "seed {seed}: duplicate furniture ids");
    }
}
