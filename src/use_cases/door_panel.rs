//! Door panels: a thin box in each doorway. Closed, the panel
//! fills the gap in the wall; open, it stands perpendicular to
//! the wall, swung into the `from` room from the hinge at the
//! door's min end. The mesh follows the door state, so an
//! `open` command is visible in the render. See ADR 0003.

use crate::domain::mesh::{Box3, Material};
use crate::domain::model::{Floor, Opening, OpeningKind};

/// Material id for a door panel (after the furniture materials).
pub const DOOR: u32 = 28;

pub fn door_material() -> Material {
    Material { name: "door", color: [0.60, 0.45, 0.30] }
}

/// Door height in metres; the wall above is the lintel.
const DOOR_H: f32 = 2.0;
/// Panel thickness in metres.
const THICK: f32 = 0.05;
/// A door at or above this width is two leaves, each hinged at its
/// outer end. A single leaf wider than 1.4 m would be unwieldy.
const DOUBLE_PANEL_AT: f32 = 1.4;

/// The panel box for every door on a floor, at `base` height.
/// A double door contributes two leaves.
pub fn door_panels(floor: &Floor, base: f32) -> Vec<(Box3, u32)> {
    let mut out = vec![];
    for o in &floor.openings {
        if !matches!(o.kind, OpeningKind::Door | OpeningKind::EntryDoor) {
            continue;
        }
        let open = floor.open_doors.contains(&o.id);
        for b in panel_boxes(floor, o, base, open) {
            out.push((b, DOOR));
        }
    }
    out
}

/// The leaf boxes for one door: one leaf, or two when the door is
/// a double door. Each leaf hinges at its outer end.
fn panel_boxes(floor: &Floor, o: &Opening, base: f32, open: bool) -> Vec<Box3> {
    let w = o.width as f32;
    if w < DOUBLE_PANEL_AT {
        return vec![panel_box(floor, o, base, open, -w / 2.0, w / 2.0, true)];
    }
    vec![
        panel_box(floor, o, base, open, -w / 2.0, 0.0, true),
        panel_box(floor, o, base, open, 0.0, w / 2.0, false),
    ]
}

/// The panel box for one leaf, spanning `lo..hi` along the wall
/// from the door centre. Closed it fills its span; open it swings
/// into the `from` room from the hinge at the leaf's outer end
/// (`hinge_at_lo` picks the min end, else the max end).
fn panel_box(floor: &Floor, o: &Opening, base: f32, open: bool, lo: f32, hi: f32, hinge_at_lo: bool) -> Box3 {
    use crate::domain::geometry::Axis;
    let x = o.x as f32;
    let y = o.y as f32;
    if !open {
        return match o.axis {
            Axis::Horizontal => Box3::new([x + lo, base, y - THICK / 2.0], [x + hi, base + DOOR_H, y + THICK / 2.0]),
            Axis::Vertical => Box3::new([x - THICK / 2.0, base, y + lo], [x + THICK / 2.0, base + DOOR_H, y + hi]),
        };
    }
    // Open: swing into the `from` room from the hinge at the leaf's outer end.
    let from = floor.rooms.iter().find(|r| r.id == o.from);
    let into_from = from.map(|r| room_side(r, o)).unwrap_or(1.0);
    let leaf_w = hi - lo;
    match o.axis {
        Axis::Horizontal => {
            let (hx0, hx1) = if hinge_at_lo { (x + lo, x + lo + THICK) } else { (x + hi - THICK, x + hi) };
            let z0 = if into_from > 0.0 { y } else { y - leaf_w };
            Box3::new([hx0, base, z0], [hx1, base + DOOR_H, z0 + leaf_w])
        }
        Axis::Vertical => {
            let (hz0, hz1) = if hinge_at_lo { (y + lo, y + lo + THICK) } else { (y + hi - THICK, y + hi) };
            let x0 = if into_from > 0.0 { x } else { x - leaf_w };
            Box3::new([x0, base, hz0], [x0 + leaf_w, base + DOOR_H, hz1])
        }
    }
}

