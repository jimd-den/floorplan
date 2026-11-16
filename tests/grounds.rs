//! Grounds and exterior fixtures: driveways, front walkways to the
//! entry door, back-yard fences, trees, and garage doors matching
//! the garage room. Sidewalks are street ribbons in the
//! neighborhood mesh, not per-lot boxes, so they are checked there.

use floorplan::domain::curve::Pt;
use floorplan::domain::mesh::Mesh;
use floorplan::domain::model::Program;
use floorplan::domain::placement::{house_extent, placed_point};
use floorplan::domain::polygon::point_in_poly;
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::grounds::{add_grounds, centerline_dashes, fence_quad, tree_positions};
use floorplan::use_cases::materials::{DRIVEWAY, LEAF, TRUNK, material_table};
use floorplan::use_cases::neighborhood::{WALKWAY_W, generate_neighborhood};
use floorplan::use_cases::neighborhood_mesh::{
    LOT_MATERIAL, SIDEWALK_MATERIAL, STREET_MATERIAL, neighborhood_material_table, neighborhood_mesh,
};
use floorplan::use_cases::porch::add_garage_door;
use floorplan::use_cases::walkway::entry_door_point;

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
    // One leaf box is 12 faces, all LEAF; houses are not in this mesh.
    let trees = mesh.materials.iter().filter(|m| **m == LEAF).count() / 12;
    assert!(trees >= n.lots.len(), "a lot has no tree");
    assert!(trees <= 2 * n.lots.len(), "too many trees");
    for index in 0..n.lots.len() {
        let lot = &n.lots[index];
        for (x, z) in tree_positions(&n, index) {
            assert!(
                point_in_poly(Pt::new(x, z), &lot.outline),
                "tree at ({x},{z}) is outside its lot"
            );
        }
    }
}

#[test]
fn every_house_gets_a_front_walkway_and_every_lot_gets_a_back_fence() {
    let n = generate_neighborhood(42, (2, 2));
    for h in &n.houses {
        let extent = house_extent(&h.house);
        let door = placed_point(h.center, h.rotation, extent, entry_door_point(&h.house, extent));
        // The walkway's door corners land on the entry door, and
        // the ribbon is WALKWAY_W wide at the sidewalk end too.
        let (d1, d2) = (h.walkway[2].dist(door), h.walkway[3].dist(door));
        assert!(d1.min(d2) < 1.2, "walkway misses its door by {:.2} m", d1.min(d2));
        let w = h.walkway[0].dist(h.walkway[1]);
        assert!((w - WALKWAY_W).abs() < 0.01, "walkway is {w:.2} m wide");
    }
    for index in 0..n.lots.len() {
        let f = fence_quad(&n, index);
        let height = f.iter().map(|v| v[1]).fold(f32::MIN, f32::max);
        assert!((height - 1.1).abs() < 0.01, "fence height should be 1.1 m");
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
fn sidewalks_are_in_the_neighborhood_mesh() {
    let n = generate_neighborhood(42, (2, 2));
    let mesh = neighborhood_mesh(&n);
    assert_eq!(mesh.material_table[SIDEWALK_MATERIAL as usize].name, "sidewalk");
    assert!(
        mesh.materials.contains(&SIDEWALK_MATERIAL),
        "no sidewalk faces in the neighborhood mesh"
    );
    // The sidewalk ribbons sit 0.04 m above the street.
    for (i, f) in mesh.faces.iter().enumerate() {
        if mesh.materials[i] == SIDEWALK_MATERIAL {
            for &vi in f {
                assert!(
                    (mesh.vertices[vi as usize][1] - 0.04).abs() < 1e-6,
                    "sidewalk face {i} is not raised 0.04 m"
                );
            }
        }
    }
}

#[test]
fn every_street_gets_centerline_dashes() {
    let n = generate_neighborhood(42, (2, 2));
    for s in &n.streets {
        assert!(!centerline_dashes(s).is_empty(), "street has no centerline dashes");
    }
}

#[test]
fn grounds_are_deterministic() {
    let a = grounded(42);
    let b = grounded(42);
    assert_eq!(a.vertices, b.vertices, "grounds vertices differ for the same seed");
    assert_eq!(a.faces, b.faces, "grounds faces differ for the same seed");
    assert_eq!(a.materials, b.materials, "grounds materials differ for the same seed");
}
