//! Camera poses for the screenshot views, as plain data.
//!
//! A pose is an eye and a target in world metres; the adapter turns it
//! into a camera. Choosing the pose is house knowledge (which room to
//! stand in, where the street runs), so it lives here rather than in
//! the renderer.

use crate::domain::model::House;
use crate::domain::neighborhood::{Neighborhood, house_extent};

/// The three views the screenshot adapter can frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Exterior,
    Interior,
    Street,
}

impl View {
    /// The name used on the command line and in the default file name.
    pub fn id(&self) -> &'static str {
        match self {
            View::Exterior => "exterior",
            View::Interior => "interior",
            View::Street => "street",
        }
    }
}

/// An eye and a look-at target, in world metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub eye: [f32; 3],
    pub target: [f32; 3],
}

/// The pose for a house view. `Street` needs a neighborhood, so a
/// house cannot frame it; the caller never asks.
pub fn house_pose(house: &House, view: View) -> Pose {
    match view {
        View::Interior => interior_pose(house),
        _ => exterior_pose(house),
    }
}

/// Stands south-east of the front at standing height and looks at the
/// middle of it. The front is the south edge, so the eye is south of
/// the extent; the lateral offset makes a three-quarter view, so the
/// roof and one side read rather than a flat elevation.
fn exterior_pose(house: &House) -> Pose {
    let extent = house_extent(house);
    let cx = (extent.w / 2.0) as f32;
    let back = (extent.w * 0.55).max(6.0) as f32;
    let side = (extent.w * 0.35).max(2.0) as f32;
    Pose {
        eye: [cx + side, 1.7, extent.bottom() as f32 + back],
        target: [cx, 1.4, (extent.d / 2.0) as f32],
    }
}

/// Stands in the largest ground-floor room and looks at the far
/// corner at chest height. Candidate eyes are the room centre and its
/// four inset corners; the first that is not inside a piece of
/// furniture wins, so the view is never blocked by a wardrobe. The
/// inset shrinks for a small room, so an eye never lands on a wall.
fn interior_pose(house: &House) -> Pose {
    let Some(floor) = house.floors.first() else {
        return Pose { eye: [0.0, 1.7, 0.0], target: [0.0, 1.2, -1.0] };
    };
    let Some(room) = floor.rooms.iter().max_by(|a, b| {
        let area = |r: &crate::domain::model::Room| r.rect.w * r.rect.d;
        area(a).partial_cmp(&area(b)).unwrap_or(std::cmp::Ordering::Equal)
    }) else {
        return Pose { eye: [0.0, 1.7, 0.0], target: [0.0, 1.2, -1.0] };
    };
    let rect = room.rect;
    let inset = 0.7f64.min(rect.w / 2.0 - 0.05).min(rect.d / 2.0 - 0.05).max(0.05);
    let centre = (rect.x + rect.w / 2.0, rect.y + rect.d / 2.0);
    let corners = [
        (rect.x + inset, rect.y + inset),
        (rect.right() - inset, rect.y + inset),
        (rect.x + inset, rect.bottom() - inset),
        (rect.right() - inset, rect.bottom() - inset),
    ];
    let eye = std::iter::once(centre)
        .chain(corners)
        .find(|&(x, z)| !blocked(floor, room.id, x, z))
        .unwrap_or(centre);
    let target = corners
        .iter()
        .max_by(|a, b| dist2(**a, eye).partial_cmp(&dist2(**b, eye)).unwrap())
        .copied()
        .unwrap_or(centre);
    Pose {
        eye: [eye.0 as f32, 1.7, eye.1 as f32],
        target: [target.0 as f32, 1.2, target.1 as f32],
    }
}

/// True when a piece of furniture in this room covers the plan point,
/// with a small margin so the eye is never flush against a side.
fn blocked(floor: &crate::domain::model::Floor, room_id: u32, x: f64, z: f64) -> bool {
    floor.furniture.iter().filter(|f| f.room_id == room_id).any(|f| {
        let r = f.rect;
        x >= r.x - 0.25 && x <= r.right() + 0.25 && z >= r.y - 0.25 && z <= r.bottom() + 0.25
    })
}

fn dist2(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)
}

/// Stands on the street with the most fronting lots and looks along
/// it. A horizontal street runs east-west, so the look is east; a
/// vertical one looks south. Starting near the street's low end gives
/// the longest view down it.
pub fn street_pose(neighborhood: &Neighborhood) -> Pose {
    let busiest = (0..neighborhood.streets.len()).max_by_key(|&i| {
        neighborhood.lots.iter().filter(|l| l.street == i).count()
    });
    let Some(street) = busiest.and_then(|i| neighborhood.streets.get(i)) else {
        return Pose { eye: [0.0, 1.7, 0.0], target: [0.0, 1.7, -1.0] };
    };
    let rect = street.rect;
    let (cx, cz) = (rect.x + rect.w / 2.0, rect.y + rect.d / 2.0);
    let (eye, target) = if street.horizontal {
        ([rect.x + 3.0, cz], [rect.right() - 3.0, cz])
    } else {
        ([cx, rect.y + 3.0], [cx, rect.bottom() - 3.0])
    };
    Pose {
        eye: [eye[0] as f32, 1.7, eye[1] as f32],
        target: [target[0] as f32, 1.7, target[1] as f32],
    }
}
