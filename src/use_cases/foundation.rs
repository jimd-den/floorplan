//! Foundation: the concrete stem wall a house stands on.
//!
//! The base slab is buried (its top is floor level, below grade),
//! so a house with no visible foundation reads as sitting on the
//! grass. The stem wall is the part you see: a concrete wall around
//! the ground floor's exterior perimeter, from the footing up to
//! 0.2 m above grade, 0.2 m thick — the same thickness as the
//! basement foundation walls — with a gap at every exterior door.
//! It is part of the envelope, so every LOD carries it.

use crate::domain::geometry::Segment;
use crate::domain::mesh::{Box3, Mesh};
use crate::domain::model::{House, OpeningKind, Room};
use crate::use_cases::materials::CONCRETE;
use crate::use_cases::walls::{WALL_T, openings_on};

/// The stem wall rises this far above grade, in metres.
pub const STEM_TOP: f32 = 0.2;
/// The stem wall starts at the footing, this far below floor level.
pub const STEM_BOTTOM: f32 = -0.3;
/// The stem wall is this thick, in metres — the basement
/// foundation-wall thickness, proud of the 0.15 m wall.
pub const STEM_T: f32 = 0.2;

/// Adds the concrete stem wall to the mesh.
pub fn add_stem_wall(mesh: &mut Mesh, house: &House) {
    for (b, m) in stem_wall_boxes(house) {
        mesh.add_box(b, m);
    }
}

/// The concrete stem wall boxes for a house: one box per run of
/// exterior wall between exterior doors, from the footing to 0.2 m
/// above grade. Only the true outline gets a stem wall: a long wall
/// beside two short rooms is shared along its whole length, so a
/// segment counts as exterior only where no neighbour overlaps it.
pub fn stem_wall_boxes(house: &House) -> Vec<(Box3, u32)> {
    let Some(floor) = house.floors.first() else { return vec![] };
    let mut out = vec![];
    for room in &floor.rooms {
        for (i, edge) in room.rect.edges().iter().enumerate() {
            for (lo, hi) in exposed_ranges(edge, room.id, &floor.rooms) {
                let doors: Vec<(f64, f64)> = openings_on(&floor.openings, edge)
                    .into_iter()
                    .filter(|(_, _, kind)| matches!(kind, OpeningKind::Door | OpeningKind::EntryDoor))
                    .map(|(a, b, _)| (a.max(lo), b.min(hi)))
                    .filter(|(a, b)| *b > *a + 1e-9)
                    .collect();
                let mut cursor = lo;
                for &(a, b) in &doors {
                    if a > cursor + 1e-9 {
                        out.push((stem_box(edge, i, cursor, a), CONCRETE));
                    }
                    cursor = b.max(cursor);
                }
                if hi > cursor + 1e-9 {
                    out.push((stem_box(edge, i, cursor, hi), CONCRETE));
                }
            }
        }
    }
    out
}

/// The parts of a wall segment that face the outside: the runs not
/// covered by another room's wall. The owner's own edge is skipped;
/// any other room's collinear overlapping edge — full or partial —
/// covers that run.
fn exposed_ranges(seg: &Segment, owner: u32, rooms: &[Room]) -> Vec<(f64, f64)> {
    let (lo, hi) = if seg.is_horizontal() { (seg.x1, seg.x2) } else { (seg.y1, seg.y2) };
    let fixed = if seg.is_horizontal() { seg.y1 } else { seg.x1 };
    let mut covered: Vec<(f64, f64)> = vec![];
    for r in rooms {
        for e in r.rect.edges() {
            if r.id == owner && e == *seg {
                continue;
            }
            if e.is_horizontal() != seg.is_horizontal() {
                continue;
            }
            let efixed = if seg.is_horizontal() { e.y1 } else { e.x1 };
            if (efixed - fixed).abs() > 1e-9 {
                continue;
            }
            let (elo, ehi) = if seg.is_horizontal() { (e.x1, e.x2) } else { (e.y1, e.y2) };
            let (a, b) = (elo.max(lo), ehi.min(hi));
            if b > a + 1e-9 {
                covered.push((a, b));
            }
        }
    }
    covered.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut out = vec![];
    let mut cursor = lo;
    for (a, b) in covered {
        if a > cursor + 1e-9 {
            out.push((cursor, a));
        }
        cursor = b.max(cursor);
    }
    if hi > cursor + 1e-9 {
        out.push((cursor, hi));
    }
    out
}

