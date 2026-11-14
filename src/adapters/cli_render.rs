//! The render command: build a mesh, frame a pose, write a PNG.
//!
//! The rasterizer is an adapter, not the WASM renderer: it exists so a
//! mesh can be inspected without a browser (ADR 0006). Every choice of
//! where to stand is a use case (`use_cases::viewpoint`); this module
//! only joins the two and writes the file.

use crate::adapters::png::encode_rgb;
use crate::adapters::render::{render as rasterize, Camera, Viewport};
use crate::domain::layout::Density;
use crate::domain::style::Style;
use crate::use_cases::generate::generate_house;
use crate::use_cases::neighborhood::generate_neighborhood_density;
use crate::use_cases::scene::{house_scene, neighborhood_scene};
use crate::use_cases::viewpoint::{house_pose, street_pose, View};

/// Screenshot size. 16:9, and small enough to render in a few seconds.
pub const WIDTH: usize = 1280;
pub const HEIGHT: usize = 720;

/// Renders one first-person view of a house or a street to a PNG.
pub(crate) fn render(
    seed: u32,
    style: Option<Style>,
    view: View,
    blocks: (u32, u32),
    out: Option<String>,
) -> Result<(), String> {
    let (mesh, pose) = match view {
        View::Street => {
            let n = generate_neighborhood_density(seed, blocks, Density::High);
            let pose = street_pose(&n);
            (neighborhood_scene(&n), pose)
        }
        _ => {
            let house = generate_house(seed, style.unwrap_or_else(|| style_for_seed(seed)));
            let pose = house_pose(&house, view);
            (house_scene(&house), pose)
        }
    };
    let camera = Camera::looking_at(pose.eye, pose.target);
    let image = rasterize(&mesh, &camera, Viewport { width: WIDTH, height: HEIGHT });
    let png = encode_rgb(WIDTH as u32, HEIGHT as u32, &image.pixels);
    let path = out.unwrap_or_else(|| format!("render-{seed}-{}.png", view.id()));
    std::fs::write(&path, png).map_err(|e| format!("cannot write {path}: {e}"))
}

/// The same style choice `generate` makes when `--style` is absent.
fn style_for_seed(seed: u32) -> Style {
    Style::all()[(seed % Style::all().len() as u32) as usize]
}
