//! Furnish: place a room's furniture set against its walls.
//! Runs after programs, doors and windows. Placement rules
//! (spec §4.2): keep door swings and circulation clear, keep
//! window walls clear where the program wants light, never block
//! a door, never overlap, every piece reachable. A piece that
//! fits nowhere is skipped. Chairs sit next to a table or desk.

use crate::domain::furniture::{Facing, Furniture, FurnitureKind};
use crate::domain::geometry::Rect;
use crate::use_cases::furnish_clear::fits;
use crate::domain::model::{Floor, Opening, OpeningKind, Program, Room};
use crate::frameworks_drivers::rng::Rng;

/// The furniture set for a room program.
pub fn furniture_set(program: Program) -> &'static [FurnitureKind] {
    use FurnitureKind::*;
    match program {
        Program::Bedroom => &[Bed, Nightstand, Cabinet],
        Program::Dining => &[Table, Chair, Chair, Chair, Chair],
        Program::Living => &[Sofa, Lamp, Table, Shelf],
        Program::Bath => &[Tub, Sink],
        Program::Study => &[Desk, Chair, Shelf],
        Program::Kitchen => &[Counter, Table, Chair, Chair],
        Program::Entry => &[Shelf],
        Program::Hall => &[Shelf],
        Program::Laundry => &[Shelf],
        Program::Garage => &[],
    }
}

/// Places furniture for every room on a floor. Deterministic in
/// the seed: the same floor always furnishes the same way. Piece
/// ids are room id × 100 plus the index in the room, unique
/// across the house so commands can address a piece.
pub fn furnish(rng: &mut Rng, floor: &Floor) -> Vec<Furniture> {
    let mut out = vec![];
    for room in &floor.rooms {
        let doors: Vec<&Opening> = floor
            .openings
            .iter()
            .filter(|o| o.kind != OpeningKind::Window && (o.from == room.id || o.to == Some(room.id)))
            .collect();
        let windows: Vec<&Opening> = floor
            .openings
            .iter()
            .filter(|o| o.kind == OpeningKind::Window && o.from == room.id)
            .collect();
        let mut placed: Vec<Furniture> = vec![];
        for &kind in furniture_set(room.program) {
            if kind == FurnitureKind::Chair {
                if let Some(c) = place_chair(rng, room, &doors, &windows, &placed) {
                    placed.push(c);
                }
            } else if let Some(p) = place_against_wall(rng, room, kind, &doors, &windows, &placed) {
                placed.push(p);
            }
        }
        for (i, p) in placed.iter_mut().enumerate() {
            p.id = room.id * 100 + i as u32;
        }
        out.extend(placed);
    }
    out
}

/// A chair sits next to a table or desk already in the room, on a
/// free side; failing that, against a wall like any other piece.
fn place_chair(
    rng: &mut Rng,
    room: &Room,
    doors: &[&Opening],
    windows: &[&Opening],
    placed: &[Furniture],
) -> Option<Furniture> {
    let mut anchors: Vec<&Furniture> = placed
        .iter()
        .filter(|p| matches!(p.kind, FurnitureKind::Table | FurnitureKind::Desk))
        .collect();
    rng.shuffle(&mut anchors);
    let (w, d) = FurnitureKind::Chair.size();
    for anchor in anchors {
        for (x, y, facing) in chair_spots(anchor, w, d) {
            let rect = Rect::new(x, y, w, d);
            if fits(room, FurnitureKind::Chair, &rect, doors, windows, placed) {
                return Some(Furniture::new(0, FurnitureKind::Chair, room.id, rect, facing));
            }
        }
    }
    place_against_wall(rng, room, FurnitureKind::Chair, doors, windows, placed)
}

