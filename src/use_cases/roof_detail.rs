//! Roof details: hip, gambrel, and mansard forms, plus fascia/gutter
//! eaves and chimney stacks on the main roof slope. Split out of
//! `roof` so both files stay under the 200-line ceiling.

use crate::domain::geometry::Rect;
use crate::domain::mesh::{Box3, Mesh};
use crate::domain::model::House;
use crate::use_cases::materials::{ROOF, SIDING, TRIM};
use crate::use_cases::roof::EAVE;

/// Hip roof: four sloped faces meeting at a ridge segment (or apex when square).
pub(crate) fn add_hip(mesh: &mut Mesh, r: Rect, wall_top: f32, pitch: f32) {
    let (x0, z0, x1, z1) = eave_bounds(r);
    let (w, d) = (x1 - x0, z1 - z0);
    if w >= d {
        let inset = (d / 2.0).min(w / 2.0 - 0.1);
        let rz = (z0 + z1) / 2.0;
        let ry = wall_top + (d / 2.0) * pitch;
        let (rx0, rx1) = (x0 + inset, x1 - inset);
        mesh.add_quad([x0, wall_top, z1], [x1, wall_top, z1], [rx1, ry, rz], [rx0, ry, rz], ROOF);
        mesh.add_quad([x1, wall_top, z0], [x0, wall_top, z0], [rx0, ry, rz], [rx1, ry, rz], ROOF);
        mesh.add_triangle([x0, wall_top, z0], [x0, wall_top, z1], [rx0, ry, rz], ROOF);
        mesh.add_triangle([x1, wall_top, z1], [x1, wall_top, z0], [rx1, ry, rz], ROOF);
    } else {
        let inset = (w / 2.0).min(d / 2.0 - 0.1);
        let rx = (x0 + x1) / 2.0;
        let ry = wall_top + (w / 2.0) * pitch;
        let (rz0, rz1) = (z0 + inset, z1 - inset);
        mesh.add_quad([x0, wall_top, z1], [x0, wall_top, z0], [rx, ry, rz0], [rx, ry, rz1], ROOF);
        mesh.add_quad([x1, wall_top, z0], [x1, wall_top, z1], [rx, ry, rz1], [rx, ry, rz0], ROOF);
        mesh.add_triangle([x0, wall_top, z0], [x1, wall_top, z0], [rx, ry, rz0], ROOF);
        mesh.add_triangle([x1, wall_top, z1], [x0, wall_top, z1], [rx, ry, rz1], ROOF);
    }
}

/// Gambrel roof: steep lower slopes and shallower upper slopes to the ridge.
pub(crate) fn add_gambrel(mesh: &mut Mesh, r: Rect, wall_top: f32, pitch: f32) {
    let (x0, z0, x1, z1) = eave_bounds(r);
    let span = (z1 - z0) / 2.0;
    let kz0 = z0 + span * 0.45;
    let kz1 = z1 - span * 0.45;
    let ky = wall_top + span * pitch * 0.68;
    let rz = (z0 + z1) / 2.0;
    let ry = wall_top + span * pitch;
    mesh.add_quad([x0, wall_top, z1], [x1, wall_top, z1], [x1, ky, kz1], [x0, ky, kz1], ROOF);
    mesh.add_quad([x0, ky, kz1], [x1, ky, kz1], [x1, ry, rz], [x0, ry, rz], ROOF);
    mesh.add_quad([x1, wall_top, z0], [x0, wall_top, z0], [x0, ky, kz0], [x1, ky, kz0], ROOF);
    mesh.add_quad([x1, ky, kz0], [x0, ky, kz0], [x0, ry, rz], [x1, ry, rz], ROOF);
    mesh.add_quad([x0, wall_top, z0], [x0, wall_top, z1], [x0, ky, kz1], [x0, ky, kz0], SIDING);
    mesh.add_triangle([x0, ky, kz0], [x0, ky, kz1], [x0, ry, rz], SIDING);
    mesh.add_quad([x1, wall_top, z1], [x1, wall_top, z0], [x1, ky, kz0], [x1, ky, kz1], SIDING);
    mesh.add_triangle([x1, ky, kz1], [x1, ky, kz0], [x1, ry, rz], SIDING);
}

/// Mansard roof: four steep lower curb slopes and a shallow pitched crown to a ridge.
pub(crate) fn add_mansard(mesh: &mut Mesh, r: Rect, wall_top: f32, pitch: f32) {
    let (x0, z0, x1, z1) = eave_bounds(r);
    let inset = ((x1 - x0).min(z1 - z0) * 0.22).max(0.4);
    let (ix0, iz0, ix1, iz1) = (x0 + inset, z0 + inset, x1 - inset, z1 - inset);
    let curb_y = wall_top + inset * 1.35;
    let ry = curb_y + ((iz1 - iz0) / 2.0) * (pitch * 0.35);
    let rz = (z0 + z1) / 2.0;
    mesh.add_quad([x0, wall_top, z1], [x1, wall_top, z1], [ix1, curb_y, iz1], [ix0, curb_y, iz1], ROOF);
    mesh.add_quad([x1, wall_top, z0], [x0, wall_top, z0], [ix0, curb_y, iz0], [ix1, curb_y, iz0], ROOF);
    mesh.add_quad([x0, wall_top, z0], [x0, wall_top, z1], [ix0, curb_y, iz1], [ix0, curb_y, iz0], ROOF);
    mesh.add_quad([x1, wall_top, z1], [x1, wall_top, z0], [ix1, curb_y, iz0], [ix1, curb_y, iz1], ROOF);
    mesh.add_quad([ix0, curb_y, iz1], [ix1, curb_y, iz1], [ix1, ry, rz], [ix0, ry, rz], ROOF);
    mesh.add_quad([ix1, curb_y, iz0], [ix0, curb_y, iz0], [ix0, ry, rz], [ix1, ry, rz], ROOF);
    mesh.add_triangle([ix0, curb_y, iz0], [ix0, curb_y, iz1], [ix0, ry, rz], ROOF);
    mesh.add_triangle([ix1, curb_y, iz1], [ix1, curb_y, iz0], [ix1, ry, rz], ROOF);
}

/// Adds fascia/gutter eave trim and a chimney stack on the main roof rectangle.
pub(crate) fn add_roof_accessories(mesh: &mut Mesh, house: &House, main: Rect, wall_top: f32, pitch: f32) {
    let (x0, z0, x1, z1) = eave_bounds(main);
    let f = 0.08;
    mesh.add_box(Box3::new([x0, wall_top - f, z0], [x1, wall_top, z0 + f]), TRIM);
    mesh.add_box(Box3::new([x0, wall_top - f, z1 - f], [x1, wall_top, z1]), TRIM);
    let span = (main.w.min(main.d) as f32 + 2.0 * EAVE) / 2.0;
    let ridge_y = wall_top + span * pitch;
    if house.seed.is_multiple_of(2) {
        let cx = (main.x + main.w * 0.28) as f32;
        let cz = (main.y + main.d * 0.35) as f32;
        mesh.add_box(
            Box3::new([cx - 0.3, wall_top, cz - 0.3], [cx + 0.3, ridge_y + 0.45, cz + 0.3]),
            SIDING,
        );
    }
}

fn eave_bounds(r: Rect) -> (f32, f32, f32, f32) {
    (
        r.x as f32 - EAVE,
        r.y as f32 - EAVE,
        r.right() as f32 + EAVE,
        r.bottom() as f32 + EAVE,
    )
}
