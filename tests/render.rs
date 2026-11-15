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

fn luma_variance(pixels: &[[u8; 3]]) -> f64 {
    let n = pixels.len() as f64;
    let lumas: Vec<f64> = pixels
        .iter()
        .map(|p| 0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64)
        .collect();
    let mean = lumas.iter().sum::<f64>() / n;
    lumas.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n
}

#[test]
fn rendered_exterior_and_interior_have_pixel_variance_and_visible_shadow() {
    use floorplan::domain::style::Style;
    use floorplan::use_cases::extrude::extrude;
    use floorplan::use_cases::generate::generate_house;
    use floorplan::use_cases::viewpoint::{View, house_pose};

    let house = generate_house(42, Style::Modern);
    let mesh = extrude(&house);
    let vp = Viewport { width: 160, height: 90 };

    for view in [View::Exterior, View::Interior] {
        let pose = house_pose(&house, view);
        let cam = Camera::looking_at(pose.eye, pose.target);
        let img = render(&mesh, &cam, vp);
        let var = luma_variance(&img.pixels);
        assert!(var > 150.0, "{view:?} render is flat-shaded (variance {var:.1} <= 150.0)");
    }

    let start = std::time::Instant::now();
    for s in 0..20u32 {
        let _ = generate_house(s, Style::all()[(s % 6) as usize]);
    }
    let per_house = start.elapsed() / 20;
    assert!(per_house.as_millis() <= 25, "house generation took {per_house:?}");

    // Direct test that a single siding wall face has non-zero pixel variance
    // (visible course shadow lines, never flat-shaded).
    let mut wall_mesh = Mesh::new(vec![Material { name: "siding", color: [0.72, 0.68, 0.58] }]);
    add_triangle(&mut wall_mesh, -2.0, 0, false);
    let wall_img = render(&wall_mesh, &camera(), Viewport { width: 64, height: 48 });
    let patch: Vec<[u8; 3]> = (18..30)
        .flat_map(|y| (24..40).map(move |x| (x, y)))
        .map(|(x, y)| wall_img.pixels[y * 64 + x])
        .collect();
    assert!(
        luma_variance(&patch) > 4.0,
        "siding wall patch is flat-shaded: variance {:.2}",
        luma_variance(&patch)
    );
}