/// Candidate chair spots around an anchor: one per side, centred
/// on the side, facing the anchor.
fn chair_spots(anchor: &Furniture, w: f64, d: f64) -> Vec<(f64, f64, Facing)> {
    let r = anchor.rect;
    let gap = 0.15;
    vec![
        (r.x + r.w / 2.0 - w / 2.0, r.y - d - gap, Facing::South),
        (r.x + r.w / 2.0 - w / 2.0, r.bottom() + gap, Facing::North),
        (r.x - w - gap, r.y + r.d / 2.0 - d / 2.0, Facing::East),
        (r.right() + gap, r.y + r.d / 2.0 - d / 2.0, Facing::West),
    ]
}

/// Tries each wall in a seeded order, sliding the piece along it
/// in 0.1 m steps, and takes the first spot that fits. `along`
/// is the piece's extent along the wall, `into` its depth into
/// the room; east/west walls swap the two.
fn place_against_wall(
    rng: &mut Rng,
    room: &Room,
    kind: FurnitureKind,
    doors: &[&Opening],
    windows: &[&Opening],
    placed: &[Furniture],
) -> Option<Furniture> {
    let (sw, sd) = kind.size();
    let mut walls = [Facing::North, Facing::South, Facing::East, Facing::West];
    rng.shuffle(&mut walls);
    for facing in walls {
        let (along, into) = match facing {
            Facing::North | Facing::South => (sw, sd),
            Facing::East | Facing::West => (sd, sw),
        };
        let (start, end) = match facing {
            Facing::North | Facing::South => (room.rect.x, room.rect.right() - along),
            Facing::East | Facing::West => (room.rect.y, room.rect.bottom() - along),
        };
        let mut at = start;
        while at <= end + 1e-9 {
            let rect = match facing {
                Facing::North => Rect::new(at, room.rect.y, along, into),
                Facing::South => Rect::new(at, room.rect.bottom() - into, along, into),
                Facing::East => Rect::new(room.rect.right() - into, at, into, along),
                Facing::West => Rect::new(room.rect.x, at, into, along),
            };
            if fits(room, kind, &rect, doors, windows, placed) {
                return Some(Furniture::new(0, kind, room.id, rect, facing));
            }
            at = (at * 10.0).round() / 10.0 + 0.1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect as R;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;

    fn room_with(doors: Vec<Opening>, windows: Vec<Opening>) -> Floor {
        Floor {
            level: 0,
            rooms: vec![Room { id: 0, program: Program::Bedroom, rect: R::new(0.0, 0.0, 6.0, 5.0) }],
            openings: doors.into_iter().chain(windows).collect(),
            furniture: vec![],
            open_doors: vec![],
        }
    }

    #[test]
    fn a_bedroom_gets_a_bed_a_nightstand_and_a_cabinet() {
        let floor = room_with(vec![], vec![]);
        let mut rng = Rng::new(1);
        let pieces = furnish(&mut rng, &floor);
        let kinds: Vec<FurnitureKind> = pieces.iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&FurnitureKind::Bed), "no bed: {kinds:?}");
        assert!(kinds.contains(&FurnitureKind::Nightstand), "no nightstand: {kinds:?}");
        assert!(kinds.contains(&FurnitureKind::Cabinet), "no cabinet: {kinds:?}");
    }

    #[test]
    fn a_bath_gets_a_tub_and_a_sink() {
        let floor = Floor {
            level: 0,
            rooms: vec![Room { id: 0, program: Program::Bath, rect: R::new(0.0, 0.0, 2.4, 2.5) }],
            openings: vec![],
            furniture: vec![],
            open_doors: vec![],
        };
        let mut rng = Rng::new(4);
        let pieces = furnish(&mut rng, &floor);
        let kinds: Vec<FurnitureKind> = pieces.iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&FurnitureKind::Tub), "no tub: {kinds:?}");
        assert!(kinds.contains(&FurnitureKind::Sink), "no sink: {kinds:?}");
    }

    #[test]
    fn furnishing_is_deterministic() {
        let house = generate_house(42, Style::Modern);
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        let pa = furnish(&mut a, &house.floors[0]);
        let pb = furnish(&mut b, &house.floors[0]);
        assert_eq!(pa, pb);
    }
}
