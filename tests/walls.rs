//! Integration: wall boxes are honest about openings in generated houses.

use floorplan::domain::model::OpeningKind;
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::walls::{
    DOOR_H, STOREY_H, WIN_SILL, WIN_TOP, openings_on, wall_boxes, wall_segments,
};

fn style_for(i: u32) -> Style {
    Style::all()[(i % 6) as usize]
}

#[test]
fn no_wall_box_spans_a_door_or_window_opening_for_200_seeds() {
    for seed in 0..200u32 {
        let house = generate_house(seed, style_for(seed));
        for floor in &house.floors {
            let base = floor.level as f32 * STOREY_H;
            let boxes = wall_boxes(floor, base, 0);
            for o in &floor.openings {
                let mid = match o.kind {
                    OpeningKind::Window => base + (WIN_SILL + WIN_TOP) / 2.0,
                    _ => base + DOOR_H / 2.0,
                };
                let p = [o.x as f32, mid, o.y as f32];
                assert!(
                    !boxes.iter().any(|(b, _)| b.contains(p)),
                    "seed {seed}: a wall box spans {:?} at {p:?}",
                    o.kind
                );
                let lintel = [o.x as f32, base + 2.4, o.y as f32];
                assert!(
                    boxes.iter().any(|(b, _)| b.contains(lintel)),
                    "seed {seed}: no lintel above {:?}",
                    o.kind
                );
            }
        }
    }
}

#[test]
fn every_wall_segment_is_covered_except_at_its_openings_for_200_seeds() {
    for seed in 0..200u32 {
        let house = generate_house(seed, style_for(seed));
        for floor in &house.floors {
            let base = floor.level as f32 * STOREY_H;
            let boxes = wall_boxes(floor, base, 0);
            for seg in wall_segments(&floor.rooms) {
                let spans = openings_on(&floor.openings, &seg);
                let steps = (seg.len() / 0.05).ceil() as usize;
                for k in 0..=steps {
                    let t = seg.x1 + (seg.x2 - seg.x1) * k as f64 / steps as f64;
                    let u = seg.y1 + (seg.y2 - seg.y1) * k as f64 / steps as f64;
                    let along = if seg.is_horizontal() { t } else { u };
                    if spans.iter().any(|(a, b, _)| along >= *a && along <= *b) {
                        continue;
                    }
                    let p = [t as f32, base + 1.35, u as f32];
                    assert!(
                        boxes.iter().any(|(b, _)| b.contains(p)),
                        "seed {seed}: wall gap at {p:?} on {seg:?}"
                    );
                }
            }
        }
    }
}
