//! Doors: the connectivity graph and the openings in shared walls.
//! One door crosses every BSP split and every footprint join, so the
//! room graph is connected by construction.

use crate::domain::geometry::{Axis, Segment, shared_edge};
use crate::domain::model::{Opening, OpeningKind, Room};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::bsp::Split;
use crate::use_cases::circulation::extras;
use crate::use_cases::quant;

/// Two rooms and the wall segment they share.
#[derive(Debug, Clone, PartialEq)]
pub struct Adjacency {
    pub a: u32,
    pub b: u32,
    pub segment: Segment,
}

/// All shared walls between rooms.
pub fn adjacency(rooms: &[Room]) -> Vec<Adjacency> {
    let mut out = vec![];
    for (i, a) in rooms.iter().enumerate() {
        for b in rooms.iter().skip(i + 1) {
            if let Some(segment) = shared_edge(&a.rect, &b.rect) {
                out.push(Adjacency { a: a.id, b: b.id, segment });
            }
        }
    }
    out
}

/// Doors: one per split, one per join, the entry door, plus extras.
pub fn place_doors(
    rng: &mut Rng,
    rooms: &[Room],
    splits: &[Split],
    joins: &[Segment],
    entry_room: u32,
) -> Vec<Opening> {
    let adj = adjacency(rooms);
    let mut openings = vec![];
    let mut next_id = 0u32;
    for split in splits {
        if let Some(door) = door_on_line(rng, &adj, &split.line, &mut next_id) {
            openings.push(door);
        }
    }
    for join in joins {
        if let Some(door) = door_on_line(rng, &adj, join, &mut next_id) {
            openings.push(door);
        }
    }
    if let Some(entry) = entry_door(rng, rooms, entry_room, &mut next_id) {
        openings.push(entry);
    }
    openings.extend(extras(rng, rooms, &adj, &openings, &mut next_id));
    openings
}

/// A door on the longest adjacency whose segment lies on `line`.
fn door_on_line(
    rng: &mut Rng,
    adj: &[Adjacency],
    line: &Segment,
    next_id: &mut u32,
) -> Option<Opening> {
    let best = adj
        .iter()
        .filter(|a| a.segment.overlap(line).is_some())
        .max_by(|a, b| a.segment.len().partial_cmp(&b.segment.len()).unwrap())?;
    Some(make_door(rng, best, next_id))
}

/// Builds one door centred on a shared segment, jittered by the seed.
pub(crate) fn make_door(rng: &mut Rng, adj: &Adjacency, next_id: &mut u32) -> Opening {
    let seg = adj.segment;
    let width = (seg.len() - 0.6).clamp(0.6, 0.9);
    let slack = (seg.len() - width - 0.6).max(0.0);
    let offset = jitter(rng, slack);
    let (x, y, axis) = if seg.is_horizontal() {
        ((seg.x1 + seg.x2) / 2.0 + offset, seg.y1, Axis::Horizontal)
    } else {
        (seg.x1, (seg.y1 + seg.y2) / 2.0 + offset, Axis::Vertical)
    };
    let id = *next_id;
    *next_id += 1;
    Opening { id, kind: OpeningKind::Door, x, y, axis, width, from: adj.a, to: Some(adj.b) }
}

/// A seeded offset within ±slack/2, quantized to 0.1 m.
fn jitter(rng: &mut Rng, slack: f64) -> f64 {
    if slack > 0.0 { quant(rng.range_f64(-slack / 2.0, slack / 2.0)) } else { 0.0 }
}

/// The wide front door on the entry room's southern wall.
fn entry_door(rng: &mut Rng, rooms: &[Room], entry_room: u32, next_id: &mut u32) -> Option<Opening> {
    let room = rooms.iter().find(|r| r.id == entry_room)?;
    let edge = Segment::new(room.rect.x, room.rect.bottom(), room.rect.right(), room.rect.bottom());
    let width = 1.0;
    let slack = (edge.len() - width - 0.6).max(0.0);
    let offset = jitter(rng, slack);
    let id = *next_id;
    *next_id += 1;
    Some(Opening {
        id,
        kind: OpeningKind::EntryDoor,
        x: (edge.x1 + edge.x2) / 2.0 + offset,
        y: edge.y1,
        axis: Axis::Horizontal,
        width,
        from: entry_room,
        to: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect;
    use crate::domain::model::Program;
    use crate::use_cases::bsp::subdivide;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    /// Two rooms from one vertical split of a 10x6 rect.
    fn two_rooms() -> (Vec<Room>, Vec<Split>) {
        let mut rng = Rng::new(4);
        let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 10.0, 6.0), 2);
        let rooms = vec![
            room(0, Program::Living, sub.leaves[0]),
            room(1, Program::Kitchen, sub.leaves[1]),
        ];
        (rooms, sub.splits)
    }

    #[test]
    fn a_split_gets_exactly_one_door() {
        let (rooms, splits) = two_rooms();
        let mut rng = Rng::new(4);
        let doors = place_doors(&mut rng, &rooms, &splits, &[], 0);
        assert_eq!(doors.iter().filter(|d| d.kind == OpeningKind::Door).count(), 1);
    }

    #[test]
    fn the_door_sits_on_the_shared_wall() {
        let (rooms, splits) = two_rooms();
        let mut rng = Rng::new(4);
        let doors = place_doors(&mut rng, &rooms, &splits, &[], 0);
        let door = doors.iter().find(|d| d.kind == OpeningKind::Door).unwrap();
        let shared = shared_edge(&rooms[0].rect, &rooms[1].rect).unwrap();
        assert!((door.x - shared.x1).abs() < 1e-9, "door x must sit on the shared wall");
        assert!(door.y >= shared.y1 - 1e-9 && door.y <= shared.y2 + 1e-9);
        assert!(door.width <= shared.len() - 0.5);
    }

    #[test]
    fn entry_door_is_wide_and_on_the_front_wall() {
        let (rooms, splits) = two_rooms();
        let mut rng = Rng::new(4);
        let doors = place_doors(&mut rng, &rooms, &splits, &[], 0);
        let entry = doors.iter().find(|d| d.kind == OpeningKind::EntryDoor).unwrap();
        assert_eq!(entry.width, 1.0);
        assert_eq!(entry.from, 0);
        assert_eq!(entry.to, None);
    }

    #[test]
    fn every_room_is_reachable_from_the_entry() {
        let mut rng = Rng::new(5);
        let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 12.0, 9.0), 6);
        let programs =
            crate::use_cases::programs::assign_programs(&sub.leaves, 0, false);
        let rooms: Vec<Room> = sub
            .leaves
            .iter()
            .enumerate()
            .map(|(i, r)| room(i as u32, programs[i], *r))
            .collect();
        let entry = rooms.iter().find(|r| r.program == Program::Entry).unwrap().id;
        let doors = place_doors(&mut rng, &rooms, &sub.splits, &[], entry);
        let mut seen = vec![entry];
        let mut stack = vec![entry];
        while let Some(cur) = stack.pop() {
            for d in doors.iter().filter(|d| d.kind == OpeningKind::Door) {
                let next = if d.from == cur { d.to } else if d.to == Some(cur) { Some(d.from) } else { None };
                if let Some(n) = next
                    && !seen.contains(&n)
                {
                    seen.push(n);
                    stack.push(n);
                }
            }
        }
        assert_eq!(seen.len(), rooms.len(), "unreachable rooms remain");
    }
}
