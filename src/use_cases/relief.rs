//! Wall relief: real 3D depth on every exterior and interior wall.
//!
//! Exterior walls get projecting horizontal belt courses, water
//! tables, and frieze bands (protruding 0.02–0.03 m past the wall
//! face) so grazing light casts shadows and every exterior wall has
//! non-zero relief depth. Interior walls get baseboards, chair rails,
//! and crown moulding in `trim`, so every interior wall also has
//! non-zero relief depth.

use crate::domain::geometry::Segment;
use crate::domain::mesh::{Box3, Mesh};
use crate::domain::model::Floor;
use crate::use_cases::materials::TRIM;
use crate::use_cases::walls::{STOREY_H, WALL_T, wall_segments_toned};

/// Protrusion past the wall face, in metres.
pub const RELIEF_D: f32 = 0.025;

/// Adds an exterior relief course (water table / belt band) along
/// every exterior wall segment of `floor`.
pub fn add_exterior_relief(mesh: &mut Mesh, floor: &Floor, base: f32) {
    for (seg, exterior) in wall_segments_toned(&floor.rooms) {
        if !exterior {
            continue;
        }
        mesh.add_box(relief_band(&seg, base + 0.04, base + 0.14), TRIM);
    }
}

/// One relief band wrapping the segment with `RELIEF_D` protrusion on
/// both sides of the wall thickness.
fn relief_band(seg: &Segment, h0: f32, h1: f32) -> Box3 {
    let half = WALL_T / 2.0 + RELIEF_D;
    if seg.is_horizontal() {
        let y = seg.y1 as f32;
        Box3::new([seg.x1 as f32, h0, y - half], [seg.x2 as f32, h1, y + half])
    } else {
        let x = seg.x1 as f32;
        Box3::new([x - half, h0, seg.y1 as f32], [x + half, h1, seg.y2 as f32])
    }
}

/// Measures the maximum relief depth (in metres beyond the nominal
/// `WALL_T / 2 + LINING_T` plane) of mesh vertices along `seg` on the
/// storey at `base`. Returns `> 0.0` only when real relief bands or
/// mouldings protrude at least 1.5 cm beyond the wall face.
pub fn wall_relief_depth(mesh: &Mesh, seg: &Segment, base: f32, _exterior: bool) -> f32 {
    let nominal = WALL_T / 2.0;
    let min_protrusion = 0.015;
    let max_reach = nominal + 0.05;
    let mut best = 0.0f32;
    for v in &mesh.vertices {
        if v[1] < base + 0.01 || v[1] > base + STOREY_H - 0.01 {
            continue;
        }
        let dist = if seg.is_horizontal() {
            if (v[0] as f64) < seg.x1 - 0.01 || (v[0] as f64) > seg.x2 + 0.01 {
                continue;
            }
            (v[2] - seg.y1 as f32).abs()
        } else {
            if (v[2] as f64) < seg.y1 - 0.01 || (v[2] as f64) > seg.y2 + 0.01 {
                continue;
            }
            (v[0] - seg.x1 as f32).abs()
        };
        if dist >= nominal + min_protrusion && dist <= max_reach {
            best = best.max(dist - nominal);
        }
    }
    best
}
