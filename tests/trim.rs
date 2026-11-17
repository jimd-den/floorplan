//! Trim bands stop at door openings instead of crossing them.

use floorplan::domain::geometry::{Axis, Rect};
use floorplan::domain::model::{Floor, Opening, OpeningKind, Program, Room};
use floorplan::domain::skeleton::Skeleton;
use floorplan::use_cases::trim::baseboard_boxes;

fn floor_with_door_on_north_wall() -> Floor {
    Floor {
        level: 0,
        rooms: vec![Room { id: 0, program: Program::Living, rect: Rect::new(0.0, 0.0, 4.0, 4.0) }],
        openings: vec![Opening {
            id: 0,
            kind: OpeningKind::Door,
            x: 2.0,
            y: 0.0,
            axis: Axis::Horizontal,
            width: 0.9,
            from: 0,
            to: None,
        }],
        furniture: vec![],
        open_doors: vec![],
        skeleton: Skeleton::default(),
    }
}

#[test]
fn no_trim_band_crosses_a_door() {
    let boxes = baseboard_boxes(&floor_with_door_on_north_wall(), 0.0);
    // The door is centred at x=2 on the north wall; the chair
    // rail (0.90-0.94) and the baseboard (0-0.1) both sit below
    // the 2.1 m door head, so neither may cover the door centre.
    assert!(!boxes.iter().any(|(b, _)| b.contains([2.0, 0.92, 0.09])), "chair rail crosses the door");
    assert!(!boxes.iter().any(|(b, _)| b.contains([2.0, 0.05, 0.09])), "baseboard crosses the door");
    // The crown (2.62-2.68) is above the door head, so it runs
    // continuously over the door.
    assert!(boxes.iter().any(|(b, _)| b.contains([2.0, 2.65, 0.09])), "crown should run over the door");
    // The bands still run up to the door on both sides.
    assert!(boxes.iter().any(|(b, _)| b.contains([1.0, 0.92, 0.09])), "chair rail missing before the door");
    assert!(boxes.iter().any(|(b, _)| b.contains([3.0, 0.92, 0.09])), "chair rail missing past the door");
}

#[test]
fn two_doors_on_one_wall_split_the_band_twice() {
    let mut floor = floor_with_door_on_north_wall();
    floor.openings.push(Opening {
        id: 1,
        kind: OpeningKind::Door,
        x: 3.2,
        y: 0.0,
        axis: Axis::Horizontal,
        width: 0.8,
        from: 0,
        to: None,
    });
    let boxes = baseboard_boxes(&floor, 0.0);
    assert!(!boxes.iter().any(|(b, _)| b.contains([2.0, 0.92, 0.09])), "chair rail crosses the first door");
    assert!(!boxes.iter().any(|(b, _)| b.contains([3.2, 0.92, 0.09])), "chair rail crosses the second door");
    assert!(boxes.iter().any(|(b, _)| b.contains([2.7, 0.92, 0.09])), "chair rail missing between the doors");
}
