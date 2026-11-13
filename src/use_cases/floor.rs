//! One floor: subdivide, assign programs, cap sizes, doors,
//! windows. `build_floor_retiled` retries the build with one more
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
use crate::use_cases::distribute::distribute;
use crate::use_cases::doors::place_doors;
use crate::use_cases::garage::place_garage;
use crate::use_cases::programs::assign_programs_pinned;
use crate::use_cases::repair_window::repair_interior_rooms;
use crate::use_cases::retile::find_retile_target;
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
        let floor = build_floor(rng, rects, joins, level, style, next_room, garage, &overrides);
        let Some(target) = find_retile_target(&floor.rooms, rects, &overrides) else {
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
    let entry = rooms
        .iter()
        .find(|r| r.program == Program::Entry)
        .or_else(|| rooms.iter().find(|r| r.program == Program::Living))
        .map(|r| r.id)
        .unwrap_or(0);
    let mut openings = place_doors(rng, &rooms, &splits, &joins, entry);
    let mut next_id = openings.len() as u32;
    openings.extend(place_windows(rng, &rooms, style, &mut next_id));
    let mut floor = Floor { level, rooms, openings, furniture: vec![], open_doors: vec![] };
    floor.furniture = crate::use_cases::furnish::furnish(rng, &floor);
    floor
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

/// The hall room id, falling back to living when a floor has no hall.
pub fn hall_or_living(floor: &Floor) -> u32 {
    floor
        .rooms
        .iter()
        .find(|r| r.program == Program::Hall)
        .or_else(|| floor.rooms.iter().find(|r| r.program == Program::Living))
        .map(|r| r.id)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::Program as P;
    use crate::use_cases::caps::cap_for;
    use crate::use_cases::footprint::footprint;

    fn floor_rooms(seed: u32, style: Style) -> Floor {
        let mut rng = Rng::new(seed);
        let fp = footprint(&mut rng, style);
        let mut next_room = 0;
        build_floor_retiled(&mut rng, &fp.rects, &fp.joins, 0, style, &mut next_room, fp.garage)
    }

    #[test]
    fn re_tiling_is_deterministic() {
        let a = floor_rooms(97, Style::Modern);
        let b = floor_rooms(97, Style::Modern);
        assert_eq!(a, b);
    }

    #[test]
    fn the_seed_97_modern_bath_is_capped_after_re_tiling() {
        // Seed 97 Modern carried a 53.0 sq m bath (15.6 x 3.4 m).
        let floor = floor_rooms(97, Style::Modern);
        for room in &floor.rooms {
            if let Some(cap) = cap_for(room.program) {
                assert!(
                    room.rect.area() <= cap + 1e-9,
                    "{:?} room {} is {:.1} sq m",
                    room.program,
                    room.id,
                    room.rect.area()
                );
            }
        }
        assert!(floor.rooms.iter().any(|r| r.program == P::Bath), "the floor lost its bath");
    }

    #[test]
    fn a_re_tiled_floor_still_tiles_its_footprint() {
        let mut rng = Rng::new(97);
        let fp = footprint(&mut rng, Style::Modern);
        let floor = floor_rooms(97, Style::Modern);
        let footprint_area: f64 = fp.rects.iter().map(|r| r.area()).sum();
        let rooms_area: f64 = floor
            .rooms
            .iter()
            .filter(|r| r.program != P::Garage)
            .map(|r| r.rect.area())
            .sum();
        assert!((footprint_area - rooms_area).abs() < 1e-6, "area changed");
    }
}
