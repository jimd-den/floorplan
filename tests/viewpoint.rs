//! Camera poses: exterior, interior, and street.

use floorplan::domain::geometry::Rect;
use floorplan::domain::neighborhood::house_extent;
use floorplan::domain::model::Program;
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::neighborhood::generate_neighborhood;
use floorplan::use_cases::viewpoint::{house_pose, street_pose, Pose, View};

fn modern() -> floorplan::domain::model::House {
    generate_house(42, Style::Modern)
}

fn inside(r: Rect, p: [f32; 3]) -> bool {
    p[0] >= r.x as f32 && p[0] <= r.right() as f32 && p[2] >= r.y as f32 && p[2] <= r.bottom() as f32
}

#[test]
fn the_exterior_stands_south_of_the_house_and_looks_in() {
    let house = modern();
    let extent = house_extent(&house);
    let Pose { eye, target } = house_pose(&house, View::Exterior);

    assert!((eye[2] as f64) > extent.bottom(), "eye must be south of the front");
    assert!((eye[1] - 1.7).abs() < 0.01, "eye at standing height");
    assert!(inside(extent, target), "target must be inside the footprint");
}

#[test]
fn the_exterior_is_a_three_quarter_view_of_the_centred_footprint() {
    let house = modern();
    let extent = house_extent(&house);
    let Pose { eye, target } = house_pose(&house, View::Exterior);

    assert!((target[0] - extent.w as f32 / 2.0).abs() < 0.01, "target must be centred");
    assert!((eye[0] - extent.w as f32 / 2.0).abs() >= 2.0, "eye must stand off to one side");
    assert!((eye[2] as f64 - extent.bottom()) >= 6.0, "eye must stand back");
}

#[test]
fn the_interior_eye_does_not_stand_inside_furniture() {
    for seed in 0..20u32 {
        let house = generate_house(seed, Style::Modern);
        let Pose { eye, .. } = house_pose(&house, View::Interior);
        for piece in &house.floors[0].furniture {
            let r = piece.rect;
            let inside = eye[0] >= r.x as f32 - 0.2
                && eye[0] <= r.right() as f32 + 0.2
                && eye[2] >= r.y as f32 - 0.2
                && eye[2] <= r.bottom() as f32 + 0.2;
            assert!(!inside, "seed {seed}: eye stands in {:?}", piece.kind);
        }
    }
}

#[test]
fn the_interior_stands_in_a_furnished_room_not_the_garage() {
    for seed in 0..20u32 {
        for style in Style::all() {
            let house = generate_house(seed, style);
            if house.floors[0].rooms.iter().all(|r| r.program != Program::Garage) {
                continue;
            }
            let Pose { eye, .. } = house_pose(&house, View::Interior);
            let room = house.floors[0]
                .rooms
                .iter()
                .find(|r| r.rect.contains_point(eye[0] as f64, eye[2] as f64))
                .expect("interior eye is not inside a room");
            assert_ne!(
                room.program,
                Program::Garage,
                "seed {seed} {style:?}: interior eye is in the garage"
            );
        }
    }
}

#[test]
fn the_street_pose_picks_the_busiest_street() {
    let n = generate_neighborhood(42, (2, 2));
    let busiest = (0..n.streets.len())
        .max_by_key(|&i| n.lots.iter().filter(|l| l.street == i).count())
        .unwrap();
    let Pose { eye, .. } = street_pose(&n);

    assert!(inside(n.streets[busiest].rect, eye), "eye must stand on the busiest street");
}

#[test]
fn the_interior_stands_in_the_largest_ground_room() {
    let house = modern();
    let floor = &house.floors[0];
    let largest = floor
        .rooms
        .iter()
        .max_by(|a, b| {
            (a.rect.w * a.rect.d).partial_cmp(&(b.rect.w * b.rect.d)).unwrap()
        })
        .unwrap();
    let Pose { eye, target } = house_pose(&house, View::Interior);

    assert!(inside(largest.rect, eye), "eye must be in the largest room");
    assert!(inside(largest.rect, target), "target must be in the same room");
    assert!((eye[1] - 1.7).abs() < 0.01, "eye at standing height");
    assert!((target[1] - 1.2).abs() < 0.01, "target at chest height");
    assert_ne!(eye, target);
}

#[test]
fn the_street_pose_stands_on_a_street_and_looks_along_it() {
    let n = generate_neighborhood(42, (2, 2));
    let Pose { eye, target } = street_pose(&n);

    assert!(
        n.streets.iter().any(|s| inside(s.rect, eye)),
        "eye must stand on a street"
    );
    assert!((eye[1] - 1.7).abs() < 0.01, "eye at standing height");
    assert!((target[1] - 1.7).abs() < 0.01, "level look");
    assert!(
        (eye[0] - target[0]).abs() > 1.0 || (eye[2] - target[2]).abs() > 1.0,
        "must look along the street"
    );
}
