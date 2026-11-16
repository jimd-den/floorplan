//! Doors: the connectivity graph and the openings in shared walls.
//! One door crosses every backbone link of the floor's skeleton —
//! the links over BSP splits and footprint joins — so the room
//! graph is connected by construction. Doors are sized by their
//! opening (see `door_width`).

use crate::domain::geometry::{Axis, Segment};
use crate::domain::model::{Opening, OpeningKind, Room};
use crate::domain::skeleton::{Skeleton, SkeletonLink};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::circulation::extras;
use crate::use_cases::door_width::door_width;
use crate::use_cases::quant;

/// Doors: one per backbone link that can hold one, the entry door,
/// plus extras.
pub fn place_doors(
    rng: &mut Rng,
    rooms: &[Room],
    skeleton: &Skeleton,
    entry_room: u32,
) -> Vec<Opening> {
    let mut openings = vec![];
    let mut next_id = 0u32;
    // One door per backbone link that can hold one. A sliver of
    // shared wall under 0.6 m cannot hold a door; the rooms it
    // joins stay connected through their other links.
    for link in skeleton.backbone_links() {
        if link.segment.len() >= 0.6 {
            openings.push(make_door(rng, &link, &mut next_id));
        }
    }
    if let Some(entry) = entry_door(rng, rooms, entry_room, &mut next_id) {
        openings.push(entry);
    }
    openings.extend(extras(rng, rooms, &skeleton.links, &openings, &mut next_id));
    openings
}

/// Builds one door centred on a shared segment, jittered by the seed.
/// The width comes from the opening (see `door_width`).
pub(crate) fn make_door(rng: &mut Rng, link: &SkeletonLink, next_id: &mut u32) -> Opening {
    let seg = link.segment;
    let width = door_width(seg.len());
    let slack = (seg.len() - width - 0.6).max(0.0);
    let offset = jitter(rng, slack);
    let (x, y, axis) = if seg.is_horizontal() {
        ((seg.x1 + seg.x2) / 2.0 + offset, seg.y1, Axis::Horizontal)
    } else {
        (seg.x1, (seg.y1 + seg.y2) / 2.0 + offset, Axis::Vertical)
    };
    let id = *next_id;
    *next_id += 1;
    Opening { id, kind: OpeningKind::Door, x, y, axis, width, from: link.a, to: Some(link.b) }
}

/// A seeded offset within ±slack/2, quantized to 0.1 m.
fn jitter(rng: &mut Rng, slack: f64) -> f64 {
    if slack > 0.0 { quant(rng.range_f64(-slack / 2.0, slack / 2.0)) } else { 0.0 }
}

/// The front door on the entry room's southern wall, sized by
/// the front it sits in — a wide front takes a double door.
fn entry_door(rng: &mut Rng, rooms: &[Room], entry_room: u32, next_id: &mut u32) -> Option<Opening> {
    let room = rooms.iter().find(|r| r.id == entry_room)?;
    let edge = Segment::new(room.rect.x, room.rect.bottom(), room.rect.right(), room.rect.bottom());
    let width = door_width(edge.len());
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
    use crate::domain::geometry::{Rect, shared_edge};
    use crate::domain::model::Program;
    use crate::use_cases::bsp::subdivide;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    /// Two rooms from one vertical split of a 10x6 rect, and the
    /// skeleton built from them.
    fn two_rooms() -> (Vec<Room>, Skeleton) {
        let mut rng = Rng::new(4);
        let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 10.0, 6.0), 2);
        let rooms = vec![
            room(0, Program::Living, sub.leaves[0]),
            room(1, Program::Kitchen, sub.leaves[1]),
        ];
        let skeleton = crate::use_cases::skeleton::build_skeleton(&rooms, &sub.splits, &[], 0);
        (rooms, skeleton)
    }

    #[test]
    fn a_split_gets_exactly_one_door() {
        let (rooms, skeleton) = two_rooms();
        let mut rng = Rng::new(4);
        let doors = place_doors(&mut rng, &rooms, &skeleton, 0);
        assert_eq!(doors.iter().filter(|d| d.kind == OpeningKind::Door).count(), 1);
    }

    #[test]
    fn the_door_sits_on_the_shared_wall() {
        let (rooms, skeleton) = two_rooms();
        let mut rng = Rng::new(4);
        let doors = place_doors(&mut rng, &rooms, &skeleton, 0);
        let door = doors.iter().find(|d| d.kind == OpeningKind::Door).unwrap();
        let shared = shared_edge(&rooms[0].rect, &rooms[1].rect).unwrap();
        assert!((door.x - shared.x1).abs() < 1e-9, "door x must sit on the shared wall");
        assert!(door.y >= shared.y1 - 1e-9 && door.y <= shared.y2 + 1e-9);
        assert!(door.width <= shared.len() - 0.5);
    }

    #[test]
    fn a_wide_opening_gets_a_double_door() {
        let (rooms, skeleton) = two_rooms();
        let mut rng = Rng::new(4);
        let doors = place_doors(&mut rng, &rooms, &skeleton, 0);
        let door = doors.iter().find(|d| d.kind == OpeningKind::Door).unwrap();
        let shared = shared_edge(&rooms[0].rect, &rooms[1].rect).unwrap();
        assert_eq!(door.width, crate::use_cases::door_width::door_width(shared.len()));
        assert!(
            crate::use_cases::door_width::is_double_door(door.width),
            "a 6 m opening must take a double door, not {}",
            door.width
        );
    }

    #[test]
    fn the_entry_door_is_sized_by_its_front_wall() {
        let (rooms, skeleton) = two_rooms();
        let mut rng = Rng::new(4);
        let doors = place_doors(&mut rng, &rooms, &skeleton, 0);
        let entry = doors.iter().find(|d| d.kind == OpeningKind::EntryDoor).unwrap();
        let front = rooms[0].rect.w;
        assert_eq!(entry.width, crate::use_cases::door_width::door_width(front));
        assert!(
            crate::use_cases::door_width::is_double_door(entry.width),
            "a 5 m front must take a double front door, not {}",
            entry.width
        );
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
        let skeleton = crate::use_cases::skeleton::build_skeleton(&rooms, &sub.splits, &[], entry);
        let doors = place_doors(&mut rng, &rooms, &skeleton, entry);
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
