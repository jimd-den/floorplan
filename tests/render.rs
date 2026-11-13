//! The software rasterizer: projection, depth, and no backface culling.

use floorplan::adapters::render::{render, Camera, Viewport};
use floorplan::domain::mesh::{Material, Mesh};

fn viewport() -> Viewport {
    Viewport { width: 64, height: 48 }
}

fn camera() -> Camera {
    Camera::looking_at([0.0, 1.5, 0.0], [0.0, 1.5, -10.0])
}

fn table() -> Vec<Material> {
    vec![
        Material { name: "red", color: [1.0, 0.0, 0.0] },
        Material { name: "green", color: [0.0, 1.0, 0.0] },
    ]
}

/// One upright triangle at depth `z`, in `material`.
fn add_triangle(mesh: &mut Mesh, z: f32, material: u32, reversed: bool) {
    let base = mesh.vertices.len() as u32;
    mesh.vertices.extend([[-2.0, 0.0, z], [2.0, 0.0, z], [0.0, 3.0, z]]);
    let face = if reversed {
        [base, base + 2, base + 1]
    } else {
        [base, base + 1, base + 2]
    };
    mesh.faces.push(face);
    mesh.materials.push(material);
}

fn centre(image: &floorplan::adapters::render::Image) -> [u8; 3] {
    image.pixels[(image.height / 2) * image.width + image.width / 2]
}

#[test]
fn a_point_straight_ahead_projects_to_the_centre() {
    let (x, y, depth) = camera().project([0.0, 1.5, -5.0], viewport()).unwrap();

    assert!((x - 32.0).abs() < 0.01, "x was {x}");
    assert!((y - 24.0).abs() < 0.01, "y was {y}");
    assert!((depth - 5.0).abs() < 0.01, "depth was {depth}");
}

#[test]
fn a_point_behind_the_camera_does_not_project() {
    assert!(camera().project([0.0, 1.5, 5.0], viewport()).is_none());
}

#[test]
fn an_empty_mesh_renders_as_sky_only() {
    let mesh = Mesh::new(table());
    let image = render(&mesh, &camera(), viewport());

    assert!(image.pixels.iter().all(|p| p[2] > p[0]), "every pixel must be sky");
    assert_eq!(image.pixels.len(), 64 * 48);
    assert_ne!(image.pixels[0], image.pixels[image.pixels.len() - 1], "sky is a gradient");
}

#[test]
fn the_nearest_face_wins_the_depth_test() {
    let mut mesh = Mesh::new(table());
    add_triangle(&mut mesh, -6.0, 1, false);
    add_triangle(&mut mesh, -2.0, 0, false);

    let image = render(&mesh, &camera(), viewport());

    assert_eq!(centre(&image), [255, 0, 0], "the near red face must win");
}

#[test]
fn a_face_turned_away_is_still_drawn() {
    let mut mesh = Mesh::new(table());
    add_triangle(&mut mesh, -2.0, 0, true);

    let image = render(&mesh, &camera(), viewport());

    assert_ne!(centre(&image), image.pixels[0], "no backface culling");
}

#[test]
fn a_floor_passing_under_the_camera_is_still_drawn() {
    let mut mesh = Mesh::new(table());
    let base = mesh.vertices.len() as u32;
    mesh.vertices.extend([
        [-5.0, 0.0, 2.0],
        [5.0, 0.0, 2.0],
        [5.0, 0.0, -6.0],
        [-5.0, 0.0, -6.0],
    ]);
    mesh.faces.push([base, base + 1, base + 2]);
    mesh.faces.push([base, base + 2, base + 3]);
    mesh.materials.push(0);
    mesh.materials.push(0);

    let image = render(&mesh, &camera(), viewport());
    let bottom = image.pixels[(image.height - 2) * image.width + image.width / 2];

    assert_ne!(bottom, image.pixels[0], "a floor under the camera must still render");
}

#[test]
fn rendering_the_same_mesh_twice_gives_the_same_pixels() {
    let mut mesh = Mesh::new(table());
    add_triangle(&mut mesh, -2.0, 0, false);
    add_triangle(&mut mesh, -6.0, 1, false);

    let a = render(&mesh, &camera(), viewport());
    let b = render(&mesh, &camera(), viewport());

    assert_eq!(a.pixels, b.pixels);
}
