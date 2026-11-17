//! Framing as mesh: the wall plates (bottom + doubled top) and the
//! ridge beam, as wood boxes. These are the frame's defining
//! members in the mesh. The individual studs and floor joists are
//! NOT separate boxes: they stand inside the walls and below the
//! floors (hidden), one box each would blow the first-person mesh
//! budget for no visible gain — their exact positions ride in the
//! JSON. Rafters are sloped, so they ride in the JSON as positions,
//! not as boxes — `Box3` is axis-aligned. Full LOD only: the frame
//! is detail, not massing.

use crate::domain::framing::{PLATE_H, RAFTER_D, STUD_D, TOP_PLATES};
use crate::domain::mesh::{Box3, Mesh};
use crate::domain::model::House;
use crate::use_cases::materials::FRAMING;
use crate::use_cases::roof::roof_pitch_for;
use crate::use_cases::walls::STOREY_H;

/// Adds the framing members of `house` to `mesh`: the wall plates
/// and the ridge beam. Studs and joists are JSON data, not boxes
/// (see the module preamble).
pub fn add_framing(mesh: &mut Mesh, house: &House) {
    for wf in &house.framing.wall_frames {
        add_wall_frame(mesh, wf);
    }
    add_ridge(mesh, house);
}

/// One framed wall: a bottom plate and a doubled top plate. The
/// studs are JSON data, not boxes (see the module preamble).
fn add_wall_frame(mesh: &mut Mesh, wf: &crate::domain::framing::WallFrame) {
    let seg = wf.segment;
    let base = wf.base;
    let top = base + STOREY_H;
    // Plates run the whole segment: bottom at floor level, top
    // doubled at ceiling height.
    for (y0, y1) in plate_spans(base, top) {
        mesh.add_box(plate_box(&seg, y0, y1), FRAMING);
    }
}

/// The plate spans: one bottom plate, `TOP_PLATES` top plates.
fn plate_spans(base: f32, top: f32) -> Vec<(f32, f32)> {
    let mut spans = vec![(base, base + PLATE_H as f32)];
    for i in 0..TOP_PLATES {
        let y1 = top - i as f32 * PLATE_H as f32;
        spans.push((y1 - PLATE_H as f32, y1));
    }
    spans
}

/// A plate box along a wall segment, `STUD_D` deep, centred on it.
fn plate_box(seg: &crate::domain::geometry::Segment, y0: f32, y1: f32) -> Box3 {
    let d = (STUD_D / 2.0) as f32;
    if seg.is_horizontal() {
        Box3::new([seg.x1 as f32, y0, seg.y1 as f32 - d], [seg.x2 as f32, y1, seg.y1 as f32 + d])
    } else {
        Box3::new([seg.x1 as f32 - d, y0, seg.y1 as f32], [seg.x1 as f32 + d, y1, seg.y2 as f32])
    }
}

/// The ridge beam: a horizontal box along the ridge, hanging below
/// the roof peak. Its top sits one rafter depth below the apex
/// (`peak - RAFTER_D`): the rafters sit on the beam and carry the
/// roof surface, so the beam must never reach the apex — a beam
/// centred on the apex pokes through the roof slopes and out of
/// the gable ends.
fn add_ridge(mesh: &mut Mesh, house: &House) {
    let roof = &house.framing.roof_frame;
    if roof.ridge.len() < 1e-9 {
        return;
    }
    let wall_top = house.floors.len() as f32 * STOREY_H;
    let peak = wall_top + roof.span as f32 * roof_pitch_for(house);
    let d = RAFTER_D as f32;
    let t = (STUD_D / 2.0) as f32;
    let ridge = roof.ridge;
    let b = if ridge.is_horizontal() {
        Box3::new(
            [ridge.x1 as f32, peak - 2.0 * d, ridge.y1 as f32 - t],
            [ridge.x2 as f32, peak - d, ridge.y1 as f32 + t],
        )
    } else {
        Box3::new(
            [ridge.x1 as f32 - t, peak - 2.0 * d, ridge.y1 as f32],
            [ridge.x1 as f32 + t, peak - d, ridge.y2 as f32],
        )
    };
    mesh.add_box(b, FRAMING);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;
    use crate::use_cases::materials::material_table;

    #[test]
    fn the_mesh_carries_framing_members() {
        let house = generate_house(42, Style::Ranch);
        let mut mesh = Mesh::new(material_table());
        add_framing(&mut mesh, &house);
        assert!(mesh.materials.contains(&FRAMING), "no framing faces");
        let framing_faces = mesh.materials.iter().filter(|m| **m == FRAMING).count();
        // Plates (3 per wall), joists, rim, ridge — the major
        // members. Studs are JSON data, not boxes.
        assert!(framing_faces > 50, "too few framing members: {framing_faces}");
        assert_eq!(mesh.validate(), Ok(()));
    }

    #[test]
    fn no_framing_member_reaches_the_roof_apex() {
        // Regression: the ridge beam was centred on the apex, so
        // its top half poked through the roof slopes and out of
        // the gable ends (seed 42 exterior). Every framing vertex
        // must sit below the roof peak height.
        for style in [Style::Ranch, Style::Modern, Style::Colonial] {
            let house = generate_house(42, style);
            let mut mesh = Mesh::new(material_table());
            add_framing(&mut mesh, &house);
            let wall_top = house.floors.len() as f32 * STOREY_H;
            let peak = wall_top + house.framing.roof_frame.span as f32 * roof_pitch_for(&house);
            let highest = mesh
                .faces
                .iter()
                .zip(mesh.materials.iter())
                .filter(|(_, m)| **m == FRAMING)
                .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
                .fold(0.0f32, f32::max);
            assert!(
                highest < peak,
                "{style:?} framing reaches {highest}, roof peak is {peak}"
            );
        }
    }

    #[test]
    fn framing_is_deterministic() {
        let house = generate_house(42, Style::Modern);
        let mut a = Mesh::new(material_table());
        add_framing(&mut a, &house);
        let mut b = Mesh::new(material_table());
        add_framing(&mut b, &house);
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.faces, b.faces);
        assert_eq!(a.materials, b.materials);
    }
}
