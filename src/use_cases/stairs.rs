//! Stairs: stepped boxes in the ground hall, rising one storey.
//!
//! Solid steps, 0.25 m treads, 1.0 m wide. Empty for one-storey
//! houses.

use crate::domain::mesh::Box3;
use crate::domain::model::House;
use crate::use_cases::materials::STAIRS;
use crate::use_cases::walls::STOREY_H;

pub fn stairs_boxes(house: &House) -> Vec<(Box3, u32)> {
    let Some((ground_hall, _)) = house.stairs else { return vec![] };
    let Some(hall) = house.floors[0].rooms.iter().find(|r| r.id == ground_hall) else {
        return vec![];
    };
    let r = hall.rect;
    let horizontal = r.w >= r.d;
    let run = if horizontal { r.w } else { r.d };
    let across = if horizontal { r.d } else { r.w };
    let steps = ((run / 0.25).floor() as usize).clamp(2, 12);
    let rise = STOREY_H / steps as f32;
    let tread = run as f32 / steps as f32;
    let width = (across as f32).min(1.0);
    let mut out = vec![];
    for i in 0..steps {
        let top = (i + 1) as f32 * rise;
        let (min, max) = if horizontal {
            let x0 = r.x as f32 + i as f32 * tread;
            let zc = r.y as f32 + r.d as f32 / 2.0;
            ([x0, 0.0, zc - width / 2.0], [x0 + tread, top, zc + width / 2.0])
        } else {
            let z0 = r.y as f32 + i as f32 * tread;
            let xc = r.x as f32 + r.w as f32 / 2.0;
            ([xc - width / 2.0, 0.0, z0], [xc + width / 2.0, top, z0 + tread])
        };
        out.push((Box3::new(min, max), STAIRS));
    }
    out
}
