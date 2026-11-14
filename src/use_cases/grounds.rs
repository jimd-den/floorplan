//! Grounds: driveways on the garage side and trees in the back yard.
//!
//! Deterministic in the neighborhood seed and the lot index. A
//! driveway is a parking pad on the garage's east side, plus a strip
//! to the street when that route does not cross the house. Trees
//! stand near the back edge of the lot, clear of the house and the
//! driveway, so they never block a door or a path.

use crate::domain::geometry::Rect;
use crate::domain::mesh::{Box3, Mesh};
use crate::domain::neighborhood::{Neighborhood, PlacedHouse, house_extent, place_rect};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::materials::{DRIVEWAY, LEAF, TRUNK};

/// Adds every driveway, front walkway, sidewalk, road centerline,
/// back-yard fence, and tree to a neighborhood mesh.
pub fn add_grounds(mesh: &mut Mesh, n: &Neighborhood) {
    for s in &n.streets {
        for dash in centerline_boxes(s) {
            mesh.add_box(dash, DRIVEWAY);
        }
    }
    for h in &n.houses {
        if let Some(garage) = h.house.footprint.garage {
            add_driveway(mesh, n, h, garage);
        }
        mesh.add_box(walkway_box(n, h), DRIVEWAY);
    }
    for index in 0..n.lots.len() {
        mesh.add_box(sidewalk_box(n, index), DRIVEWAY);
        mesh.add_box(fence_box(n, index), TRUNK);
        add_trees(mesh, n, index);
    }
}

/// A pad on the garage's east side, plus a connector to the street.
/// A west-facing house is skipped: a west connector would cross it.
fn add_driveway(mesh: &mut Mesh, n: &Neighborhood, h: &PlacedHouse, garage: Rect) {
    let extent = house_extent(&h.house);
    let g = place_rect(garage, extent, h.rect, h.rotation);
    let street = &n.streets[n.lots[h.lot].street];
    let (px0, pz0, px1, pz1) = (g.right(), g.y, g.right() + 3.0, g.bottom());
    add_pad(mesh, px0, pz0, px1, pz1);
    let cx = (px0 + px1) / 2.0;
    let cz = (pz0 + pz1) / 2.0;
    let (x0, z0, x1, z1) = match h.rotation {
        0 => (cx - 1.5, pz1, cx + 1.5, street.rect.y),
        180 => (cx - 1.5, street.rect.bottom(), cx + 1.5, pz0),
        90 => (px1, cz - 1.5, street.rect.x, cz + 1.5),
        _ => return,
    };
    add_pad(mesh, x0.min(x1), z0.min(z1), x0.max(x1), z0.max(z1));
}

/// A driveway strip just above the lot, below the house floor.
fn add_pad(mesh: &mut Mesh, x0: f64, z0: f64, x1: f64, z1: f64) {
    if x1 - x0 < 0.1 || z1 - z0 < 0.1 {
        return;
    }
    mesh.add_box(
        Box3::new([x0 as f32, -0.02, z0 as f32], [x1 as f32, -0.005, z1 as f32]),
        DRIVEWAY,
    );
}

/// Which side of the lot the street is on.
#[derive(Clone, Copy)]
enum Side {
    North,
    South,
    East,
    West,
}

fn street_side(lot: Rect, street: Rect) -> Side {
    if (lot.y - street.bottom()).abs() < 1e-9 {
        Side::North
    } else if (lot.bottom() - street.y).abs() < 1e-9 {
        Side::South
    } else if (lot.x - street.right()).abs() < 1e-9 {
        Side::West
    } else {
        Side::East
    }
}

/// One or two trees in the back yard of a lot.
fn add_trees(mesh: &mut Mesh, n: &Neighborhood, index: usize) {
    for (x, z) in tree_positions(n, index) {
        add_tree(mesh, x, z);
    }
}

/// The tree positions on one lot: one street-side front-yard tree and
/// one back-yard tree, both clear of the house and inside the lot.
pub fn tree_positions(n: &Neighborhood, index: usize) -> Vec<(f64, f64)> {
    let lot = n.lots[index].rect;
    let side = street_side(lot, n.streets[n.lots[index].street].rect);
    let mut rng = Rng::new(n.seed ^ (index as u32).wrapping_mul(0x9E37_79B9));
    let spread = rng.range_f64(-2.0, 2.0);
    let front = match side {
        Side::North => (clamp(lot.x + 1.8, lot.x + 1.0, lot.right() - 1.0), lot.y + 2.8),
        Side::South => (clamp(lot.right() - 1.8, lot.x + 1.0, lot.right() - 1.0), lot.bottom() - 2.8),
        Side::West => (lot.x + 2.8, clamp(lot.y + 1.8, lot.y + 1.0, lot.bottom() - 1.0)),
        Side::East => (lot.right() - 2.8, clamp(lot.bottom() - 1.8, lot.y + 1.0, lot.bottom() - 1.0)),
    };
    let back = match side {
        Side::North => (clamp(lot.x + lot.w / 2.0 + spread, lot.x + 1.0, lot.right() - 1.0), lot.bottom() - 1.5),
        Side::South => (clamp(lot.x + lot.w / 2.0 + spread, lot.x + 1.0, lot.right() - 1.0), lot.y + 1.5),
        Side::West => (lot.right() - 1.5, clamp(lot.y + lot.d / 2.0 + spread, lot.y + 1.0, lot.bottom() - 1.0)),
        Side::East => (lot.x + 1.5, clamp(lot.y + lot.d / 2.0 + spread, lot.y + 1.0, lot.bottom() - 1.0)),
    };
    vec![front, back]
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    v.max(lo).min(hi)
}

