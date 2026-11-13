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

/// Adds every driveway and tree to a neighborhood mesh.
pub fn add_grounds(mesh: &mut Mesh, n: &Neighborhood) {
    for h in &n.houses {
        if let Some(garage) = h.house.footprint.garage {
            add_driveway(mesh, n, h, garage);
        }
    }
    for index in 0..n.lots.len() {
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

/// The tree positions on one lot: in the back yard, clear of the
/// house's back setback, spread deterministically along the lot.
pub fn tree_positions(n: &Neighborhood, index: usize) -> Vec<(f64, f64)> {
    let lot = n.lots[index].rect;
    let side = street_side(lot, n.streets[n.lots[index].street].rect);
    let mut rng = Rng::new(n.seed ^ (index as u32).wrapping_mul(0x9E37_79B9));
    let count = 1 + rng.range_usize(2);
    let spread = rng.range_f64(-2.5, 2.5);
    let mut out = vec![];
    for i in 0..count {
        let offset = spread + i as f64 * 2.5;
        let (x, z) = match side {
            Side::North => (clamp(lot.x + lot.w / 2.0 + offset, lot.x + 1.0, lot.right() - 1.0), lot.bottom() - 1.5),
            Side::South => (clamp(lot.x + lot.w / 2.0 + offset, lot.x + 1.0, lot.right() - 1.0), lot.y + 1.5),
            Side::West => (lot.right() - 1.5, clamp(lot.y + lot.d / 2.0 + offset, lot.y + 1.0, lot.bottom() - 1.0)),
            Side::East => (lot.x + 1.5, clamp(lot.y + lot.d / 2.0 + offset, lot.y + 1.0, lot.bottom() - 1.0)),
        };
        out.push((x, z));
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::materials::material_table;
    use crate::use_cases::neighborhood::generate_neighborhood;

    fn grounded(seed: u32) -> Mesh {
        let n = generate_neighborhood(seed, (2, 2));
        let mut mesh = Mesh::new(material_table());
        add_grounds(&mut mesh, &n);
        mesh
    }

    fn leaf_faces(mesh: &Mesh) -> usize {
        mesh.materials.iter().filter(|m| **m == LEAF).count()
    }

    #[test]
    fn a_neighborhood_with_garages_gets_driveways() {
        let mesh = grounded(42);
        assert!(mesh.materials.contains(&DRIVEWAY), "no driveway material");
    }

    #[test]
    fn every_lot_gets_one_or_two_trees() {
        let n = generate_neighborhood(42, (2, 2));
        let mesh = grounded(42);
        let trees = leaf_faces(&mesh) / 12;
        assert!(trees >= n.lots.len(), "a lot has no tree");
        assert!(trees <= 2 * n.lots.len(), "too many trees");
    }

    #[test]
    fn every_tree_stands_inside_a_lot() {
        let n = generate_neighborhood(42, (2, 2));
        for index in 0..n.lots.len() {
            let lot = n.lots[index].rect;
            for (x, z) in tree_positions(&n, index) {
                assert!(lot.contains_point(x, z), "tree at ({x},{z}) is outside its lot");
            }
        }
    }

    #[test]
    fn grounds_are_deterministic() {
        assert_eq!(grounded(42).vertices, grounded(42).vertices);
    }
}