/// One stem wall box for the run [p, q] along a wall segment:
/// 0.2 m thick, from the footing to 0.2 m above grade. `edge` is
/// the room edge index (0 north, 1 east, 2 south, 3 west): the inner
/// face is flush with the wall's inner face, so it never shows
/// inside the room; it stands proud of the siding by 0.05 m
/// outside, where the foundation is meant to read.
fn stem_box(seg: &Segment, edge: usize, p: f64, q: f64) -> Box3 {
    let half = WALL_T as f64 / 2.0;
    let proud = STEM_T as f64 - half;
    match edge {
        0 => Box3::new(
            [p as f32, STEM_BOTTOM, (seg.y1 - proud) as f32],
            [q as f32, STEM_TOP, (seg.y1 + half) as f32],
        ),
        2 => Box3::new(
            [p as f32, STEM_BOTTOM, (seg.y1 - half) as f32],
            [q as f32, STEM_TOP, (seg.y1 + proud) as f32],
        ),
        3 => Box3::new(
            [(seg.x1 - proud) as f32, STEM_BOTTOM, p as f32],
            [(seg.x1 + half) as f32, STEM_TOP, q as f32],
        ),
        _ => Box3::new(
            [(seg.x1 - half) as f32, STEM_BOTTOM, p as f32],
            [(seg.x1 + proud) as f32, STEM_TOP, q as f32],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::OpeningKind;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;

    #[test]
    fn a_house_gets_a_concrete_stem_wall_above_grade_with_a_gap_at_the_entry() {
        let house = generate_house(42, Style::Ranch);
        let boxes = stem_wall_boxes(&house);
        assert!(!boxes.is_empty(), "no stem wall");
        assert!(boxes.iter().all(|(_, m)| *m == CONCRETE), "stem wall is not concrete");
        assert!(
            boxes.iter().all(|(b, _)| (b.max[1] - STEM_TOP).abs() < 1e-6),
            "stem wall top is not 0.2 above grade"
        );
        assert!(
            boxes.iter().all(|(b, _)| (b.min[1] - STEM_BOTTOM).abs() < 1e-6),
            "stem wall bottom is not the footing"
        );
        let entry = house.floors[0]
            .openings
            .iter()
            .find(|o| o.kind == OpeningKind::EntryDoor)
            .expect("seed 42 Ranch has no entry door");
        assert!(
            !boxes.iter().any(|(b, _)| b.contains([entry.x as f32, 0.1, entry.y as f32])),
            "stem wall blocks the entry door"
        );
    }

    #[test]
    fn the_stem_wall_never_shows_inside_a_room() {
        let house = generate_house(42, Style::Ranch);
        let boxes = stem_wall_boxes(&house);
        let living = house.floors[0]
            .rooms
            .iter()
            .find(|r| r.program == crate::domain::model::Program::Living)
            .expect("seed 42 Ranch has no living room");
        // Just inside the living room's north wall, above the
        // baseboard and below the stem wall top: the old centred
        // stem wall stuck 0.025 m into the room here and read as a
        // grey strip; the flush stem wall must not reach this point.
        let cx = (living.rect.x + living.rect.w / 2.0) as f32;
        let z = (living.rect.y + 0.09) as f32;
        assert!(
            !boxes.iter().any(|(b, _)| b.contains([cx, 0.15, z])),
            "stem wall protrudes into the living room"
        );
    }
}
