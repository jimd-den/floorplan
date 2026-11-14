//! Clutter: a light pass of books, dishes, plants, and rugs.
//!
//! Surface clutter sits on a table, desk, counter, or nightstand,
//! never on the floor, so it never blocks circulation. Rugs are flat
//! (2 cm) and may sit under furniture; they never block a door either.
//! A moved piece carries its surface clutter with it.

use crate::domain::furniture::{Furniture, FurnitureKind, FurnitureState};
use crate::domain::mesh::Box3;
use crate::domain::model::{Floor, Program, Room};
use crate::use_cases::materials::CLUTTER;

/// The clutter boxes for every surface and floor on a floor, at
/// `base` height.
pub fn clutter_boxes(floor: &Floor, base: f32) -> Vec<(Box3, u32)> {
    let mut out = vec![];
    for piece in &floor.furniture {
        out.extend(surface_clutter(piece, base));
    }
    for room in &floor.rooms {
        out.extend(rug_boxes(room, base));
    }
    out
}

/// Books, dishes, and a plant on one horizontal surface.
fn surface_clutter(piece: &Furniture, base: f32) -> Vec<(Box3, u32)> {
    let Some(top) = surface_top(piece.kind) else { return vec![] };
    let (x, z) = position(piece);
    let (w, d) = (piece.rect.w as f32, piece.rect.d as f32);
    let bx = x + 0.15;
    let bz = z + 0.15;
    let mut out = vec![(
        Box3::new([bx, base + top, bz], [bx + 0.28, base + top + 0.06, bz + 0.2]),
        CLUTTER,
    )];
    if piece.id.is_multiple_of(3) && w > 0.9 && d > 0.6 {
        out.push((
            Box3::new([bx + 0.35, base + top, bz], [bx + 0.5, base + top + 0.04, bz + 0.15]),
            CLUTTER,
        ));
    }
    if piece.id.is_multiple_of(4) && w > 0.8 && d > 0.5 {
        out.push((
            Box3::new([bx + 0.5, base + top, bz + 0.25], [bx + 0.62, base + top + 0.18, bz + 0.37]),
            CLUTTER,
        ));
    }
    if piece.id.is_multiple_of(5) && w > 1.2 && d > 0.6 {
        out.push((
            Box3::new([bx + 0.7, base + top, bz + 0.1], [bx + 0.9, base + top + 0.07, bz + 0.3]),
            CLUTTER,
        ));
    }
    out
}

/// One flat rug per living space, centred in the room.
fn rug_boxes(room: &Room, base: f32) -> Vec<(Box3, u32)> {
    let r = room.rect;
    let (w, d): (f64, f64) = match room.program {
        Program::Living => (2.6, 1.9),
        Program::Dining => (3.0, 2.0),
        Program::Bedroom => (2.2, 1.4),
        Program::Hall | Program::Entry => (1.4, r.d - 1.0),
        _ => return vec![],
    };
    let w = w.min(r.w - 0.2);
    let d = d.min(r.d - 0.2);
    if w <= 0.2 || d <= 0.2 {
        return vec![];
    }
    let (cx, cz) = (r.x + r.w / 2.0, r.y + r.d / 2.0);
    vec![(
        Box3::new(
            [(cx - w / 2.0) as f32, base, (cz - d / 2.0) as f32],
            [(cx + w / 2.0) as f32, base + 0.02, (cz + d / 2.0) as f32],
        ),
        CLUTTER,
    )]
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
