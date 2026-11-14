//! Procedural 3D surface textures for the software rasterizer.
//!
//! Pure functions of material name and world-space `(x, y, z)` in
//! metres: turf blades on grass lots, aggregate grain on asphalt
//! streets, expansion joints on concrete sidewalks, horizontal
//! clapboard courses on exterior siding, staggered shingles on
//! pitched roofs, and wood planks or ceramic tiles on interior floors.

fn hash2(x: f32, z: f32) -> f32 {
    let xi = (x * 100.0).floor() as i32 as u32;
    let zi = (z * 100.0).floor() as i32 as u32;
    let mut h = xi.wrapping_mul(0x9E37_79B9) ^ zi.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    (h & 0xFFFF) as f32 / 65535.0
}

fn fract_pos(v: f32) -> f32 {
    v - v.floor()
}

/// Returns a deterministic brightness multiplier around `1.0` for the
/// given material at world position `p`.
pub fn texture_scale(material: &str, p: [f32; 3]) -> f32 {
    match material {
        "lot" | "lawn" => {
            let fine = hash2(p[0] * 0.18, p[2] * 0.18) - 0.5;
            let clump = hash2(p[0] * 0.03, p[2] * 0.03) - 0.5;
            let stripe = ((p[0] * 1.4 + p[2] * 0.5).sin()) * 0.04;
            (1.0 + fine * 0.22 + clump * 0.14 + stripe).clamp(0.76, 1.22)
        }
        "street" => {
            let grain = hash2(p[0] * 0.25, p[2] * 0.25) - 0.5;
            (1.0 + grain * 0.14).clamp(0.88, 1.12)
        }
        "driveway" | "porch" => {
            let jx = fract_pos(p[0] / 1.5);
            let jz = fract_pos(p[2] / 1.5);
            if jx < 0.025 || jz < 0.025 {
                return 0.80;
            }
            let grain = hash2(p[0] * 0.2, p[2] * 0.2) - 0.5;
            (1.0 + grain * 0.08).clamp(0.92, 1.08)
        }
        "siding" => {
            let course = fract_pos(p[1] / 0.18);
            if course < 0.12 {
                0.82
            } else {
                let grain = (p[0] * 3.0 + p[2] * 3.0 + p[1] * 12.0).sin() * 0.025;
                (0.95 + 0.08 * course + grain).clamp(0.85, 1.08)
            }
        }
        "roof" => {
            let row = (p[1] / 0.22).floor();
            let cy = fract_pos(p[1] / 0.22);
            let cx = fract_pos((p[0] + p[2] + row * 0.22) / 0.45);
            if cy < 0.14 || cx < 0.06 {
                0.82
            } else {
                let g = hash2(p[0] * 0.3, p[2] * 0.3) - 0.5;
                (0.96 + g * 0.12).clamp(0.86, 1.10)
            }
        }
        "leaf" => {
            let cluster = hash2(p[0] * 0.12 + p[1] * 0.09, p[2] * 0.12) - 0.5;
            (1.0 + cluster * 0.26).clamp(0.78, 1.20)
        }
        "living" | "dining" | "hall" | "entry" | "study" => {
            let board_x = fract_pos(p[0] / 0.16);
            let row = (p[0] / 0.16).floor();
            let board_z = fract_pos((p[2] + row * 0.45) / 1.2);
            if board_x < 0.05 || board_z < 0.02 {
                0.82
            } else {
                let grain = (p[2] * 9.0 + row * 1.7).sin() * 0.04;
                (0.98 + grain).clamp(0.88, 1.08)
            }
        }
        "kitchen" | "bath" => {
            let tx = fract_pos(p[0] / 0.35);
            let tz = fract_pos(p[2] / 0.35);
            if tx < 0.04 || tz < 0.04 {
                0.82
            } else {
                1.02
            }
        }
        "bedroom" => {
            let weave = hash2(p[0] * 0.25, p[2] * 0.25) - 0.5;
            (1.0 + weave * 0.09).clamp(0.92, 1.08)
        }
        "door" | "trunk" | "table" | "chair" | "bed" | "desk" | "shelf" | "cabinet"
        | "nightstand" => {
            let grain = ((p[0] + p[2]) * 24.0 + (p[1] * 8.0).sin() * 2.0).sin() * 0.035;
            (1.0 + grain).clamp(0.92, 1.08)
        }
        "wall" => {
            let stipple = hash2((p[0] + p[2]) * 1.6, p[1] * 1.6) - 0.5;
            (1.0 + stipple * 0.02).clamp(0.98, 1.02)
        }
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grass_texture_varies_across_the_lawn() {
        let a = texture_scale("lot", [1.05, 0.0, 2.15]);
        let b = texture_scale("lot", [1.35, 0.0, 2.85]);
        assert_ne!(a, b, "grass texture must vary spatially");
        assert!((0.75..=1.25).contains(&a));
    }

    #[test]
    fn siding_has_horizontal_clapboard_courses() {
        let shadow = texture_scale("siding", [2.0, 0.18 * 5.0 + 0.01, 0.0]);
        let board = texture_scale("siding", [2.0, 0.18 * 5.0 + 0.12, 0.0]);
        assert!(shadow < board, "clapboard bottom edge must be darker than the board face");
    }

    #[test]
    fn sidewalk_has_concrete_expansion_joints() {
        let joint = texture_scale("driveway", [1.505, 0.0, 0.8]);
        let slab = texture_scale("driveway", [0.75, 0.0, 0.8]);
        assert!(joint < slab, "expansion joint must be darker than the slab centre");
    }

    #[test]
    fn interior_floors_have_wood_planks_and_ceramic_tiles() {
        let seam = texture_scale("living", [0.16 * 3.0 + 0.004, 0.0, 1.0]);
        let plank = texture_scale("living", [0.16 * 3.0 + 0.08, 0.0, 1.0]);
        assert!(seam < plank, "wood plank seam must be darker than the plank");

        let grout = texture_scale("bath", [0.35 * 2.0 + 0.005, 0.0, 0.35 * 2.0 + 0.17]);
        let tile = texture_scale("bath", [0.35 * 2.0 + 0.17, 0.0, 0.35 * 2.0 + 0.17]);
        assert!(grout < tile, "tile grout must be darker than the tile centre");
    }
}
