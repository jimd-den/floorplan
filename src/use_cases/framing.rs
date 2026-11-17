//! build_framing: derive the structural frame of a house from its
//! plan. Wall frames come from the floor's wall segments and the
//! openings on them; floor frames from the room rects; the roof
//! frame from the main footprint rect. The frame is deterministic
//! in the seed, so the same house always frames the same way.

use crate::domain::framing::{
    FloorFrame, FramedOpening, Framing, WallFrame, joist_positions, stud_positions,
};
use crate::domain::geometry::{Axis, Rect, Segment};
use crate::domain::model::{Floor, Footprint, Opening, OpeningKind};
use crate::domain::style::{RoofForm, Style};
use crate::use_cases::roof_frame::roof_frame;
use crate::use_cases::walls::{DOOR_H, WIN_TOP, openings_on, wall_segments};

/// The structural frame of a house: wall frames for every floor,
/// floor frames for every floor, and the roof frame. Built from the
/// floors and the footprint, so `generate_house` can frame the
/// house as it builds it. `style`/`seed` pick the roof form, which
/// sets the ridge length (a hip roof's ridge is shorter than the
/// long side; a gable roof's spans it).
pub fn build_framing(floors: &[Floor], footprint: &Footprint, style: Style, seed: u32) -> Framing {
    let mut wall_frames = vec![];
    let mut floor_frames = vec![];
    for (fi, floor) in floors.iter().enumerate() {
        let base = fi as f32 * crate::use_cases::walls::STOREY_H;
        for seg in wall_segments(&floor.rooms) {
            wall_frames.push(wall_frame(&floor.openings, &seg, base));
        }
        for room in &floor.rooms {
            floor_frames.push(floor_frame(&room.rect, base));
        }
    }
    let roof_frame = roof_frame(footprint, RoofForm::for_house(style, seed));
    Framing { wall_frames, floor_frames, roof_frame }
}

/// One framed wall: studs skipping the openings on the segment,
/// and a framed opening per door/window with its header.
fn wall_frame(openings: &[Opening], seg: &Segment, base: f32) -> WallFrame {
    let on = openings_on(openings, seg);
    let spans: Vec<(f64, f64)> = on.iter().map(|&(lo, hi, _)| (lo, hi)).collect();
    // `openings_on` spans are absolute along the segment axis;
    // stud offsets are relative to the segment start.
    let start = if seg.is_horizontal() { seg.x1 } else { seg.y1 };
    let rel: Vec<(f64, f64)> = spans.iter().map(|&(lo, hi)| (lo - start, hi - start)).collect();
    let studs = stud_positions(seg.len(), &rel);
    let framed = on
        .iter()
        .filter_map(|&(lo, hi, kind)| {
            let opening = openings.iter().find(|o| {
                let c = if seg.is_horizontal() { o.x } else { o.y };
                (c - (lo + hi) / 2.0).abs() < 1e-9 && o.kind == kind
            })?;
            let header_y = match kind {
                OpeningKind::Window => WIN_TOP as f64,
                _ => DOOR_H as f64,
            };
            Some(FramedOpening { opening_id: opening.id, lo: lo - start, hi: hi - start, header_y })
        })
        .collect();
    WallFrame { segment: *seg, studs, openings: framed, base }
}

/// One framed floor: joists spanning the room's short direction,
/// spaced along the long direction.
fn floor_frame(rect: &Rect, base: f32) -> FloorFrame {
    if rect.w >= rect.d {
        FloorFrame { rect: *rect, joists: joist_positions(rect.w), direction: Axis::Vertical, base }
    } else {
        FloorFrame { rect: *rect, joists: joist_positions(rect.d), direction: Axis::Horizontal, base }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;

    fn framed(seed: u32, style: Style) -> Framing {
        let house = generate_house(seed, style);
        build_framing(&house.floors, &house.footprint, style, seed)
    }

    #[test]
    fn every_wall_gets_a_frame_and_no_stud_stands_in_a_doorway() {
        let framing = framed(42, Style::Ranch);
        assert!(!framing.wall_frames.is_empty(), "no wall frames");
        for wf in &framing.wall_frames {
            for o in &wf.openings {
                assert!(
                    !wf.studs.iter().any(|s| *s > o.lo + 1e-9 && *s < o.hi - 1e-9),
                    "a stud stands in opening {} of {wf:?}",
                    o.opening_id
                );
            }
        }
    }

    #[test]
    fn every_room_gets_floor_joists_spanning_its_short_side() {
        let framing = framed(42, Style::Ranch);
        assert!(!framing.floor_frames.is_empty(), "no floor frames");
        for ff in &framing.floor_frames {
            assert!(ff.joists.len() >= 2, "a floor needs at least two joists: {ff:?}");
            assert!((ff.joists[0] - 0.0).abs() < 1e-9, "first joist at the edge");
        }
    }

    #[test]
    fn framing_is_deterministic() {
        let a = framed(42, Style::Modern);
        let b = framed(42, Style::Modern);
        assert_eq!(a, b);
    }

    #[test]
    fn a_door_opening_gets_a_header_at_door_height() {
        let framing = framed(42, Style::Ranch);
        let door = framing
            .wall_frames
            .iter()
            .flat_map(|wf| &wf.openings)
            .find(|o| {
                // A door opening: its header is at door height.
                (o.header_y - DOOR_H as f64).abs() < 1e-9
            });
        assert!(door.is_some(), "no door opening carries a header at door height");
    }
}
