//! Re-tiling targets: which over-cap room to fix, and where.
//!
//! The loop in `floor` rebuilds a floor with one more alcove per
//! round; this module finds the next room that needs one. A room
//! qualifies when its footprint rect admits an alcove and that
//! rect is not already pinned — one rect holds one pin.

use crate::domain::geometry::Rect;
use crate::domain::model::{Program, Room};
use crate::domain::room_size::is_room_sized;
use crate::use_cases::alcove::alcove_fits_n;
use crate::use_cases::caps::cap_for;

/// One alcove to add: the footprint rect (by index) and the
/// program its alcove leaf is pinned to.
pub struct RetileTarget {
    pub rect: usize,
    pub program: Program,
}

/// The rooms on a floor that exceed their program cap, in order.
/// ADR 0014: a hall or entry that is dimensionally a habitable
/// room (area >= 6.5 sq m and short side >= 2.13 m) also
/// qualifies — it must be re-tiled to a foyer-sized alcove
/// leaf, not emitted as a ballroom hall or a room-sized entry.
/// Only Bath is a capped target: a pinned Study alcove leaf
/// (6.0 sq m) is sub-floor for a study, so the choke turns it
/// into laundry and wastes the leaf a bath needs (seed 103
/// Colonial). A study is fixed by the assignment's fitting-leaf
/// rule, not by pinning. Capped rooms (Bath) come first: a rect
/// holds only a few alcove strips, and a hard cap must never
/// lose its strip to a hall.
pub fn over_cap_rooms(rooms: &[Room]) -> Vec<&Room> {
    let mut out: Vec<&Room> = rooms
        .iter()
        .filter(|r| {
            (r.program == Program::Bath
                && cap_for(r.program).is_some_and(|cap| r.rect.area() > cap + 1e-9))
                || ((r.program == Program::Hall || r.program == Program::Entry)
                    && is_room_sized(&r.rect))
        })
        .collect();
    out.sort_by_key(|r| cap_for(r.program).is_none());
    out
}

/// The footprint rect containing `room`, if any. Rooms from a cap
/// split stay inside their rect; the garage room is in none.
pub fn containing_rect(rects: &[Rect], room: &Room) -> Option<usize> {
    rects.iter().position(|r| {
        room.rect.x >= r.x - 1e-9
            && room.rect.y >= r.y - 1e-9
            && room.rect.right() <= r.right() + 1e-9
            && room.rect.bottom() <= r.bottom() + 1e-9
    })
}

/// The next re-tiling target: the first over-cap room whose rect
/// admits one more alcove strip. A rect can hold several strips,
/// one per pin, so a rect already pinned for one bath can take a
/// second. When the room's own rect cannot take a strip — a thin
/// wing that can never hold a bath leaf — a different rect that
/// admits an alcove is pinned instead: the pinned leaf satisfies
/// the bath requirement, so the bounded bath count leaves this
/// room a non-bath program. Rooms in a footprint with no
/// alcove-admitting rect at all are skipped here; they are the
/// genuine residual.
pub fn find_retile_target(
    rooms: &[Room],
    rects: &[Rect],
    pinned: &[(usize, Program)],
) -> Option<RetileTarget> {
    // ADR 0015: when the floor already has a fitting bath, a
    // room-sized hall/entry outranks an over-cap bath for the
    // next strip — the bath falls back to demotion to laundry,
    // and the floor keeps its fitting bath. Without a fitting
    // bath, baths keep priority: a house must keep a bath.
    let mut targets = over_cap_rooms(rooms);
    let has_fitting_bath = rooms.iter().any(|r| {
        r.program == Program::Bath
            && cap_for(r.program).is_some_and(|cap| r.rect.area() <= cap + 1e-9)
    });
    if has_fitting_bath {
        targets.sort_by_key(|r| r.program == Program::Bath);
    }
    for room in targets {
        let Some(rect) = containing_rect(rects, room) else { continue };
        let strips = pinned.iter().filter(|&&(r, _)| r == rect).count() + 1;
        if alcove_fits_n(&rects[rect], strips) {
            return Some(RetileTarget { rect, program: room.program });
        }
        for (i, r) in rects.iter().enumerate() {
            let strips_i = pinned.iter().filter(|&&(rr, _)| rr == i).count() + 1;
            if alcove_fits_n(r, strips_i) {
                return Some(RetileTarget { rect: i, program: room.program });
            }
        }
    }
    None
}