/// A trunk and one leaf box, rooted just above the lot.
fn add_tree(mesh: &mut Mesh, x: f64, z: f64) {
    let (x, z) = (x as f32, z as f32);
    mesh.add_box(Box3::new([x - 0.15, 0.0, z - 0.15], [x + 0.15, 2.2, z + 0.15]), TRUNK);
    mesh.add_box(Box3::new([x - 0.7, 1.9, z - 0.7], [x + 0.7, 3.5, z + 0.7]), LEAF);
}

/// The walkway box from a placed house's front to its street.
pub fn walkway_box(n: &Neighborhood, h: &PlacedHouse) -> Box3 {
    let street = n.streets[n.lots[h.lot].street].rect;
    let cx = h.rect.x + h.rect.w / 2.0;
    let cz = h.rect.y + h.rect.d / 2.0;
    let (x0, z0, x1, z1) = match h.rotation {
        0 => (cx - 0.55, h.rect.bottom(), cx + 0.55, street.y),
        180 => (cx - 0.55, street.bottom(), cx + 0.55, h.rect.y),
        90 => (h.rect.right(), cz - 0.55, street.x, cz + 0.55),
        _ => (street.right(), cz - 0.55, h.rect.x, cz + 0.55),
    };
    let (xa, xb) = (x0.min(x1), x0.max(x1).max(x0.min(x1) + 0.5));
    let (za, zb) = (z0.min(z1), z0.max(z1).max(z0.min(z1) + 0.5));
    Box3::new([xa as f32, -0.018, za as f32], [xb as f32, -0.004, zb as f32])
}

/// The back-yard fence box along the rear edge of a lot.
pub fn fence_box(n: &Neighborhood, index: usize) -> Box3 {
    let lot = n.lots[index].rect;
    let side = street_side(lot, n.streets[n.lots[index].street].rect);
    let (x0, z0, x1, z1) = match side {
        Side::North => (lot.x + 0.5, lot.bottom() - 0.3, lot.right() - 0.5, lot.bottom() - 0.2),
        Side::South => (lot.x + 0.5, lot.y + 0.2, lot.right() - 0.5, lot.y + 0.3),
        Side::West => (lot.right() - 0.3, lot.y + 0.5, lot.right() - 0.2, lot.bottom() - 0.5),
        Side::East => (lot.x + 0.2, lot.y + 0.5, lot.x + 0.3, lot.bottom() - 0.5),
    };
    Box3::new([x0 as f32, 0.0, z0 as f32], [x1 as f32, 1.1, z1 as f32])
}

/// A raised 1.6 m concrete sidewalk along a lot's street-fronting edge.
pub fn sidewalk_box(n: &Neighborhood, index: usize) -> Box3 {
    let lot = n.lots[index].rect;
    let side = street_side(lot, n.streets[n.lots[index].street].rect);
    let (x0, z0, x1, z1) = match side {
        Side::North => (lot.x, lot.y, lot.right(), lot.y + 1.6),
        Side::South => (lot.x, lot.bottom() - 1.6, lot.right(), lot.bottom()),
        Side::West => (lot.x, lot.y, lot.x + 1.6, lot.bottom()),
        Side::East => (lot.right() - 1.6, lot.y, lot.right(), lot.bottom()),
    };
    Box3::new([x0 as f32, -0.015, z0 as f32], [x1 as f32, 0.04, z1 as f32])
}

/// Dashed centreline boxes along a street.
pub fn centerline_boxes(street: &crate::domain::neighborhood::Street) -> Vec<Box3> {
    let r = street.rect;
    let mut out = Vec::new();
    if street.horizontal {
        let cz = (r.y + r.d / 2.0) as f32;
        let mut x = r.x + 2.0;
        while x + 2.5 <= r.right() - 1.0 {
            out.push(Box3::new([x as f32, 0.002, cz - 0.09], [(x + 2.5) as f32, 0.012, cz + 0.09]));
            x += 7.5;
        }
        if out.is_empty() {
            let cx = (r.x + r.w / 2.0) as f32;
            out.push(Box3::new([cx - 1.0, 0.002, cz - 0.09], [cx + 1.0, 0.012, cz + 0.09]));
        }
    } else {
        let cx = (r.x + r.w / 2.0) as f32;
        let mut z = r.y + 2.0;
        while z + 2.5 <= r.bottom() - 1.0 {
            out.push(Box3::new([cx - 0.09, 0.002, z as f32], [cx + 0.09, 0.012, (z + 2.5) as f32]));
            z += 7.5;
        }
        if out.is_empty() {
            let cz = (r.y + r.d / 2.0) as f32;
            out.push(Box3::new([cx - 0.09, 0.002, cz - 1.0], [cx + 0.09, 0.012, cz + 1.0]));
        }
    }
    out
}
