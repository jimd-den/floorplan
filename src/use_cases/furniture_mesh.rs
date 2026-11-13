//! Furniture mesh: a piece is a composed box mesh (a table is a
//! top plus four legs, a bed a frame plus a mattress) built into
//! the same domain `Mesh` as the house, with its own material
//! ids. The lamp shade uses a bright material while the lamp is
//! on, so a toggle changes the mesh. See ADR 0003 and spec §4.2.

use crate::domain::furniture::{Furniture, FurnitureKind, FurnitureState};
use crate::use_cases::furniture_parts::{cabinet_boxes, chair_boxes, sofa_boxes, table_boxes};
use crate::domain::mesh::{Box3, Material};
use crate::domain::model::Floor;

/// First furniture material id in `material_table` (after the 5
/// structural and 10 program materials).
pub const FURNITURE_BASE: u32 = 15;
/// The bright shade material for a lamp that is on.
pub const LAMP_ON: u32 = 27;

/// One material per furniture kind, in `FurnitureKind` order.
pub fn furniture_materials() -> Vec<Material> {
    vec![
        Material { name: "bed", color: [0.45, 0.35, 0.65] },
        Material { name: "nightstand", color: [0.55, 0.40, 0.25] },
        Material { name: "table", color: [0.60, 0.45, 0.30] },
        Material { name: "chair", color: [0.50, 0.40, 0.30] },
        Material { name: "sofa", color: [0.70, 0.30, 0.30] },
        Material { name: "lamp", color: [0.90, 0.85, 0.60] },
        Material { name: "desk", color: [0.55, 0.45, 0.30] },
        Material { name: "tub", color: [0.95, 0.95, 0.95] },
        Material { name: "sink", color: [0.90, 0.90, 0.95] },
        Material { name: "shelf", color: [0.65, 0.50, 0.35] },
        Material { name: "cabinet", color: [0.60, 0.55, 0.45] },
        Material { name: "counter", color: [0.75, 0.75, 0.75] },
        Material { name: "lamp_on", color: [1.0, 0.95, 0.55] },
    ]
}

/// The material id for a furniture kind.
pub fn furniture_material(kind: FurnitureKind) -> u32 {
    const ORDER: [FurnitureKind; 12] = [
        FurnitureKind::Bed,
        FurnitureKind::Nightstand,
        FurnitureKind::Table,
        FurnitureKind::Chair,
        FurnitureKind::Sofa,
        FurnitureKind::Lamp,
        FurnitureKind::Desk,
        FurnitureKind::Tub,
        FurnitureKind::Sink,
        FurnitureKind::Shelf,
        FurnitureKind::Cabinet,
        FurnitureKind::Counter,
    ];
    FURNITURE_BASE + ORDER.iter().position(|&k| k == kind).unwrap() as u32
}

/// The composed boxes for every piece on a floor, at `base` height.
/// A moved piece is drawn at its stored position, not its placed
/// one — the mesh follows the interaction state.
pub fn furniture_boxes(floor: &Floor, base: f32) -> Vec<(Box3, u32)> {
    let mut out = vec![];
    for piece in &floor.furniture {
        let (x, z) = piece_position(piece);
        let (w, d) = (piece.rect.w as f32, piece.rect.d as f32);
        out.extend(piece_boxes(piece, base, x, z, w, d));
    }
    out
}

/// A moved piece is drawn where it was moved to; every other piece
/// at its placed rect.
fn piece_position(piece: &Furniture) -> (f32, f32) {
    match piece.state {
        FurnitureState::Moved { x, y } => (x as f32, y as f32),
        _ => (piece.rect.x as f32, piece.rect.y as f32),
    }
}

