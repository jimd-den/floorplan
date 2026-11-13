//! Lighting and atmospheric shading for the software rasterizer.
//!
//! Combines a camera headlight with directional sun and a sky-ground
//! hemisphere term so perpendicular walls meeting at a 45° view
//! angle still read as a corner, and blends distant street geometry
//! into the horizon haze.

const SUN_DIR: [f32; 3] = [0.60, 0.64, 0.48];
const HORIZON_RGB: [f32; 3] = [196.0, 208.0, 220.0];

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Computes the surface shade multiplier for a face normal and camera
/// forward vector.
pub fn face_shade(normal: [f32; 3], forward: [f32; 3]) -> f32 {
    let head = dot(normal, forward).abs();
    let sun = dot(normal, SUN_DIR).max(0.0);
    let hemi = 0.5 * (normal[1] + 1.0);
    (0.46 + 0.44 * head + 0.14 * sun + 0.08 * hemi).clamp(0.0, 1.0)
}

/// Applies distance haze beyond 35 m toward the horizon colour.
pub fn apply_haze(rgb: [u8; 3], depth: f32) -> [u8; 3] {
    if depth <= 35.0 {
        return rgb;
    }
    let t = ((depth - 35.0) / 180.0).clamp(0.0, 0.35);
    [
        (rgb[0] as f32 * (1.0 - t) + HORIZON_RGB[0] * t) as u8,
        (rgb[1] as f32 * (1.0 - t) + HORIZON_RGB[1] * t) as u8,
        (rgb[2] as f32 * (1.0 - t) + HORIZON_RGB[2] * t) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perpendicular_walls_at_forty_five_degrees_have_distinct_shade() {
        let forward = [-std::f32::consts::FRAC_1_SQRT_2, 0.0, -std::f32::consts::FRAC_1_SQRT_2];
        let east_wall = [1.0, 0.0, 0.0];
        let north_wall = [-1.0, 0.0, 0.0];
        assert!(
            (face_shade(east_wall, forward) - face_shade(north_wall, forward)).abs() > 0.05,
            "perpendicular/opposite walls must not flatten to the same shade"
        );
    }

    #[test]
    fn upward_surfaces_receive_more_sky_bounce_than_downward_ones() {
        let forward = [0.0, 0.0, -1.0];
        assert!(face_shade([0.0, 1.0, 0.0], forward) > face_shade([0.0, -1.0, 0.0], forward));
    }

    #[test]
    fn distant_fragments_blend_toward_the_horizon_while_near_ones_stay_crisp() {
        let base = [80, 100, 60];
        assert_eq!(apply_haze(base, 12.0), base);
        let far = apply_haze(base, 120.0);
        assert!(far[0] > base[0] && far[2] > base[2], "far fragment did not haze: {far:?}");
    }
}
