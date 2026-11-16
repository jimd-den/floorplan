//! Walk check: can a person actually walk from a start room to
//! every other room through door gaps? The skeleton says rooms are
//! connected; this proves it geometrically. A 0.1 m grid BFS over
//! the floor where shared walls block movement except inside
//! door-gap spans. Windows block (their sills); only doors and
//! the entry door are passable. Nothing else blocks: the rooms
//! are empty, so a door gap is the only way through a wall.

use crate::domain::model::Floor;
use crate::use_cases::wall_block::{blocked, walls};

/// The rooms reachable on foot from `start`, through door gaps.
pub fn reachable_rooms(floor: &Floor, start: u32) -> Vec<u32> {
    let walls = walls(floor);
    let Some(start_room) = floor.rooms.iter().find(|r| r.id == start) else {
        return vec![];
    };
    // The grid covers the floor's extent at 0.1 m per cell.
    let min_x = floor.rooms.iter().map(|r| r.rect.x).fold(f64::INFINITY, f64::min);
    let max_x = floor.rooms.iter().map(|r| r.rect.right()).fold(f64::NEG_INFINITY, f64::max);
    let min_y = floor.rooms.iter().map(|r| r.rect.y).fold(f64::INFINITY, f64::min);
    let max_y = floor.rooms.iter().map(|r| r.rect.bottom()).fold(f64::NEG_INFINITY, f64::max);
    let cell = 0.1;
    let nx = ((max_x - min_x) / cell).ceil() as usize + 1;
    let ny = ((max_y - min_y) / cell).ceil() as usize + 1;
    let centre = |i: usize, j: usize| (min_x + (i as f64 + 0.5) * cell, min_y + (j as f64 + 0.5) * cell);
    let walkable = |i: usize, j: usize| {
        let (x, y) = centre(i, j);
        floor.rooms.iter().any(|r| x >= r.rect.x && x <= r.rect.right() && y >= r.rect.y && y <= r.rect.bottom())
            && !blocked(&walls, x, y)
    };
    let (si, sj) = (
        ((start_room.rect.x + start_room.rect.w / 2.0 - min_x) / cell) as usize,
        ((start_room.rect.y + start_room.rect.d / 2.0 - min_y) / cell) as usize,
    );
    let mut seen = vec![false; nx * ny];
    let mut stack = vec![(si, sj)];
    seen[sj * nx + si] = true;
    while let Some((i, j)) = stack.pop() {
        for (ni, nj) in [(i.wrapping_sub(1), j), (i + 1, j), (i, j.wrapping_sub(1)), (i, j + 1)] {
            if ni >= nx || nj >= ny || seen[nj * nx + ni] || !walkable(ni, nj) {
                continue;
            }
            seen[nj * nx + ni] = true;
            stack.push((ni, nj));
        }
    }
    // A room is reachable when any visited cell centre is inside it.
    let mut reached = vec![];
    for room in &floor.rooms {
        let inside = (0..nx).any(|i| {
            (0..ny).any(|j| {
                seen[j * nx + i]
                    && {
                        let (x, y) = centre(i, j);
                        x >= room.rect.x && x <= room.rect.right() && y >= room.rect.y && y <= room.rect.bottom()
                    }
            })
        });
        if inside {
            reached.push(room.id);
        }
    }
    reached
}

/// True when every room on the floor is walkable from `start`.
pub fn is_traversable(floor: &Floor, start: u32) -> bool {
    let reached = reachable_rooms(floor, start);
    floor.rooms.iter().all(|r| reached.contains(&r.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::{Axis, Rect};
    use crate::domain::model::{Opening, OpeningKind, Program, Room};
    use crate::domain::skeleton::Skeleton;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    fn floor_with(rooms: Vec<Room>, openings: Vec<Opening>) -> Floor {
        Floor { level: 0, rooms, openings, furniture: vec![], open_doors: vec![], skeleton: Skeleton::default() }
    }

    fn door(id: u32, x: f64, y: f64, axis: Axis, width: f64, from: u32, to: u32) -> Opening {
        Opening { id, kind: OpeningKind::Door, x, y, axis, width, from, to: Some(to) }
    }

    #[test]
    fn two_rooms_joined_by_a_door_are_both_reachable() {
        let floor = floor_with(
            vec![
                room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0)),
                room(1, Program::Kitchen, Rect::new(4.0, 0.0, 4.0, 4.0)),
            ],
            vec![door(0, 4.0, 2.0, Axis::Vertical, 0.9, 0, 1)],
        );
        assert_eq!(reachable_rooms(&floor, 0), vec![0, 1]);
        assert!(is_traversable(&floor, 0));
    }

    #[test]
    fn two_rooms_with_no_door_are_not_both_reachable() {
        let floor = floor_with(
            vec![
                room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0)),
                room(1, Program::Kitchen, Rect::new(4.0, 0.0, 4.0, 4.0)),
            ],
            vec![],
        );
        assert_eq!(reachable_rooms(&floor, 0), vec![0]);
        assert!(!is_traversable(&floor, 0));
    }

    #[test]
    fn a_window_is_not_a_way_through() {
        let floor = floor_with(
            vec![
                room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0)),
                room(1, Program::Kitchen, Rect::new(4.0, 0.0, 4.0, 4.0)),
            ],
            vec![Opening {
                id: 0,
                kind: OpeningKind::Window,
                x: 4.0,
                y: 2.0,
                axis: Axis::Vertical,
                width: 1.2,
                from: 0,
                to: Some(1),
            }],
        );
        assert_eq!(reachable_rooms(&floor, 0), vec![0]);
        assert!(!is_traversable(&floor, 0));
    }

    #[test]
    fn a_chain_of_three_rooms_is_walkable_end_to_end() {
        let floor = floor_with(
            vec![
                room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0)),
                room(1, Program::Hall, Rect::new(4.0, 0.0, 4.0, 4.0)),
                room(2, Program::Bedroom, Rect::new(8.0, 0.0, 4.0, 4.0)),
            ],
            vec![
                door(0, 4.0, 2.0, Axis::Vertical, 0.9, 0, 1),
                door(1, 8.0, 2.0, Axis::Vertical, 0.9, 1, 2),
            ],
        );
        assert_eq!(reachable_rooms(&floor, 0), vec![0, 1, 2]);
        assert!(is_traversable(&floor, 0));
    }
}
