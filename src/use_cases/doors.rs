//! Doors: the connectivity graph and the openings in shared walls.
//! One door crosses every backbone link of the floor's skeleton —
//! the links over BSP splits and footprint joins — so the room
//! graph is connected by construction. Doors are sized by the
//! rooms they join (see `door_width`): a real house has at most
//! one double door per floor, on its living/dining opening.

use crate::domain::geometry::{Axis, Segment, shared_edge};
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

/// The front door on the entry room's southern wall when that
/// wall faces outside — otherwise on the room's longest exterior
/// stretch. A pinned entry is an alcove leaf on the west or
/// north side, so its southern wall is interior; the door must
/// not be cut into it. A 0.9 m door needs 1.5 m of exterior
/// wall (0.3 m margins each side). No door when no edge has
/// one — a fully interior entry has nowhere to put it.
fn entry_door(rng: &mut Rng, rooms: &[Room], entry_room: u32, next_id: &mut u32) -> Option<Opening> {
    let room = rooms.iter().find(|r| r.id == entry_room)?;
    let edges = room.rect.edges();
    let south = longest_part(&edges[2], rooms, room);
    let edge = south.filter(|s| s.len() >= 1.5).or_else(|| {
        edges
            .iter()
            .flat_map(|e| exterior_parts(e, rooms, room))
            .max_by(|a, b| a.len().partial_cmp(&b.len()).unwrap())
            .filter(|s| s.len() >= 1.5)
    })?;
    let width = entry_door_width();
    let slack = (edge.len() - width - 0.6).max(0.0);
    let offset = jitter(rng, slack);
    let id = *next_id;
    *next_id += 1;
    let (x, y, axis) = if edge.is_horizontal() {
        ((edge.x1 + edge.x2) / 2.0 + offset, edge.y1, Axis::Horizontal)
    } else {
        (edge.x1, (edge.y1 + edge.y2) / 2.0 + offset, Axis::Vertical)
    };
    Some(Opening {
        id,
        kind: OpeningKind::EntryDoor,
        x,
        y,
        axis,
        width,
        from: entry_room,
        to: None,
    })
}

/// The longest exterior part of one edge, if any.
fn longest_part(edge: &Segment, rooms: &[Room], room: &Room) -> Option<Segment> {
    exterior_parts(edge, rooms, room)
        .into_iter()
        .max_by(|a, b| a.len().partial_cmp(&b.len()).unwrap())
}

/// The parts of `edge` not shared with any other room. The edge
/// is axis-aligned; covered intervals are collected along it,
/// sorted, merged, and the gaps are the exterior stretches.
fn exterior_parts(edge: &Segment, rooms: &[Room], room: &Room) -> Vec<Segment> {
    let horizontal = edge.is_horizontal();
    let (lo, hi) = if horizontal {
        (edge.x1.min(edge.x2), edge.x1.max(edge.x2))
    } else {
        (edge.y1.min(edge.y2), edge.y1.max(edge.y2))
    };
    let mut covered: Vec<(f64, f64)> = rooms
        .iter()
        .filter(|o| o.id != room.id)
        .filter_map(|o| shared_edge(&room.rect, &o.rect))
        .filter_map(|s| edge.overlap(&s))
        .map(|o| {
            if horizontal {
                (o.x1.min(o.x2), o.x1.max(o.x2))
            } else {
                (o.y1.min(o.y2), o.y1.max(o.y2))
            }
        })
        .collect();
    covered.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut parts = vec![];
    let mut cur = lo;
    for (a, b) in covered {
        if a > cur + 1e-9 {
            parts.push(part(edge, horizontal, cur, a));
        }
        cur = cur.max(b);
    }
    if hi > cur + 1e-9 {
        parts.push(part(edge, horizontal, cur, hi));
    }
    parts
}

/// One exterior stretch along `edge`, from `a` to `b`.
fn part(edge: &Segment, horizontal: bool, a: f64, b: f64) -> Segment {
    if horizontal {
        Segment::new(a, edge.y1, b, edge.y1)
    } else {
        Segment::new(edge.x1, a, edge.x1, b)
    }
}
