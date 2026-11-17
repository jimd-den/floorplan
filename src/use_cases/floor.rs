//! One floor: subdivide, assign programs, cap sizes, skeleton,
//! doors, windows. `build_floor_retiled` retries the build with one more
//! alcove per round until no room exceeds its program cap — the
//! re-tiling loop. Each round pins one new rect; the loop is
//! bounded by the rect count, and exits with the residual when no
//! rect admits an alcove for the rooms that remain.

use crate::domain::geometry::{Rect, Segment};
use crate::domain::model::{Floor, Footprint, Program, Room};
use crate::domain::style::Style;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::alcove::subdivide_with_alcove;
use crate::use_cases::bsp::subdivide;
use crate::use_cases::caps::cap_room_sizes;
use crate::use_cases::demote::demote_over_cap_baths;
use crate::use_cases::distribute::distribute;
use crate::use_cases::doors::place_doors;
use crate::use_cases::entry_shrink::shrink_room_sized_entry;
use crate::use_cases::garage::place_garage;
use crate::use_cases::habitable_floor::enforce_habitable_floor_rooms;
use crate::use_cases::programs::assign_programs_pinned;
use crate::use_cases::repair_window::repair_interior_rooms;
use crate::use_cases::retile::find_retile_target;
use crate::use_cases::skeleton::build_skeleton;
use crate::use_cases::windows::place_windows;

/// Builds a floor, re-tiling until no room exceeds its cap.
pub fn build_floor_retiled(
    rng: &mut Rng,
    rects: &[Rect],
    joins: &[Segment],
    level: u8,
    style: Style,
    next_room: &mut u32,
    garage: Option<Rect>,
) -> Floor {
    let mut overrides: Vec<(usize, Program)> = vec![];
    loop {
        let start_next = *next_room;
        let mut floor = build_floor(rng, rects, joins, level, style, next_room, garage, &overrides);
        let Some(target) = find_retile_target(&floor.rooms, rects, &overrides) else {
            // No rect admits an alcove for the rooms that
            // remain: demote a residual over-cap bath to
            // Laundry rather than emit it over its cap.
            demote_over_cap_baths(&mut floor.rooms);
            return floor;
        };
        overrides.push((target.rect, target.program));
        *next_room = start_next;
    }
}

/// One floor: subdivide, assign programs, cap sizes, doors,
/// windows. `overrides` pins one rect each to an alcove program.
#[allow(clippy::too_many_arguments)]
pub fn build_floor(
    rng: &mut Rng,
    rects: &[Rect],
    joins: &[Segment],
    level: u8,
    style: Style,
    next_room: &mut u32,
    garage: Option<Rect>,
    overrides: &[(usize, Program)],
) -> Floor {
    let room_range = if level == 0 { style.spec().rooms } else { style.spec().upper_rooms };
    let mut joins: Vec<Segment> = joins.to_vec();
    let target = room_range.0 + rng.range_usize(room_range.1 - room_range.0 + 1);
    let counts = distribute(target, rects);
    let first_id = *next_room;
    let mut leaves = vec![];
    let mut splits = vec![];
    let mut pins: Vec<(usize, Program)> = vec![];
    for (ri, (rect, &count)) in rects.iter().zip(counts.iter()).enumerate() {
        if count == 0 {
            continue;
        }
        let base = leaves.len();
        let rect_pins: Vec<Program> =
            overrides.iter().filter(|&&(r, _)| r == ri).map(|&(_, p)| p).collect();
        if !rect_pins.is_empty() {
            let sub = subdivide_with_alcove(rng, *rect, count, &rect_pins);
            for &(li, p) in &sub.pins {
                pins.push((base + li, p));
            }
            leaves.extend(sub.leaves);
            splits.extend(sub.splits);
        } else {
            let sub = subdivide(rng, *rect, count);
            leaves.extend(sub.leaves);
            splits.extend(sub.splits);
        }
    }
    let two_storey = level == 0 && style.spec().stories == 2;
    let programs = assign_programs_pinned(&leaves, level, two_storey, &pins);
    let mut rooms: Vec<Room> = leaves
        .iter()
        .enumerate()
        .map(|(i, r)| Room { id: *next_room + i as u32, program: programs[i], rect: *r })
        .collect();
    *next_room += rooms.len() as u32;
    if let Some(g) = garage {
        rooms.push(Room { id: *next_room, program: Program::Garage, rect: g });
        *next_room += 1;
    }
    if let Some(g) = garage {
        place_garage(&mut rooms, &mut joins, rects, g);
    }
    // The garage can cover a room's only exterior edge; pinned rooms keep their program.
    let pinned_ids: Vec<u32> = pins.iter().map(|&(g, _)| first_id + g as u32).collect();
    repair_interior_rooms(&mut rooms, &pinned_ids);
    // Oversized baths and studies split to their cap; the new split gets a door.
    cap_room_sizes(&mut rooms, &mut splits, next_room);
    // A strip cut can leave a room windowless; repair again.
    repair_interior_rooms(&mut rooms, &pinned_ids);
    // A window swap can land a habitable program on a sub-floor
    // leaf after the assignment-time choke ran; re-choke here.
    enforce_habitable_floor_rooms(&mut rooms);
    // ADR 0015: a room-sized entry that can give depth to its
    // northern neighbour shrinks to a foyer here, before the
    // skeleton is built from the rooms and splits.
    shrink_room_sized_entry(&mut rooms, &mut splits);
    let entry = rooms
        .iter()
        .find(|r| r.program == Program::Entry)
        .or_else(|| rooms.iter().find(|r| r.program == Program::Living))
        .map(|r| r.id)
        .unwrap_or(0);
    // The skeleton is the floor's circulation structure: walls and
    // doors are built from it, not from the raw splits.
    let skeleton = build_skeleton(&rooms, &splits, &joins, entry);
    let mut openings = place_doors(rng, &rooms, &skeleton, entry);
    let mut next_id = openings.len() as u32;
    openings.extend(place_windows(rng, &rooms, style, &openings, &mut next_id));
    Floor { level, rooms, openings, furniture: vec![], open_doors: vec![], skeleton }
}

/// The upper floor covers the main rect (Colonial) or the whole footprint.
pub fn upper_rects(fp: &Footprint, style: Style) -> Vec<Rect> {
    if style == Style::Colonial {
        let main = fp
            .rects
            .iter()
            .max_by(|a, b| a.area().partial_cmp(&b.area()).unwrap())
            .unwrap();
        vec![*main]
    } else {
        fp.rects.clone()
    }
}

