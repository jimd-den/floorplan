//! Viewing scenes: a mesh plus the ground it stands on.

use floorplan::domain::mesh::Mesh;
use floorplan::domain::style::Style;
use floorplan::use_cases::extrude::extrude;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::neighborhood::generate_neighborhood;
use floorplan::use_cases::neighborhood_mesh::neighborhood_mesh;
use floorplan::use_cases::scene::{house_scene, neighborhood_scene};

fn house() -> floorplan::domain::model::House {
    generate_house(42, Style::Modern)
}

fn lowest(mesh: &Mesh) -> f32 {
    mesh.vertices.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min)
}

#[test]
fn a_house_scene_adds_one_ground_box_and_one_material() {
    let h = house();
    let bare = extrude(&h);
    let scene = house_scene(&h);

    assert_eq!(scene.vertices.len(), bare.vertices.len() + 8, "one box of vertices");
    assert_eq!(scene.faces.len(), bare.faces.len() + 12, "one box of faces");
    assert_eq!(scene.material_table.len(), bare.material_table.len() + 1, "a lawn material");
}

#[test]
fn the_house_ground_lies_below_the_floor_slab() {
    let scene = house_scene(&house());

    assert!(lowest(&scene) <= -0.02, "the lawn must sit below the floor");
    assert!(lowest(&scene) > -0.5, "the lawn must not be a pit");
}

#[test]
fn the_neighborhood_ground_covers_every_street_and_lot() {
    let n = generate_neighborhood(42, (2, 2));
    let bare = neighborhood_mesh(&n);
    let scene = neighborhood_scene(&n);

    assert_eq!(scene.faces.len(), bare.faces.len() + 12, "one ground box");
    let ground = scene.vertices[bare.vertices.len()..].iter();
    let (min_x, max_x) = ground.clone().fold((f32::MAX, f32::MIN), |(lo, hi), v| {
        (lo.min(v[0]), hi.max(v[0]))
    });
    let (min_z, max_z) = ground.fold((f32::MAX, f32::MIN), |(lo, hi), v| {
        (lo.min(v[2]), hi.max(v[2]))
    });
    for s in &n.streets {
        assert!(min_x <= s.rect.x as f32 && max_x >= s.rect.right() as f32, "ground short in x");
        assert!(min_z <= s.rect.y as f32 && max_z >= s.rect.bottom() as f32, "ground short in z");
    }
}
