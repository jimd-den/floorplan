//! Grounds: driveways on the garage side and trees in the back yard.
//!
//! Deterministic in the neighborhood seed and the lot index. A
//! driveway is a parking pad on the garage's east side, plus a strip
//! to the street when that route does not cross the house. Trees
//! stand near the back edge of the lot, clear of the house and the
//! driveway, so they never block a door or a path. Sidewalks and
//! walkways are not here — they are street ribbons, built by the
//! mesh from the sampled centerlines and the placed house.

use crate::domain::curve::Pt;
use crate::domain::geometry::Rect;
use crate::domain::mesh::{Box3, Mesh};
use crate::domain::neighborhood::{Neighborhood, PlacedHouse, Street};
use crate::domain::placement::{house_corners, house_extent, placed_point};
use crate::domain::polygon::{point_in_poly, polys_overlap};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::materials::{DRIVEWAY, LEAF, TRUNK};
use crate::use_cases::neighborhood::{SIDEWALK_W, STREET_W};
use crate::use_cases::ribbon::add_flat_quad;

/// Adds every driveway, road centerline, back-yard fence, and tree
/// to a neighborhood mesh.
pub fn add_grounds(mesh: &mut Mesh, n: &Neighborhood) {
    for s in &n.streets {
        for dash in centerline_dashes(s) {
            add_flat_quad(mesh, dash[0], dash[1], dash[2], dash[3], 0.01, DRIVEWAY);
        }
    }
    for h in &n.houses {
        if let Some(garage) = h.house.footprint.garage {
            add_driveway(mesh, n, h, garage);
        }
    }
    for index in 0..n.lots.len() {
        add_fence(mesh, n, index);
        add_trees(mesh, n, index);
    }
}

/// A pad on the garage's east side, plus a connector to the street.
/// The connector is skipped when it would cross the house.
fn add_driveway(mesh: &mut Mesh, n: &Neighborhood, h: &PlacedHouse, garage: Rect) {
    let extent = house_extent(&h.house);
    let pad = Rect::new(garage.right(), garage.y, 3.0, garage.d);
    let corners = [
        placed_point(h.center, h.rotation, extent, Pt::new(pad.x, pad.y)),
        placed_point(h.center, h.rotation, extent, Pt::new(pad.right(), pad.y)),
        placed_point(h.center, h.rotation, extent, Pt::new(pad.right(), pad.bottom())),
        placed_point(h.center, h.rotation, extent, Pt::new(pad.x, pad.bottom())),
    ];
    add_flat_quad(mesh, corners[0], corners[1], corners[2], corners[3], -0.01, DRIVEWAY);
    let lot = &n.lots[h.lot];
    let street = &n.streets[lot.street];
    let centre = corners.iter().fold(Pt::new(0.0, 0.0), |a, p| a.add(*p)).scale(0.25);
    let s = nearest_s(street, centre);
    let outward = street.samples.normal_at(s).scale(lot.side as f64);
    let end = street.samples.point_at(s).add(outward.scale(STREET_W / 2.0 + SIDEWALK_W));
    let dir = end.sub(centre).unit();
    let perp = dir.perp().scale(1.5);
    let quad = [centre.add(perp), centre.sub(perp), end.sub(perp), end.add(perp)];
    let house_quad = house_corners(h.center, h.rotation, extent);
    if !polys_overlap(&quad, &house_quad) {
        add_flat_quad(mesh, quad[0], quad[1], quad[2], quad[3], -0.01, DRIVEWAY);
    }
}

/// The arc length of the street sample nearest a point.
fn nearest_s(street: &Street, p: Pt) -> f64 {
    let mut best = (f64::MAX, 0.0f64);
    for (i, q) in street.samples.points.iter().enumerate() {
        let d = q.dist(p);
        if d < best.0 {
            best = (d, street.samples.lengths[i]);
        }
    }
    best.1
}

/// One or two trees in the back yard of a lot.
fn add_trees(mesh: &mut Mesh, n: &Neighborhood, index: usize) {
    for (x, z) in tree_positions(n, index) {
        add_tree(mesh, x, z);
    }
}

