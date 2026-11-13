//! Integration: wall boxes are honest about openings in generated
//! houses, and exterior walls get the exterior material.

use floorplan::domain::geometry::{Axis, Rect};
use floorplan::domain::mesh::Mesh;
use floorplan::domain::model::{Floor, Opening, OpeningKind, Program, Room};
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::lining::lining_boxes;
use floorplan::use_cases::materials::{WINDOW_FRAME, material_table};
use floorplan::use_cases::trim::add_trim;
use floorplan::use_cases::walls::{
    DOOR_H, STOREY_H, WIN_SILL, WIN_TOP, openings_on, wall_boxes, wall_boxes_toned,
    wall_segments,
};

fn style_for(i: u32) -> Style {
    Style::all()[(i % 6) as usize]
}

fn room(id: u32, program: Program, rect: Rect) -> Room {
    Room { id, program, rect }
}

fn opening(id: u32, kind: OpeningKind, x: f64, y: f64, axis: Axis, width: f64) -> Opening {
    Opening { id, kind, x, y, axis, width, from: 0, to: None }
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

#[test]
fn a_shared_wall_appears_once() {
    let rooms = vec![
        room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0)),
        room(1, Program::Kitchen, Rect::new(4.0, 0.0, 4.0, 4.0)),
    ];
    let segs = wall_segments(&rooms);
    assert_eq!(segs.len(), 7, "two 4x4 rooms share one wall: {segs:?}");
    assert_eq!(segs.iter().filter(|s| s.is_vertical() && (s.x1 - 4.0).abs() < 1e-9).count(), 1);
}

#[test]
fn a_door_leaves_a_gap_and_a_lintel() {
    let floor = Floor {
        level: 0,
        rooms: vec![
            room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0)),
            room(1, Program::Kitchen, Rect::new(4.0, 0.0, 4.0, 4.0)),
        ],
        openings: vec![opening(0, OpeningKind::Door, 4.0, 2.0, Axis::Vertical, 0.9)],
        furniture: vec![],
        open_doors: vec![],
    };
    let boxes = wall_boxes(&floor, 0.0, 0);
    assert!(!boxes.iter().any(|(b, _)| b.contains([4.0, 1.0, 2.0])));
    assert!(boxes.iter().any(|(b, _)| b.contains([4.0, 2.4, 2.0])));
    assert!(boxes.iter().any(|(b, _)| b.contains([4.0, 1.0, 0.5])));
}

#[test]
fn a_window_leaves_a_gap_a_sill_and_a_lintel() {
    let floor = Floor {
        level: 0,
        rooms: vec![room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0))],
        openings: vec![opening(0, OpeningKind::Window, 2.0, 0.0, Axis::Horizontal, 1.2)],
        furniture: vec![],
        open_doors: vec![],
    };
    let boxes = wall_boxes(&floor, 0.0, 0);
    assert!(!boxes.iter().any(|(b, _)| b.contains([2.0, 1.5, 0.0])));
    assert!(boxes.iter().any(|(b, _)| b.contains([2.0, 0.5, 0.0])), "no sill");
    assert!(boxes.iter().any(|(b, _)| b.contains([2.0, 2.4, 0.0])), "no lintel");
}

#[test]
fn a_window_frame_on_a_vertical_wall_stays_at_its_opening() {
    let floor = Floor {
        level: 0,
        rooms: vec![room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0))],
        openings: vec![opening(0, OpeningKind::Window, 0.0, 2.0, Axis::Vertical, 1.2)],
        furniture: vec![],
        open_doors: vec![],
    };
    let mut mesh = Mesh::new(material_table());
    add_trim(&mut mesh, &floor, 0.0);

    let mut zs = vec![];
    for (i, f) in mesh.faces.iter().enumerate() {
        if mesh.materials[i] == WINDOW_FRAME {
            for &v in f {
                zs.push(mesh.vertices[v as usize][2]);
            }
        }
    }
    assert!(!zs.is_empty(), "no window frame");
    assert!(
        zs.iter().all(|&z| (1.4..=2.6).contains(&z)),
        "frame is not at the opening: {zs:?}"
    );
}

#[test]
fn a_lining_covers_the_wall_but_not_a_window() {
    let floor = Floor {
        level: 0,
        rooms: vec![room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0))],
        openings: vec![opening(0, OpeningKind::Window, 2.0, 0.0, Axis::Horizontal, 1.2)],
        furniture: vec![],
        open_doors: vec![],
    };
    let boxes = lining_boxes(&floor, 0.0);
    assert!(boxes.iter().any(|b| b.contains([1.0, 1.5, 0.08])), "solid wall unlined");
    assert!(!boxes.iter().any(|b| b.contains([2.0, 1.5, 0.08])), "window covered");
}

#[test]
fn exterior_walls_get_the_exterior_material() {
    let floor = Floor {
        level: 0,
        rooms: vec![room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0))],
        openings: vec![],
        furniture: vec![],
        open_doors: vec![],
    };
    let boxes = wall_boxes_toned(&floor, 0.0, 0, 1);
    assert_eq!(boxes.len(), 4, "a single room has four walls");
    assert!(boxes.iter().all(|(_, m)| *m == 1), "a single room has only exterior walls");

    let shared = Floor {
        level: 0,
        rooms: vec![
            room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0)),
            room(1, Program::Kitchen, Rect::new(4.0, 0.0, 4.0, 4.0)),
        ],
        openings: vec![],
        furniture: vec![],
        open_doors: vec![],
    };
    let boxes = wall_boxes_toned(&shared, 0.0, 0, 1);
    assert!(boxes.iter().any(|(_, m)| *m == 0), "the shared wall is interior");
    assert!(boxes.iter().any(|(_, m)| *m == 1), "the perimeter is exterior");
}
