//! Pinhole camera: world to screen, with near-plane clipping.
//!
//! `Camera::project` returns `None` for a point behind the near plane;
//! `clip_near` keeps the part of a polygon that is in front of it, so a
//! floor passing under the eye still renders. The rasterizer that
//! consumes these projections is `adapters::render`.

/// The near plane, in metres in front of the eye. A vertex closer than
/// this cannot be projected without dividing by ~zero.
pub const NEAR: f32 = 0.05;

/// A pinhole camera. `yaw` 0 looks north (-z); positive `yaw` turns
/// east. Positive `pitch` looks up. `fov` is vertical, in radians.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub eye: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
}

impl Camera {
    /// A camera at `eye` aimed at `target`, 70° vertical field of view.
    pub fn looking_at(eye: [f32; 3], target: [f32; 3]) -> Self {
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let yaw = d[0].atan2(-d[2]);
        let horiz = (d[0] * d[0] + d[2] * d[2]).sqrt();
        Camera { eye, yaw, pitch: d[1].atan2(horiz), fov: 70f32.to_radians() }
    }

    /// The world-space forward vector.
    pub fn forward(&self) -> [f32; 3] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        [sy * cp, sp, -cy * cp]
    }

    /// World point -> camera space (x right, y up, -z forward).
    pub(crate) fn camera_space(&self, p: [f32; 3]) -> [f32; 3] {
        let (dx, dy, dz) = (p[0] - self.eye[0], p[1] - self.eye[1], p[2] - self.eye[2]);
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let x = dx * cy + dz * sy;
        let z = -dx * sy + dz * cy;
        [x, dy * cp + z * sp, -dy * sp + z * cp]
    }

    /// Projects a world point to `(screen_x, screen_y, depth)`, or
    /// `None` when it is at or behind the near plane.
    pub fn project(&self, p: [f32; 3], viewport: Viewport) -> Option<(f32, f32, f32)> {
        let c = self.camera_space(p);
        if c[2] >= -NEAR {
            return None;
        }
        Some(self.project_camera(c, viewport))
    }

    /// Projects a camera-space point that `clip_near` has already put
    /// in front of the near plane.
    pub(crate) fn project_camera(&self, c: [f32; 3], viewport: Viewport) -> (f32, f32, f32) {
        let aspect = viewport.width as f32 / viewport.height as f32;
        let f = 1.0 / (self.fov / 2.0).tan();
        let inv = 1.0 / -c[2];
        let ndc_x = c[0] * f * inv / aspect;
        let ndc_y = c[1] * f * inv;
        let sx = (ndc_x * 0.5 + 0.5) * viewport.width as f32;
        let sy = (0.5 - ndc_y * 0.5) * viewport.height as f32;
        (sx, sy, -c[2])
    }
}

/// Output dimensions in pixels.
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub width: usize,
    pub height: usize,
}

/// Sutherland–Hodgman against the near plane, in camera space. The
/// result is the polygon's front part: 3 or 4 points for a triangle.
pub(crate) fn clip_near(poly: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for i in 0..poly.len() {
        let cur = poly[i];
        let prev = poly[(i + poly.len() - 1) % poly.len()];
        let cur_in = cur[2] <= -NEAR;
        let prev_in = prev[2] <= -NEAR;
        if cur_in != prev_in {
            out.push(crossing(prev, cur));
        }
        if cur_in {
            out.push(cur);
        }
    }
    out
}

/// The point where the edge `a -> b` crosses the near plane.
fn crossing(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    let t = (-NEAR - a[2]) / (b[2] - a[2]);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipping_keeps_a_triangle_fully_in_front() {
        let poly = [[0.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.0, 1.0, -1.0]];
        assert_eq!(clip_near(&poly).len(), 3);
    }

    #[test]
    fn clipping_drops_a_triangle_fully_behind() {
        let poly = [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]];
        assert!(clip_near(&poly).is_empty());
    }

    #[test]
    fn clipping_trims_a_triangle_that_straddles_the_plane() {
        let poly = [[0.0, 0.0, 1.0], [1.0, 0.0, -1.0], [-1.0, 0.0, -1.0]];
        let clipped = clip_near(&poly);
        assert_eq!(clipped.len(), 4, "one corner behind gives a quad");
        assert!(clipped.iter().all(|p| p[2] <= -NEAR + 1e-6));
    }
}
