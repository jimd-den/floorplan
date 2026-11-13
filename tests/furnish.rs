use floorplan::domain::furniture::FurnitureKind;
use floorplan::domain::model::{OpeningKind, Program};
use floorplan::domain::style::Style;
use floorplan::use_cases::furnish::furnish;
use floorplan::use_cases::furnish_clear::{blocks_door, blocks_window};
use floorplan::use_cases::generate::generate_house;

/// Every placed piece sits inside its room, overlaps no other
/// piece, blocks no door swing and no window, and stands against
/// a wall so the room's centre stays walkable. Chairs sit next to
/// a table or desk instead. A piece that fits nowhere is skipped,
/// so the test also gates the placement rate: at least 90% of
/// bedrooms get a bed and 90% of baths get a tub.
#[test]
fn placement_rules_hold_for_200_seeds() {
    let mut bedrooms = 0;
    let mut bedrooms_with_bed = 0;
    let mut baths = 0;
    let mut baths_with_tub = 0;
    for style in Style::all() {
        for seed in 0..200u32 {
            let house = generate_house(seed, style);
            for floor in &house.floors {
                let pieces = furnish(&mut floorplan::frameworks_drivers::rng::Rng::new(seed), floor);
                check_floor(floor, &pieces, seed, style);
                for room in &floor.rooms {
                    if room.program == Program::Bedroom {
                        bedrooms += 1;
                        if pieces.iter().any(|p| p.room_id == room.id && p.kind == FurnitureKind::Bed) {
                            bedrooms_with_bed += 1;
                        }
                    }
                    if room.program == Program::Bath {
                        baths += 1;
                        if pieces.iter().any(|p| p.room_id == room.id && p.kind == FurnitureKind::Tub) {
                            baths_with_tub += 1;
                        }
                    }
                }
            }
        }
    }
    assert!(
        bedrooms_with_bed * 10 >= bedrooms * 9,
        "only {bedrooms_with_bed}/{bedrooms} bedrooms got a bed"
    );
    assert!(
        baths_with_tub * 10 >= baths * 9,
        "only {baths_with_tub}/{baths} baths got a tub"
    );
}

fn check_floor(
    floor: &floorplan::domain::model::Floor,
    pieces: &[floorplan::domain::furniture::Furniture],
    seed: u32,
    style: Style,
) {
    use floorplan::domain::furniture::FurnitureKind;
    let rooms: std::collections::HashMap<u32, &floorplan::domain::model::Room> =
        floor.rooms.iter().map(|r| (r.id, r)).collect();
    for (i, p) in pieces.iter().enumerate() {
        let room = rooms[&p.room_id];
        assert!(
            room.rect.x - 1e-9 <= p.rect.x
                && room.rect.y - 1e-9 <= p.rect.y
                && p.rect.right() <= room.rect.right() + 1e-9
                && p.rect.bottom() <= room.rect.bottom() + 1e-9,
            "seed {seed} {style:?} piece {i} ({:?}) escapes room {}",
            p.kind,
            room.id
        );
        // Against a wall: one edge of the piece touches a room wall.
        // Chairs sit next to a table or desk instead.
        let against = (p.rect.x - room.rect.x).abs() < 1e-9
            || (p.rect.y - room.rect.y).abs() < 1e-9
            || (p.rect.right() - room.rect.right()).abs() < 1e-9
            || (p.rect.bottom() - room.rect.bottom()).abs() < 1e-9;
        let by_table = p.kind == FurnitureKind::Chair
            && pieces.iter().any(|q| {
                q.room_id == p.room_id
                    && matches!(q.kind, FurnitureKind::Table | FurnitureKind::Desk)
                    && ((p.rect.x - q.rect.right()).abs() < 0.2
                        || (q.rect.x - p.rect.right()).abs() < 0.2
                        || (p.rect.y - q.rect.bottom()).abs() < 0.2
                        || (q.rect.y - p.rect.bottom()).abs() < 0.2)
            });
        assert!(
            against || by_table,
            "seed {seed} {style:?} piece {i} ({:?}) is not against a wall or a table",
            p.kind
        );
        for q in &pieces[i + 1..] {
            assert!(
                !p.rect.overlaps(&q.rect),
                "seed {seed} {style:?} pieces {i} and {:?} overlap",
                q.kind
            );
        }
        for o in &floor.openings {
            if o.from != p.room_id && o.to != Some(p.room_id) {
                continue;
            }
            if o.kind == OpeningKind::Window {
                assert!(
                    !p.kind.is_tall() || !blocks_window(&p.rect, o),
                    "seed {seed} {style:?} piece {i} ({:?}) blocks a window in room {}",
                    p.kind,
                    room.id
                );
            } else {
                assert!(
                    !blocks_door(room, &p.rect, o),
                    "seed {seed} {style:?} piece {i} ({:?}) blocks a door in room {}",
                    p.kind,
                    room.id
                );
            }
        }
    }
}