/// The boxes for one piece, in its kind material. `x, z` is the
/// north-west corner, `w, d` the footprint, all in metres.
fn piece_boxes(piece: &Furniture, base: f32, x: f32, z: f32, w: f32, d: f32) -> Vec<(Box3, u32)> {
    let m = furniture_material(piece.kind);
    let b = |x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32, mat: u32| {
        (Box3::new([x0, base + y0, z0], [x1, base + y1, z1]), mat)
    };
    match piece.kind {
        FurnitureKind::Bed => vec![
            b(x, 0.0, z, x + w, 0.35, z + d, m),
            b(x + 0.05, 0.35, z + 0.05, x + w - 0.05, 0.55, z + d - 0.05, m),
        ],
        FurnitureKind::Nightstand => vec![b(x, 0.0, z, x + w, 0.5, z + d, m)],
        FurnitureKind::Table => table_boxes(x, z, w, d, m, 0.75, base),
        FurnitureKind::Chair => chair_boxes(piece, x, z, w, d, m, base),
        FurnitureKind::Sofa => sofa_boxes(piece, x, z, w, d, m, base),
        FurnitureKind::Lamp => {
            let shade = if piece.state == FurnitureState::On { LAMP_ON } else { m };
            vec![
                b(x, 0.0, z, x + w, 0.05, z + d, m),
                b(x + w / 2.0 - 0.025, 0.05, z + d / 2.0 - 0.025, x + w / 2.0 + 0.025, 1.5, z + d / 2.0 + 0.025, m),
                b(x, 1.5, z, x + w, 1.75, z + d, shade),
            ]
        }
        FurnitureKind::Desk => {
            let mut v = table_boxes(x, z, w, d, m, 0.75, base);
            v.push(b(x + 0.05, 0.0, z + 0.05, x + 0.4, 0.5, z + d - 0.05, m));
            v
        }
        FurnitureKind::Tub => vec![b(x, 0.0, z, x + w, 0.55, z + d, m)],
        FurnitureKind::Sink => vec![
            b(x + w / 2.0 - 0.075, 0.0, z + d / 2.0 - 0.075, x + w / 2.0 + 0.075, 0.8, z + d / 2.0 + 0.075, m),
            b(x, 0.8, z, x + w, 0.95, z + d, m),
        ],
        FurnitureKind::Shelf => vec![b(x, 0.0, z, x + w, 1.8, z + d, m)],
        FurnitureKind::Cabinet => cabinet_boxes(piece, x, z, w, d, m, base),
        FurnitureKind::Counter => vec![b(x, 0.0, z, x + w, 0.9, z + d, m)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;

    #[test]
    fn a_furnished_house_has_furniture_geometry() {
        use crate::use_cases::extrude::extrude;
        let house = generate_house(42, Style::Modern);
        assert!(!house.floors[0].furniture.is_empty(), "seed 42 has no furniture");
        let mesh = extrude(&house);
        assert!(mesh.material_table.iter().any(|m| m.name == "bed"), "no bed material");
        let furn_faces = mesh
            .faces
            .iter()
            .zip(mesh.materials.iter())
            .filter(|(_, m)| **m >= FURNITURE_BASE)
            .count();
        assert!(furn_faces > 0, "no furniture faces");
        assert_eq!(mesh.validate(), Ok(()), "furnished mesh invalid");
    }

    #[test]
    fn furniture_materials_follow_kind_order() {
        assert_eq!(furniture_material(FurnitureKind::Bed), FURNITURE_BASE);
        assert_eq!(furniture_material(FurnitureKind::Counter), FURNITURE_BASE + 11);
        assert_eq!(furniture_materials().len(), 13);
    }

    #[test]
    fn a_table_is_a_top_plus_four_legs() {
        let house = generate_house(42, Style::Modern);
        let boxes = furniture_boxes(&house.floors[0], 0.0);
        let table = house.floors[0]
            .furniture
            .iter()
            .find(|p| p.kind == FurnitureKind::Table)
            .expect("no table in seed 42");
        let (x, z) = (table.rect.x as f32, table.rect.y as f32);
        let (w, d) = (table.rect.w as f32, table.rect.d as f32);
        let ours: Vec<_> = boxes
            .iter()
            .filter(|(b, m)| *m == furniture_material(FurnitureKind::Table)
                && b.min[0] >= x - 0.01
                && b.max[0] <= x + w + 0.01
                && b.min[2] >= z - 0.01
                && b.max[2] <= z + d + 0.01)
            .collect();
        assert_eq!(ours.len(), 5, "a table is a top plus four legs, got {}", ours.len());
    }

    #[test]
    fn a_moved_chair_is_drawn_where_it_was_moved_to() {
        let mut house = generate_house(42, Style::Modern);
        let chair = house.floors[0]
            .furniture
            .iter_mut()
            .find(|p| p.kind == FurnitureKind::Chair)
            .expect("no chair in seed 42");
        chair.state = FurnitureState::Moved { x: 1.0, y: 1.0 };
        chair.rect = crate::domain::geometry::Rect::new(1.0, 1.0, 0.45, 0.45);
        let boxes = furniture_boxes(&house.floors[0], 0.0);
        let m = furniture_material(FurnitureKind::Chair);
        assert!(
            boxes.iter().any(|(b, mat)| *mat == m && (b.min[0] - 1.0).abs() < 0.01 && (b.min[2] - 1.0).abs() < 0.01),
            "no chair box at the moved position"
        );
    }

    #[test]
    fn a_lamp_on_uses_the_bright_shade_material() {
        let mut house = generate_house(42, Style::Modern);
        let lamp = house.floors[0]
            .furniture
            .iter_mut()
            .find(|p| p.kind == FurnitureKind::Lamp)
            .expect("no lamp in seed 42");
        lamp.state = FurnitureState::On;
        let boxes = furniture_boxes(&house.floors[0], 0.0);
        assert!(boxes.iter().any(|(_, m)| *m == LAMP_ON), "no bright shade for a lamp that is on");
    }
}
