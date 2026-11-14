//! Furnish: place a room's furniture set against its walls.
//! Runs after programs, doors and windows. Placement rules
//! (spec §4.2): keep door swings and circulation clear, keep
//! window walls clear where the program wants light, never block
//! a door, never overlap, every piece reachable. A piece that
//! fits nowhere is skipped. Chairs sit next to a table or desk.

use crate::domain::furniture::{Facing, Furniture, FurnitureKind};
use crate::domain::geometry::Rect;
use crate::use_cases::furnish_clear::{fits, walkable};
use crate::domain::model::{Floor, Opening, OpeningKind, Program, Room};
use crate::frameworks_drivers::rng::Rng;

/// The furniture set for a room program: a full set, not a token
/// piece. Pieces that cannot fit are skipped by `furnish`.
pub fn furniture_set(program: Program) -> &'static [FurnitureKind] {
    use FurnitureKind::*;
    match program {
        Program::Bedroom => &[Bed, Nightstand, Nightstand, Cabinet, Lamp, Chair],
        Program::Dining => &[Table, Chair, Chair, Chair, Chair, Cabinet],
        Program::Living => &[Sofa, Lamp, Table, Shelf, Chair, Cabinet],
        Program::Bath => &[Tub, Sink, Cabinet],
        Program::Study => &[Desk, Chair, Shelf, Shelf, Lamp],
        Program::Kitchen => &[Counter, Counter, Table, Chair, Chair, Shelf],
        Program::Entry => &[Shelf, Cabinet, Lamp],
        Program::Hall => &[Shelf, Cabinet],
        Program::Laundry => &[Shelf, Cabinet],
        Program::Garage => &[Cabinet, Shelf],
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
        keep_walkable(room, &doors, &mut placed);
        for (i, p) in placed.iter_mut().enumerate() {
            p.id = room.id * 100 + i as u32;
        }
        out.extend(placed);
    }
    out
}

/// Drops the last pieces until a person can walk from every door to
/// the room's centre. The set is ordered most-essential first, so the
/// pieces dropped are the extras.
fn keep_walkable(room: &Room, doors: &[&Opening], placed: &mut Vec<Furniture>) {
    while !placed.is_empty() && !walkable(room, doors, placed) {
        placed.pop();
    }
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

