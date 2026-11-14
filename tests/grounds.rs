//! Grounds and exterior fixtures: driveways, front walkways, back-yard
//! fences, trees, and garage doors matching the garage room.

use floorplan::domain::mesh::Mesh;
use floorplan::domain::model::Program;
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::grounds::{
    add_grounds, centerline_boxes, fence_box, sidewalk_box, tree_positions, walkway_box,
};
use floorplan::use_cases::materials::{DRIVEWAY, LEAF, TRUNK, material_table};
use floorplan::use_cases::neighborhood::generate_neighborhood;
use floorplan::use_cases::neighborhood_mesh::{
    LOT_MATERIAL, STREET_MATERIAL, neighborhood_material_table,
};
use floorplan::use_cases::porch::add_garage_door;

fn grounded(seed: u32) -> Mesh {
    let n = generate_neighborhood(seed, (2, 2));
    let mut mesh = Mesh::new(material_table());
    add_grounds(&mut mesh, &n);
    mesh
}

#[test]
fn a_neighborhood_with_garages_gets_driveways() {
    let mesh = grounded(42);
    assert!(mesh.materials.contains(&DRIVEWAY), "no driveway material");
}

#[test]
fn every_lot_gets_one_or_two_trees_inside_its_lot() {
    let n = generate_neighborhood(42, (2, 2));
    let mesh = grounded(42);
    let trees = mesh.materials.iter().filter(|m| **m == LEAF).count() / 12;
    assert!(trees >= n.lots.len(), "a lot has no tree");
    assert!(trees <= 2 * n.lots.len(), "too many trees");
    for index in 0..n.lots.len() {
        let lot = n.lots[index].rect;
        for (x, z) in tree_positions(&n, index) {
            assert!(lot.contains_point(x, z), "tree at ({x},{z}) is outside its lot");
        }
    }
}

#[test]
fn every_house_gets_a_front_walkway_and_every_lot_gets_a_back_fence() {
    let n = generate_neighborhood(42, (2, 2));
    for h in &n.houses {
        let w = walkway_box(&n, h);
        assert!(w.max[0] > w.min[0] && w.max[2] > w.min[2], "empty walkway");
    }
    for index in 0..n.lots.len() {
        let f = fence_box(&n, index);
        assert!((f.max[1] - 1.1).abs() < 0.01, "fence height should be 1.1 m");
    }
    assert!(grounded(42).materials.contains(&TRUNK), "no wooden fence or trunk material");
}

#[test]
fn a_garage_room_gets_an_overhead_garage_door() {
    let house = (0..30u32)
        .flat_map(|seed| Style::all().into_iter().map(move |s| generate_house(seed, s)))
        .find(|h| h.floors[0].rooms.iter().any(|r| r.program == Program::Garage))
        .expect("no house with a garage room");
    let garage = house.floors[0]
        .rooms
        .iter()
        .find(|r| r.program == Program::Garage)
        .unwrap()
        .rect;
    let mut mesh = Mesh::new(material_table());
    add_garage_door(&mut mesh, &house);
    assert!(!mesh.faces.is_empty(), "garage room got no garage door");
    let z_front = garage.bottom() as f32;
    assert!(
        mesh.vertices.iter().any(|v| (v[2] - z_front).abs() < 0.2),
        "garage door is not on the garage front wall"
    );
}

#[test]
fn street_and_lot_material_constants_match_the_neighborhood_table() {
    let table = neighborhood_material_table();
    assert_eq!(table[STREET_MATERIAL as usize].name, "street");
    assert_eq!(table[LOT_MATERIAL as usize].name, "lot");
}

#[test]
fn every_lot_gets_a_sidewalk_and_every_street_gets_centerlines() {
    let n = generate_neighborhood(42, (2, 2));
    for index in 0..n.lots.len() {
        let sw = sidewalk_box(&n, index);
        assert!(sw.max[0] > sw.min[0] && sw.max[2] > sw.min[2], "empty sidewalk");
        assert!(sw.max[1] > 0.02, "sidewalk must be raised above the street");
    }
    for s in &n.streets {
        assert!(!centerline_boxes(s).is_empty(), "street has no centerline dashes");
    }
}

#[test]
fn grounds_are_deterministic() {
    assert_eq!(grounded(42).vertices, grounded(42).vertices);
}