/// The tree positions on one lot: one street-side front-yard tree
/// and one back-yard tree, both inside the lot and clear of the
/// house. A lot always gets at least one tree.
pub fn tree_positions(n: &Neighborhood, index: usize) -> Vec<(f64, f64)> {
    let lot = &n.lots[index];
    let street = &n.streets[lot.street];
    let origin = street.samples.point_at(lot.center_s);
    let tangent = street.samples.tangent_at(lot.center_s);
    let outward = street.samples.normal_at(lot.center_s).scale(lot.side as f64);
    let mut rng = Rng::new(n.seed ^ (index as u32).wrapping_mul(0x9E37_79B9));
    let spread = rng.range_f64(-2.0, 2.0);
    let house_quad = n
        .houses
        .iter()
        .find(|h| h.lot == index)
        .map(|h| house_corners(h.center, h.rotation, house_extent(&h.house)));
    let front = origin.add(outward.scale(6.8)).add(tangent.scale(-lot.frontage / 4.0 + spread));
    let back = origin.add(outward.scale(4.0 + lot.depth - 1.5)).add(tangent.scale(spread));
    let mut out: Vec<(f64, f64)> = [front, back]
        .into_iter()
        .filter(|p| clear(*p, lot, house_quad.as_ref()))
        .map(|p| (p.x, p.y))
        .collect();
    if out.is_empty() {
        let fallback = origin.add(outward.scale(4.0 + lot.depth - 1.5));
        out.push((fallback.x, fallback.y));
    }
    out
}

/// True when a point is inside the lot polygon and clear of the house.
fn clear(p: Pt, lot: &crate::domain::neighborhood::Lot, house_quad: Option<&[Pt; 4]>) -> bool {
    point_in_poly(p, &lot.outline)
        && house_quad.is_none_or(|q| !point_in_poly(p, q))
}

/// A trunk and one leaf box, rooted just above the lot.
fn add_tree(mesh: &mut Mesh, x: f64, z: f64) {
    let (x, z) = (x as f32, z as f32);
    mesh.add_box(Box3::new([x - 0.15, 0.0, z - 0.15], [x + 0.15, 2.2, z + 0.15]), TRUNK);
    mesh.add_box(Box3::new([x - 0.7, 1.9, z - 0.7], [x + 0.7, 3.5, z + 0.7]), LEAF);
}

/// The back-yard fence: the lot's back edge as a vertical quad,
/// 1.1 m tall. The mesh adds it facing both ways.
fn add_fence(mesh: &mut Mesh, n: &Neighborhood, index: usize) {
    let q = fence_quad(n, index);
    mesh.add_quad(q[0], q[1], q[2], q[3], TRUNK);
    mesh.add_quad(q[0], q[3], q[2], q[1], TRUNK);
}

/// The back-yard fence quad along the rear edge of a lot: a
/// straight quad between the back arc's two end corners.
pub fn fence_quad(n: &Neighborhood, index: usize) -> [[f32; 3]; 4] {
    let lot = &n.lots[index];
    let m = lot.outline.len() / 2;
    let (br, bl) = (lot.outline[m], lot.outline[lot.outline.len() - 1]);
    [
        [bl.x as f32, 0.0, bl.y as f32],
        [br.x as f32, 0.0, br.y as f32],
        [br.x as f32, 1.1, br.y as f32],
        [bl.x as f32, 1.1, bl.y as f32],
    ]
}

/// Dashed centreline quads along a street, one dash every 7.5 m.
pub fn centerline_dashes(street: &Street) -> Vec<[Pt; 4]> {
    let mut out = Vec::new();
    let mut s = 2.0;
    while s + 2.5 <= street.samples.len() - 1.0 {
        out.push(dash_at(street, s + 1.25));
        s += 7.5;
    }
    if out.is_empty() {
        out.push(dash_at(street, street.samples.len() / 2.0));
    }
    out
}

/// One 2.5 m dash centred `s` metres along the street.
fn dash_at(street: &Street, s: f64) -> [Pt; 4] {
    let c = street.samples.point_at(s);
    let t = street.samples.tangent_at(s).scale(1.25);
    let n = street.samples.normal_at(s).scale(0.09);
    [c.sub(t).add(n), c.add(t).add(n), c.add(t).sub(n), c.sub(t).sub(n)]
}
