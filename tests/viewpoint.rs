//! Camera poses: exterior, interior, and street.

use floorplan::domain::geometry::Rect;
use floorplan::domain::placement::house_extent;
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

/// The eye stands on a street when its plan position is within
/// half the street width (4 m) of the street's centerline.
fn on_street(street: &floorplan::domain::neighborhood::Street, eye: [f32; 3]) -> bool {
    let p = floorplan::domain::curve::Pt::new(eye[0] as f64, eye[2] as f64);
    street
        .samples
        .points
        .iter()
        .any(|&q| q.dist(p) < 4.0)
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
fn the_interior_stands_in_a_room_not_the_garage() {
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

    assert!(on_street(&n.streets[busiest], eye), "eye must stand on the busiest street");
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
        n.streets.iter().any(|s| on_street(s, eye)),
        "eye must stand on a street"
    );
    assert!((eye[1] - 1.7).abs() < 0.01, "eye at standing height");
    assert!((target[1] - 1.7).abs() < 0.01, "level look");
    assert!(
        (eye[0] - target[0]).abs() > 1.0 || (eye[2] - target[2]).abs() > 1.0,
        "must look along the street"
    );
}

#[test]
fn the_basement_stands_in_the_basement_and_looks_at_the_stair() {
    for seed in [0u32, 7, 42, 106] {
        for style in Style::all() {
            let house = generate_house(seed, style);
            let b = &house.basement;
            let Pose { eye, target } = house_pose(&house, View::Basement);
            let floor_y = -(b.depth as f32);
            // The eye stands in the basement, at standing height
            // above its floor.
            assert!(inside(b.rect, eye), "seed {seed} {style:?}: eye outside the basement");
            assert!(
                (eye[1] - (floor_y + 1.7)).abs() < 0.01,
                "seed {seed} {style:?}: eye at {eye:?}, not 1.7 m above the basement floor"
            );
            // The eye stands off the stair's axis, so the stepped
            // profile reads instead of the flat end.
            let (sx, sz) = b.stair_top;
            let off_axis = match b.stair_axis {
                floorplan::domain::geometry::Axis::Horizontal => (eye[2] as f64 - sz).abs(),
                floorplan::domain::geometry::Axis::Vertical => (eye[0] as f64 - sx).abs(),
            };
            assert!(
                off_axis >= 1.0,
                "seed {seed} {style:?}: eye {eye:?} is on the stair axis, not to the side"
            );
            // The target is the stair coming down: in plan it sits
            // on the stair run, and its height is between the
            // basement floor and grade.
            let (tx, tz) = (target[0] as f64, target[2] as f64);
            let (sx, sz) = b.stair_top;
            let on_run = match b.stair_axis {
                floorplan::domain::geometry::Axis::Horizontal => {
                    tx >= sx - 0.01 && tx <= sx + b.stair_run + 0.01 && (tz - sz).abs() < 0.6
                }
                floorplan::domain::geometry::Axis::Vertical => {
                    tz >= sz - 0.01 && tz <= sz + b.stair_run + 0.01 && (tx - sx).abs() < 0.6
                }
            };
            assert!(on_run, "seed {seed} {style:?}: target {target:?} is not on the stair");
            assert!(
                target[1] > floor_y && target[1] < 0.0,
                "seed {seed} {style:?}: target height {} not between floor and grade",
                target[1]
            );
            assert_ne!(eye, target);
        }
    }
}
