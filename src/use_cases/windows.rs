//! Windows: one per habitable room on an exterior wall, sized by style.
//!
//! A window sits on a contiguous exterior stretch of a room's edge —
//! the parts no neighbour shares — with 0.3 m of wall left at each
//! end. Large rooms get a second window for Modern.

use crate::domain::geometry::{Axis, Segment, shared_edge};
use crate::domain::model::{Opening, OpeningKind, Room};
use crate::domain::style::Style;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::quant;

/// Places windows for one floor. `next_id` continues the floor's
/// opening numbering (doors come first).
pub fn place_windows(
    rng: &mut Rng,
    rooms: &[Room],
    style: Style,
    next_id: &mut u32,
) -> Vec<Opening> {
    let spec = style.spec();
    let mut out = vec![];
    for room in rooms {
        let stretches = exterior_segments(rooms, room);
        if stretches.is_empty() {
            continue;
        }
        // Fit the longest stretch, never below a 0.6 m slit.
        let mut width = quant(rng.range_f64(spec.window.0, spec.window.1));
        width = width.min((stretches[0].len() - 0.6).max(0.6));
        let mut placed = 0;
        for stretch in stretches.iter() {
            if stretch.len() < width + 0.6 {
                continue;
            }
            let second = placed == 1 && style == Style::Modern && room.rect.area() >= 20.0;
            let required = room.program.is_habitable() || rng.chance(0.5);
            if placed == 0 && !required {
                continue;
            }
            if placed == 1 && !second {
                break;
            }
            out.push(make_window(rng, room, stretch, width, next_id));
            placed += 1;
            if placed >= 2 {
                break;
            }
        }
    }
    out
}

/// The contiguous exterior stretches of a room's edges — the parts no
/// neighbour shares.
fn exterior_segments(rooms: &[Room], room: &Room) -> Vec<Segment> {
    let mut segs: Vec<Segment> = room.rect.edges().to_vec();
    for other in rooms.iter().filter(|r| r.id != room.id) {
        if let Some(shared) = shared_edge(&room.rect, &other.rect) {
            segs = segs.iter().flat_map(|e| subtract(e, &shared)).collect();
        }
    }
    segs.sort_by(|a, b| b.len().partial_cmp(&a.len()).unwrap());
    segs
}

/// The pieces of `e` outside `cut` (both collinear).
fn subtract(e: &Segment, cut: &Segment) -> Vec<Segment> {
    let Some(ov) = e.overlap(cut) else { return vec![*e] };
    let mut out = vec![];
    if e.is_horizontal() {
        let (a, b) = (e.x1.min(e.x2), e.x1.max(e.x2));
        let (c, d) = (ov.x1.min(ov.x2), ov.x1.max(ov.x2));
        if a < c - 1e-9 {
            out.push(Segment::new(a, e.y1, c, e.y1));
        }
        if d < b - 1e-9 {
            out.push(Segment::new(d, e.y1, b, e.y1));
        }
    } else {
        let (a, b) = (e.y1.min(e.y2), e.y1.max(e.y2));
        let (c, d) = (ov.y1.min(ov.y2), ov.y1.max(ov.y2));
        if a < c - 1e-9 {
            out.push(Segment::new(e.x1, a, e.x1, c));
        }
        if d < b - 1e-9 {
            out.push(Segment::new(e.x1, d, e.x1, b));
        }
    }
    out
}

/// One window centred on a stretch, jittered by the seed.
fn make_window(
    rng: &mut Rng,
    room: &Room,
    stretch: &Segment,
    width: f64,
    next_id: &mut u32,
) -> Opening {
    let slack = (stretch.len() - width - 0.6).max(0.0);
    let offset = if slack > 0.0 {
        quant(rng.range_f64(-slack / 2.0, slack / 2.0))
    } else {
        0.0
    };
    let (x, y, axis) = if stretch.is_horizontal() {
        ((stretch.x1 + stretch.x2) / 2.0 + offset, stretch.y1, Axis::Horizontal)
    } else {
        (stretch.x1, (stretch.y1 + stretch.y2) / 2.0 + offset, Axis::Vertical)
    };
    let id = *next_id;
    *next_id += 1;
    Opening { id, kind: OpeningKind::Window, x, y, axis, width, from: room.id, to: None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect;
    use crate::use_cases::bsp::subdivide;
    use crate::use_cases::programs::assign_programs;

    fn house_rooms(seed: u32) -> Vec<Room> {
        let mut rng = Rng::new(seed);
        let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 12.0, 9.0), 6);
        let programs = assign_programs(&sub.leaves, 0, false);
        sub.leaves
            .iter()
            .enumerate()
            .map(|(i, r)| Room { id: i as u32, program: programs[i], rect: *r })
            .collect()
    }

    fn window_rooms(windows: &[Opening]) -> Vec<u32> {
        windows.iter().map(|w| w.from).collect()
    }

    #[test]
    fn every_habitable_room_gets_a_window() {
        let rooms = house_rooms(5);
        let mut rng = Rng::new(5);
        let mut next_id = 0;
        let windows = place_windows(&mut rng, &rooms, Style::Ranch, &mut next_id);
        let got = window_rooms(&windows);
        for room in rooms.iter().filter(|r| r.program.is_habitable()) {
            assert!(got.contains(&room.id), "{:?} room {} has no window", room.program, room.id);
        }
    }

    #[test]
    fn windows_are_on_exterior_walls_only() {
        let rooms = house_rooms(5);
        let mut rng = Rng::new(5);
        let mut next_id = 0;
        let windows = place_windows(&mut rng, &rooms, Style::Ranch, &mut next_id);
        for w in &windows {
            let room = &rooms[w.from as usize];
            for other in rooms.iter().filter(|r| r.id != room.id) {
                if let Some(shared) = shared_edge(&room.rect, &other.rect) {
                    let on_shared = if shared.is_horizontal() {
                        (w.y - shared.y1).abs() < 1e-9 && w.x >= shared.x1 - 1e-9 && w.x <= shared.x2 + 1e-9
                    } else {
                        (w.x - shared.x1).abs() < 1e-9 && w.y >= shared.y1 - 1e-9 && w.y <= shared.y2 + 1e-9
                    };
                    assert!(!on_shared, "window on a shared wall");
                }
            }
        }
    }

    #[test]
    fn modern_windows_are_large() {
        let rooms = house_rooms(5);
        let mut rng = Rng::new(5);
        let mut next_id = 0;
        let windows = place_windows(&mut rng, &rooms, Style::Modern, &mut next_id);
        assert!(!windows.is_empty());
        for w in &windows {
            assert!(w.width >= 1.8, "modern window {} too small", w.width);
        }
    }

    #[test]
    fn cottage_windows_are_small() {
        let rooms = house_rooms(5);
        let mut rng = Rng::new(5);
        let mut next_id = 0;
        let windows = place_windows(&mut rng, &rooms, Style::Cottage, &mut next_id);
        assert!(!windows.is_empty());
        for w in &windows {
            assert!(w.width <= 1.0, "cottage window {} too large", w.width);
        }
    }
}
