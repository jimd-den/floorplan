//! The basement camera: framing a below-grade space.
//!
//! The basement is an enclosed room with no windows, so the camera
//! stands to the side of the stair — the stepped profile reads,
//! not the flat end — and looks at the stair coming down from the
//! ground floor. The above-grade poses live in `viewpoint.rs`.

use crate::domain::geometry::Axis;
use crate::domain::model::House;
use crate::use_cases::viewpoint::Pose;

/// Stands in the basement to the side of the stair, at standing
/// height above its floor, and looks at the stair coming down:
/// the target is its midpoint, halfway down.
pub fn basement_pose(house: &House) -> Pose {
    let b = &house.basement;
    let r = b.rect;
    let floor_y = -(b.depth as f32);
    let (sx, sz) = b.stair_top;
    let (mx, mz) = match b.stair_axis {
        Axis::Horizontal => (sx + b.stair_run / 2.0, sz),
        Axis::Vertical => (sx, sz + b.stair_run / 2.0),
    };
    let target = [mx as f32, floor_y + (b.depth as f32) / 2.0, mz as f32];
    // Side step across the stair (so the stepped profile reads, not
    // the flat end) plus a step back toward the open end, clamped
    // inside the rect so the eye never lands in a wall.
    let (ex, ez) = match b.stair_axis {
        Axis::Horizontal => {
            let dir = if mx <= r.x + r.w / 2.0 { 2.0 } else { -2.0 };
            ((mx + dir).clamp(r.x + 0.5, r.right() - 0.5), (mz - 2.5).clamp(r.y + 0.5, r.bottom() - 0.5))
        }
        Axis::Vertical => {
            let dir = if mz <= r.y + r.d / 2.0 { 2.0 } else { -2.0 };
            ((mx - 2.5).clamp(r.x + 0.5, r.right() - 0.5), (mz + dir).clamp(r.y + 0.5, r.bottom() - 0.5))
        }
    };
    Pose { eye: [ex as f32, floor_y + 1.7, ez as f32], target }
}
