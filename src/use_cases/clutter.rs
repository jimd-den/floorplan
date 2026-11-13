//! Clutter: a light pass of books and dishes on horizontal surfaces.
//!
//! Clutter sits on a table, desk, counter, or nightstand, never on the
//! floor, so it never blocks circulation. It follows a moved piece,
//! like the piece's own mesh.

use crate::domain::furniture::{Furniture, FurnitureKind, FurnitureState};
use crate::domain::mesh::Box3;
use crate::domain::model::Floor;
use crate::use_cases::materials::CLUTTER;

/// The clutter boxes for every surface on a floor, at `base` height.
pub fn clutter_boxes(floor: &Floor, base: f32) -> Vec<(Box3, u32)> {
    let mut out = vec![];
    for piece in &floor.furniture {
        let Some(top) = surface_top(piece.kind) else { continue };
        let (x, z) = position(piece);
        let (w, d) = (piece.rect.w as f32, piece.rect.d as f32);
        let bx = x + 0.15;
        let bz = z + 0.15;
        out.push((Box3::new([bx, base + top, bz], [bx + 0.28, base + top + 0.06, bz + 0.2]), CLUTTER));
        if piece.id % 3 == 0 && w > 0.9 && d > 0.6 {
            out.push((Box3::new([bx + 0.35, base + top, bz], [bx + 0.5, base + top + 0.04, bz + 0.15]), CLUTTER));
        }
    }
    out
}

/// The height of a piece's usable top, or `None` when it has none.
fn surface_top(kind: FurnitureKind) -> Option<f32> {
    match kind {
        FurnitureKind::Table | FurnitureKind::Desk => Some(0.75),
        FurnitureKind::Counter => Some(0.9),
        FurnitureKind::Nightstand => Some(0.5),
        _ => None,
    }
}

/// A moved piece carries its clutter with it.
fn position(piece: &Furniture) -> (f32, f32) {
    match piece.state {
        FurnitureState::Moved { x, y } => (x as f32, y as f32),
        _ => (piece.rect.x as f32, piece.rect.y as f32),
    }
}