/// +1 when the `from` room lies in the +coordinate direction from
/// the door's wall, -1 when it lies in the -direction.
fn room_side(room: &crate::domain::model::Room, o: &Opening) -> f32 {
    use crate::domain::geometry::Axis;
    match o.axis {
        Axis::Horizontal => {
            if (o.y - room.rect.y).abs() < 1e-9 {
                1.0
            } else {
                -1.0
            }
        }
        Axis::Vertical => {
            if (o.x - room.rect.x).abs() < 1e-9 {
                1.0
            } else {
                -1.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Axis;
    use crate::domain::model::{Floor, Program, Room};
    use crate::domain::geometry::Rect;
    use crate::domain::skeleton::Skeleton;

    fn floor_with_door(open: bool) -> Floor {
        floor_with_width(open, 0.9)
    }

    fn floor_with_width(open: bool, width: f64) -> Floor {
        let door = Opening {
            id: 5,
            kind: OpeningKind::Door,
            x: 3.0,
            y: 0.0,
            axis: Axis::Horizontal,
            width,
            from: 0,
            to: Some(1),
        };
        Floor {
            level: 0,
            rooms: vec![
                Room { id: 0, program: Program::Living, rect: Rect::new(0.0, 0.0, 6.0, 5.0) },
                Room { id: 1, program: Program::Kitchen, rect: Rect::new(0.0, 5.0, 6.0, 5.0) },
            ],
            openings: vec![door],
            furniture: vec![],
            open_doors: if open { vec![5] } else { vec![] },
            skeleton: Skeleton::default(),
        }
    }

    #[test]
    fn a_closed_door_fills_its_gap() {
        let panels = door_panels(&floor_with_door(false), 0.0);
        assert_eq!(panels.len(), 1);
        let (b, _) = panels[0];
        assert!((b.max[0] - b.min[0] - 0.9).abs() < 1e-6, "closed panel should span the door: {b:?}");
        assert!((b.max[1] - 2.0).abs() < 1e-6, "closed panel should be 2 m tall: {b:?}");
    }

    #[test]
    fn an_open_door_stands_perpendicular_to_its_wall() {
        let panels = door_panels(&floor_with_door(true), 0.0);
        assert_eq!(panels.len(), 1);
        let (b, _) = panels[0];
        // Perpendicular: thin along the wall (x), long into the room (z).
        assert!((b.max[0] - b.min[0] - 0.05).abs() < 1e-6, "open panel should be thin along the wall: {b:?}");
        assert!((b.max[2] - b.min[2] - 0.9).abs() < 1e-6, "open panel should reach into the room: {b:?}");
    }

    #[test]
    fn opening_a_door_moves_its_panel() {
        let closed = door_panels(&floor_with_door(false), 0.0);
        let open = door_panels(&floor_with_door(true), 0.0);
        assert_ne!(closed[0].0, open[0].0, "the panel must move when the door opens");
    }

    #[test]
    fn a_double_door_has_two_panels() {
        let panels = door_panels(&floor_with_width(false, 1.6), 0.0);
        assert_eq!(panels.len(), 2, "a 1.6 m door is two leaves");
        let (a, _) = panels[0];
        let (b, _) = panels[1];
        // Closed, the two leaves together span the opening.
        let span = (b.max[0] - a.min[0]).max(b.max[2] - a.min[2]);
        assert!((span - 1.6).abs() < 1e-6, "two leaves should span 1.6 m: {a:?} {b:?}");
        assert!((a.max[0] - a.min[0] - 0.8).abs() < 1e-6, "each leaf is 0.8 m: {a:?}");
        assert!((b.max[0] - b.min[0] - 0.8).abs() < 1e-6, "each leaf is 0.8 m: {b:?}");
    }

    #[test]
    fn an_open_double_door_swings_both_leaves() {
        let panels = door_panels(&floor_with_width(true, 1.6), 0.0);
        assert_eq!(panels.len(), 2, "an open double door is still two leaves");
        for (b, _) in &panels {
            // Perpendicular: thin along the wall, reaching into the room.
            assert!((b.max[0] - b.min[0] - 0.05).abs() < 1e-6, "open leaf should be thin: {b:?}");
            assert!((b.max[2] - b.min[2] - 0.8).abs() < 1e-6, "open leaf should reach 0.8 m in: {b:?}");
        }
    }
}
