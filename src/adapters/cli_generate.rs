//! The generate command: build houses and write SVG/JSON/OBJ/glTF files.

use crate::adapters::gltf::render_gltf;
use crate::adapters::json::render_json;
use crate::adapters::obj::{render_mtl, render_obj};
use crate::adapters::svg::render_svg;
use crate::domain::style::Style;
use crate::use_cases::extrude::extrude;
use crate::use_cases::generate::generate_house;

/// The 3D output paths: OBJ (+ its .mtl) and/or glTF.
pub(crate) struct MeshOutputs {
    pub obj: Option<String>,
    pub gltf: Option<String>,
}

pub(crate) fn generate(
    seed: u32,
    style: Option<Style>,
    out: Option<String>,
    json: Option<Option<String>>,
    count: Option<u32>,
    dir: Option<String>,
    mesh: MeshOutputs,
) -> Result<(), String> {
    match count {
        Some(n) => {
            let dir = dir.unwrap_or_else(|| "houses".to_string());
            std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {dir}: {e}"))?;
            for k in 0..n {
                let s = seed.wrapping_add(k);
                let house = generate_house(s, style.unwrap_or_else(|| style_for_seed(s)));
                let svg_path = format!("{dir}/house-{s}.svg");
                write(&svg_path, &render_svg(&house))?;
                if json.is_some() {
                    let json_path = format!("{dir}/house-{s}.json");
                    write(&json_path, &render_json(&house))?;
                }
            }
            Ok(())
        }
        None => {
            let house = generate_house(seed, style.unwrap_or_else(|| style_for_seed(seed)));
            let svg_path = out.unwrap_or_else(|| format!("house-{seed}.svg"));
            write(&svg_path, &render_svg(&house))?;
            match json {
                Some(Some(path)) => write(&path, &render_json(&house)),
                Some(None) => {
                    let path = format!("house-{seed}.json");
                    write(&path, &render_json(&house))
                }
                None => Ok(()),
            }?;
            write_3d(&house, mesh.obj.as_deref(), mesh.gltf.as_deref())
        }
    }
}

/// Writes the OBJ (+ its .mtl, same stem) and/or glTF for a house.
fn write_3d(house: &crate::domain::model::House, obj: Option<&str>, gltf: Option<&str>) -> Result<(), String> {
    if obj.is_none() && gltf.is_none() {
        return Ok(());
    }
    let mesh = extrude(house);
    if let Some(path) = obj {
        let mtl_path = format!("{}.mtl", stem(path));
        let mtl_name = mtl_path.rsplit('/').next().unwrap_or(&mtl_path);
        write(path, &render_obj(&mesh, mtl_name))?;
        write(&mtl_path, &render_mtl(&mesh))?;
    }
    if let Some(path) = gltf {
        write(path, &render_gltf(&mesh))?;
    }
    Ok(())
}

/// The file stem of a path: `dir/house.obj` -> `dir/house`.
fn stem(path: &str) -> &str {
    path.strip_suffix(".obj").unwrap_or(path)
}

fn write(path: &str, contents: &str) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|e| format!("cannot write {path}: {e}"))
}

/// No style given: the seed picks one, deterministically.
fn style_for_seed(seed: u32) -> Style {
    Style::all()[(seed % Style::all().len() as u32) as usize]
}

