//! Composed box parts for furniture pieces: a table is a top
//! plus four legs, a chair a seat plus legs plus a backrest, a
//! sofa a base plus backrest and arms, a cabinet a carcass plus
//! door panels. Split out of `furniture_mesh` so both files stay
//! under the 200-line ceiling.

use crate::domain::furniture::{Facing, Furniture, FurnitureState};
use crate::domain::mesh::Box3;

/// A table: a top plus four legs.
pub(crate) fn table_boxes(x: f32, z: f32, w: f32, d: f32, m: u32, h: f32, base: f32) -> Vec<(Box3, u32)> {
    let b = |x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32| {
        (Box3::new([x0, base + y0, z0], [x1, base + y1, z1]), m)
    };
    let leg = 0.05;
    vec![
        b(x, h - 0.05, z, x + w, h, z + d),
        b(x, 0.0, z, x + leg, h - 0.05, z + leg),
        b(x + w - leg, 0.0, z, x + w, h - 0.05, z + leg),
        b(x, 0.0, z + d - leg, x + leg, h - 0.05, z + d),
        b(x + w - leg, 0.0, z + d - leg, x + w, h - 0.05, z + d),
    ]
}

/// A chair: a seat plus four legs plus a backrest on the side
/// opposite its facing.
pub(crate) fn chair_boxes(piece: &Furniture, x: f32, z: f32, w: f32, d: f32, m: u32, base: f32) -> Vec<(Box3, u32)> {
    let b = |x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32| {
        (Box3::new([x0, base + y0, z0], [x1, base + y1, z1]), m)
    };
    let leg = 0.04;
    let mut v = vec![
        b(x, 0.45, z, x + w, 0.5, z + d),
        b(x, 0.0, z, x + leg, 0.45, z + leg),
        b(x + w - leg, 0.0, z, x + w, 0.45, z + leg),
        b(x, 0.0, z + d - leg, x + leg, 0.45, z + d),
        b(x + w - leg, 0.0, z + d - leg, x + w, 0.45, z + d),
    ];
    let t = 0.05;
    let back = match piece.facing {
        Facing::North => b(x, 0.5, z + d - t, x + w, 1.0, z + d),
        Facing::South => b(x, 0.5, z, x + w, 1.0, z + t),
        Facing::East => b(x + w - t, 0.5, z, x + w, 1.0, z + d),
        Facing::West => b(x, 0.5, z, x + t, 1.0, z + d),
    };
    v.push(back);
    v
}

/// A sofa: a base plus a backrest opposite its facing plus two arms.
pub(crate) fn sofa_boxes(piece: &Furniture, x: f32, z: f32, w: f32, d: f32, m: u32, base: f32) -> Vec<(Box3, u32)> {
    let b = |x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32| {
        (Box3::new([x0, base + y0, z0], [x1, base + y1, z1]), m)
    };
    let t = 0.15;
    let mut v = vec![b(x, 0.0, z, x + w, 0.45, z + d)];
    let (back, a0, a1) = match piece.facing {
        Facing::North => (b(x, 0.45, z + d - t, x + w, 0.95, z + d), b(x, 0.45, z, x + t, 0.65, z + d), b(x + w - t, 0.45, z, x + w, 0.65, z + d)),
        Facing::South => (b(x, 0.45, z, x + w, 0.95, z + t), b(x, 0.45, z, x + t, 0.65, z + d), b(x + w - t, 0.45, z, x + w, 0.65, z + d)),
        Facing::East => (b(x + w - t, 0.45, z, x + w, 0.95, z + d), b(x, 0.45, z, x + w, 0.65, z + t), b(x, 0.45, z + d - t, x + w, 0.65, z + d)),
        Facing::West => (b(x, 0.45, z, x + w, 0.95, z + t), b(x, 0.45, z, x + w, 0.65, z + t), b(x, 0.45, z + d - t, x + w, 0.65, z + d)),
    };
    v.push(back);
    v.push(a0);
    v.push(a1);
    v
}

/// A cabinet: a carcass plus two door panels on its facing side.
/// Open doors stick out, closed doors sit flush.
pub(crate) fn cabinet_boxes(piece: &Furniture, x: f32, z: f32, w: f32, d: f32, m: u32, base: f32) -> Vec<(Box3, u32)> {
    let b = |x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32| {
        (Box3::new([x0, base + y0, z0], [x1, base + y1, z1]), m)
    };
    let mut v = vec![b(x, 0.0, z, x + w, 0.9, z + d)];
    let open = piece.state == FurnitureState::Open;
    let (d0, d1) = match piece.facing {
        Facing::North => {
            let y = if open { z - 0.35 } else { z - 0.02 };
            ((b(x, 0.05, y, x + w / 2.0, 0.85, y + 0.02)), (b(x + w / 2.0, 0.05, y, x + w, 0.85, y + 0.02)))
        }
        Facing::South => {
            let y = if open { z + d + 0.33 } else { z + d };
            ((b(x, 0.05, y, x + w / 2.0, 0.85, y + 0.02)), (b(x + w / 2.0, 0.05, y, x + w, 0.85, y + 0.02)))
        }
        Facing::East => {
            let xx = if open { x + w + 0.33 } else { x + w };
            ((b(xx, 0.05, z, xx + 0.02, 0.85, z + d / 2.0)), (b(xx, 0.05, z + d / 2.0, xx + 0.02, 0.85, z + d)))
        }
        Facing::West => {
            let xx = if open { x - 0.35 } else { x - 0.02 };
            ((b(xx, 0.05, z, xx + 0.02, 0.85, z + d / 2.0)), (b(xx, 0.05, z + d / 2.0, xx + 0.02, 0.85, z + d)))
        }
    };
    v.push(d0);
    v.push(d1);
    v
}
