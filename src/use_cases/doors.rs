//! Doors: the connectivity graph and the openings in shared walls.
//! One door crosses every backbone link of the floor's skeleton —
//! the links over BSP splits and footprint joins — so the room
//! graph is connected by construction. Doors are sized by the
//! rooms they join (see `door_width`): a real house has at most
//! one double door per floor, on its living/dining opening.

use crate::domain::geometry::{Axis, Segment};
use crate::domain::model::{Opening, OpeningKind, Program, Room};
use crate::domain::skeleton::{Skeleton, SkeletonLink};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::circulation::extras;
use crate::use_cases::door_width::{
    entry_door_width, interior_door_width, is_double_opening, is_small_room,
};
use crate::use_cases::quant;

/// Doors: one per backbone link that can hold one, the entry door,
/// plus extras. At most one backbone door is a double: the widest
/// opening between two main rooms.
pub fn place_doors(
    rng: &mut Rng,
    rooms: &[Room],
    skeleton: &Skeleton,
    entry_room: u32,
) -> Vec<Opening> {
    let mut openings = vec![];
    let mut next_id = 0u32;
    let links = skeleton.backbone_links();
    // The floor's one double door: the widest opening between the
    // living room and the dining room — the one place a real house
    // puts a double door. A long shared wall elsewhere does not
    // make a double doorway; those are singles.
    let double = links
        .iter()
        .enumerate()
        .filter(|(_, link)| {
            link.segment.len() >= 0.6
                && is_double_opening(link.segment.len(), link_is_small(link, rooms))
                && link_joins(link, rooms, Program::Living, Program::Dining)
        })
        .max_by(|(_, a), (_, b)| a.segment.len().partial_cmp(&b.segment.len()).unwrap())
        .map(|(i, _)| i);
    // One door per backbone link that can hold one. A sliver of
    // shared wall under 0.6 m cannot hold a door; the rooms it
    // joins stay connected through their other links.
    for (i, link) in links.iter().enumerate() {
        if link.segment.len() >= 0.6 {
            openings.push(make_door(rng, link, rooms, double == Some(i), &mut next_id));
        }
    }
    if let Some(entry) = entry_door(rng, rooms, entry_room, &mut next_id) {
        openings.push(entry);
    }
    openings.extend(extras(rng, rooms, &skeleton.links, &openings, &mut next_id));
    openings
}

/// True when either room a link joins is small.
fn link_is_small(link: &SkeletonLink, rooms: &[Room]) -> bool {
    [link.a, link.b].iter().any(|id| {
        rooms.iter().find(|r| r.id == *id).is_some_and(|r| is_small_room(r.rect.area()))
    })
}

/// True when a link joins the two given programs, in either order.
fn link_joins(link: &SkeletonLink, rooms: &[Room], p: Program, q: Program) -> bool {
    let programs: Vec<Program> = [link.a, link.b]
        .iter()
        .filter_map(|id| rooms.iter().find(|r| r.id == *id).map(|r| r.program))
        .collect();
    programs.len() == 2 && programs.contains(&p) && programs.contains(&q)
}

/// Builds one door centred on a shared segment, jittered by the seed.
/// The width comes from the rooms it joins (see `door_width`): a
/// door to a small room is narrow, and only the floor's designated
/// double opening takes a double door.
pub(crate) fn make_door(
    rng: &mut Rng,
    link: &SkeletonLink,
    rooms: &[Room],
    double: bool,
    next_id: &mut u32,
) -> Opening {
    let seg = link.segment;
    let width = interior_door_width(seg.len(), link_is_small(link, rooms), double);
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

/// The front door on the entry room's southern wall: a single
/// 0.9 m door, whatever the front. A wide front does not make a
/// wide front door.
fn entry_door(rng: &mut Rng, rooms: &[Room], entry_room: u32, next_id: &mut u32) -> Option<Opening> {
    let room = rooms.iter().find(|r| r.id == entry_room)?;
    let edge = Segment::new(room.rect.x, room.rect.bottom(), room.rect.right(), room.rect.bottom());
    let width = entry_door_width();
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
