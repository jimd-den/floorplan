//! Garage placement: sliding the garage along the east wall
//! so it orphans as few habitable rooms as possible.

use crate::domain::geometry::{Rect, Segment};
use crate::domain::model::{Program, Room};
use crate::use_cases::placement::windowless;
use crate::use_cases::quant;

/// Picks the garage's y so the fewest habitable rooms end up
/// windowless. Candidates run the 0.1 m grid; the footprint's own
/// gy wins ties, so the garage only moves when it helps.
pub(crate) fn choose_garage_y(rooms: &[Room], garage: Rect, depth: f64) -> f64 {
    let mut best_y = garage.y;
    let mut best_score = windowless_habitable(rooms, garage, garage.y);
    let mut gy = 0.0;
    while gy <= depth - garage.d + 1e-9 {
        let score = windowless_habitable(rooms, garage, gy);
        if score < best_score {
            best_score = score;
            best_y = gy;
        }
        gy = quant(gy + 0.1);
    }
    best_y
}

/// How many habitable rooms would be windowless with the garage at `gy`.
fn windowless_habitable(rooms: &[Room], garage: Rect, gy: f64) -> usize {
    let rects: Vec<Rect> = rooms
        .iter()
        .map(|r| {
            if r.program == Program::Garage {
                Rect::new(garage.x, gy, garage.w, garage.d)
            } else {
                r.rect
            }
        })
        .collect();
    rooms
        .iter()
        .enumerate()
        .filter(|(i, r)| r.program.is_habitable() && windowless(&rects, *i))
        .count()
}

/// Slides the garage along the east wall to the spot that orphans
/// the fewest habitable rooms, and moves its join with it.
pub(crate) fn place_garage(rooms: &mut [Room], joins: &mut [Segment], rects: &[Rect], garage: Rect) {
    let depth = rects.iter().map(|r| r.bottom()).fold(0.0, f64::max);
    let gy = choose_garage_y(rooms, garage, depth);
    if let Some(room) = rooms.iter_mut().find(|r| r.program == Program::Garage) {
        room.rect = Rect::new(garage.x, gy, garage.w, garage.d);
    }
    let east = rects.iter().map(|r| r.right()).fold(f64::MIN, f64::max);
    if let Some(join) = joins
        .iter_mut()
        .find(|j| (j.x1 - east).abs() < 1e-9 && (j.x2 - east).abs() < 1e-9)
    {
        *join = Segment::new(east, gy, east, gy + garage.d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::Program;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    #[test]
    fn garage_slides_off_the_only_window_of_a_room() {
        // A bedroom boxed in on three sides; its only exterior edge
        // is the east wall, and a garage at y 4.5 would cover all
        // but 0.5 m of it.
        let rooms = vec![
            room(0, Program::Living, Rect::new(0.0, 0.0, 15.0, 4.0)),
            room(1, Program::Bath, Rect::new(0.0, 4.0, 10.0, 4.0)),
            room(2, Program::Bedroom, Rect::new(10.0, 4.0, 5.0, 4.0)),
            room(3, Program::Study, Rect::new(0.0, 8.0, 15.0, 4.0)),
            room(4, Program::Garage, Rect::new(15.0, 4.5, 6.0, 6.0)),
        ];
        let garage = Rect::new(15.0, 4.5, 6.0, 6.0);
        let gy = choose_garage_y(&rooms, garage, 12.0);
        assert!(gy < 4.0, "garage stayed on the bedroom's window: gy {gy}");
        assert_eq!(windowless_habitable(&rooms, garage, gy), 0);
    }

    #[test]
    fn garage_stays_put_when_nothing_is_orphaned() {
        let rooms = vec![
            room(0, Program::Living, Rect::new(0.0, 0.0, 10.0, 8.0)),
            room(1, Program::Bedroom, Rect::new(10.0, 0.0, 5.0, 8.0)),
            room(2, Program::Garage, Rect::new(15.0, 1.0, 6.0, 6.0)),
        ];
        assert_eq!(choose_garage_y(&rooms, Rect::new(15.0, 1.0, 6.0, 6.0), 8.0), 1.0);
    }
}
