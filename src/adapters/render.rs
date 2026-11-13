//! Software rasterizer: `Mesh` + `Camera` -> RGB image.
//!
//! The screenshot adapter's renderer, std only: a z-buffer and flat
//! shading with a headlight. It deliberately does not cull backfaces,
//! so a room's inner walls are visible from inside. It is not the WASM
//! renderer; it exists to make the mesh inspectable without a browser
//! (ADR 0006). Camera math and near-plane clipping are `adapters::camera`.

use crate::adapters::camera::clip_near;
use crate::adapters::shade::{apply_haze, face_shade};
use crate::domain::mesh::Mesh;

pub use crate::adapters::camera::{Camera, Viewport};

/// A rendered RGB image, row-major, top-left first.
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[u8; 3]>,
}

/// The sky a pixel keeps when no triangle covers it. A vertical
/// gradient from horizon haze to zenith blue; a flat fill makes an
/// exterior read as a cardboard cut-out.
fn sky(y: usize, height: usize) -> [u8; 3] {
    let t = if height <= 1 { 0.0 } else { y as f32 / (height - 1) as f32 };
    let horizon = [196.0, 208.0, 220.0];
    let zenith = [96.0, 138.0, 196.0];
    [
        (horizon[0] + (zenith[0] - horizon[0]) * t) as u8,
        (horizon[1] + (zenith[1] - horizon[1]) * t) as u8,
        (horizon[2] + (zenith[2] - horizon[2]) * t) as u8,
    ]
}

/// Renders every face of `mesh` into an image. The z-buffer keeps the
/// nearest fragment; shading is a headlight, so both sides of a wall
/// read.
pub fn render(mesh: &Mesh, camera: &Camera, viewport: Viewport) -> Image {
    let mut image = Image {
        width: viewport.width,
        height: viewport.height,
        pixels: (0..viewport.height).flat_map(|y| vec![sky(y, viewport.height); viewport.width]).collect(),
    };
    let mut depth = vec![f32::INFINITY; viewport.width * viewport.height];
    let forward = camera.forward();
    for (i, face) in mesh.faces.iter().enumerate() {
        let color = mesh
            .material_table
            .get(mesh.materials[i] as usize)
            .map(|m| m.color)
            .unwrap_or([0.8, 0.8, 0.8]);
        let v = [
            mesh.vertices[face[0] as usize],
            mesh.vertices[face[1] as usize],
            mesh.vertices[face[2] as usize],
        ];
        let Some(normal) = normal(v[0], v[1], v[2]) else { continue };
        let shade = face_shade(normal, forward);
        let rgb = [
            (color[0] * shade * 255.0).clamp(0.0, 255.0) as u8,
            (color[1] * shade * 255.0).clamp(0.0, 255.0) as u8,
            (color[2] * shade * 255.0).clamp(0.0, 255.0) as u8,
        ];
        let camera_space = [camera.camera_space(v[0]), camera.camera_space(v[1]), camera.camera_space(v[2])];
        let poly = clip_near(&camera_space);
        if poly.len() < 3 {
            continue;
        }
        let pts: Vec<(f32, f32, f32)> =
            poly.iter().map(|c| camera.project_camera(*c, viewport)).collect();
        for k in 1..pts.len() - 1 {
            let tri = Triangle { p: [pts[0], pts[k], pts[k + 1]], rgb };
            raster(&mut image, &mut depth, &tri);
        }
    }
    image
}

struct Triangle {
    p: [(f32, f32, f32); 3],
    rgb: [u8; 3],
}

fn raster(image: &mut Image, depth: &mut [f32], tri: &Triangle) {
    let area = edge(xy(tri.p[0]), xy(tri.p[1]), xy(tri.p[2]));
    if area.abs() < 1e-6 {
        return;
    }
    let (min_x, max_x) = span(tri.p.iter().map(|q| q.0), image.width);
    let (min_y, max_y) = span(tri.p.iter().map(|q| q.1), image.height);
    let (Some((min_x, max_x)), Some((min_y, max_y))) = (min_x.zip(max_x), min_y.zip(max_y)) else {
        return;
    };
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = edge(xy(tri.p[1]), xy(tri.p[2]), (px, py));
            let w1 = edge(xy(tri.p[2]), xy(tri.p[0]), (px, py));
            let w2 = edge(xy(tri.p[0]), xy(tri.p[1]), (px, py));
            let inside = (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0)
                || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);
            if !inside {
                continue;
            }
            let inv_z = (w0 / tri.p[0].2 + w1 / tri.p[1].2 + w2 / tri.p[2].2) / area;
            let z = 1.0 / inv_z;
            let idx = y * image.width + x;
            if z < depth[idx] {
                depth[idx] = z;
                image.pixels[idx] = apply_haze(tri.rgb, z);
            }
        }
    }
}

/// The clamped integer span of one screen axis, or `None` when the
/// triangle misses the viewport entirely.
fn span(values: impl Iterator<Item = f32>, size: usize) -> (Option<usize>, Option<usize>) {
    let (lo, hi) = values.fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
        (lo.min(v), hi.max(v))
    });
    if hi < 0.0 || lo > (size - 1) as f32 {
        return (None, None);
    }
    let min = (lo.floor().max(0.0) as usize).min(size - 1);
    let max = (hi.ceil().min((size - 1) as f32) as usize).min(size - 1);
    (Some(min), Some(max))
}

fn edge(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

/// The screen-plane part of a projected vertex.
fn xy(p: (f32, f32, f32)) -> (f32, f32) {
    (p.0, p.1)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Option<[f32; 3]> {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = dot(n, n).sqrt();
    if len < 1e-9 { None } else { Some([n[0] / len, n[1] / len, n[2] / len]) }
}
