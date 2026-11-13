//! Integration: the CLI end to end, through the built binary.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_floorplan"))
}

fn tmpdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("floorplan-test-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn list_styles_prints_all_six_styles() {
    let out = Command::new(bin()).arg("--list-styles").output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    for style in ["ranch", "colonial", "craftsman", "modern", "cottage", "townhouse"] {
        assert!(text.contains(style), "missing style {style}");
    }
}

#[test]
fn generate_writes_svg_and_json_files() {
    let dir = tmpdir("single");
    let svg = dir.join("house.svg");
    let json = dir.join("house.json");
    let status = Command::new(bin())
        .arg("generate")
        .arg("--seed")
        .arg("42")
        .arg("--style")
        .arg("ranch")
        .arg("--out")
        .arg(&svg)
        .arg("--json")
        .arg(&json)
        .status()
        .unwrap();
    assert!(status.success());
    let svg_text = fs::read_to_string(&svg).unwrap();
    let json_text = fs::read_to_string(&json).unwrap();
    assert!(svg_text.contains("<svg"));
    assert!(json_text.contains("\"seed\":42"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn generate_twice_gives_byte_identical_files() {
    let dir = tmpdir("determinism");
    let run = |name: &str| {
        let svg = dir.join(format!("{name}.svg"));
        let json = dir.join(format!("{name}.json"));
        let status = Command::new(bin())
            .arg("generate")
            .arg("--seed")
            .arg("7")
            .arg("--style")
            .arg("colonial")
            .arg("--out")
            .arg(&svg)
            .arg("--json")
            .arg(&json)
            .status()
            .unwrap();
        assert!(status.success());
        (fs::read(&svg).unwrap(), fs::read(&json).unwrap())
    };
    assert_eq!(run("a"), run("b"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn count_creates_that_many_houses() {
    let dir = tmpdir("count");
    let houses = dir.join("houses");
    let status = Command::new(bin())
        .arg("generate")
        .arg("--seed")
        .arg("100")
        .arg("--count")
        .arg("4")
        .arg("--dir")
        .arg(&houses)
        .status()
        .unwrap();
    assert!(status.success());
    let svgs = fs::read_dir(&houses).unwrap().filter(|e| {
        e.as_ref().unwrap().path().extension().is_some_and(|x| x == "svg")
    });
    assert_eq!(svgs.count(), 4);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn bad_input_exits_non_zero_with_an_error_on_stderr() {
    let out = Command::new(bin())
        .arg("generate")
        .arg("--seed")
        .arg("abc")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains("floorplan:"));
    let out = Command::new(bin()).arg("generate").output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn generate_obj_and_gltf_are_written_and_deterministic() {
    let dir = tmpdir("3d");
    let obj = dir.join("house.obj");
    let gltf = dir.join("house.gltf");
    let run = || {
        let status = Command::new(bin())
            .arg("generate")
            .arg("--seed")
            .arg("42")
            .arg("--style")
            .arg("modern")
            .arg("--out")
            .arg(dir.join("house.svg"))
            .arg("--obj")
            .arg(&obj)
            .arg("--gltf")
            .arg(&gltf)
            .status()
            .unwrap();
        assert!(status.success());
        (fs::read(&obj).unwrap(), fs::read(&gltf).unwrap(), fs::read(dir.join("house.mtl")).unwrap())
    };
    let (obj_a, gltf_a, mtl_a) = run();
    let (obj_b, gltf_b, mtl_b) = run();
    assert_eq!(obj_a, obj_b, "OBJ differs between runs");
    assert_eq!(gltf_a, gltf_b, "glTF differs between runs");
    assert_eq!(mtl_a, mtl_b, "MTL differs between runs");
    let obj_text = String::from_utf8(obj_a).unwrap();
    assert!(obj_text.contains("mtllib house.mtl"), "OBJ does not reference its MTL");
    assert!(obj_text.contains("usemtl wall"), "OBJ has no wall material");
    let mtl = String::from_utf8(mtl_a).unwrap();
    assert!(mtl.contains("newmtl wall"));
    let gltf_text = String::from_utf8(gltf_a).unwrap();
    assert!(gltf_text.contains("\"version\":\"2.0\""));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn obj_and_gltf_reject_count() {
    let out = Command::new(bin())
        .arg("generate")
        .arg("--seed")
        .arg("1")
        .arg("--count")
        .arg("2")
        .arg("--obj")
        .arg("house.obj")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains("floorplan:"));
}

#[test]
fn generated_svg_has_walls_doors_windows_and_labels() {
    let dir = tmpdir("elements");
    let svg_path = dir.join("house.svg");
    let status = Command::new(bin())
        .arg("generate")
        .arg("--seed")
        .arg("42")
        .arg("--style")
        .arg("ranch")
        .arg("--out")
        .arg(&svg_path)
        .status()
        .unwrap();
    assert!(status.success());
    let svg = fs::read_to_string(&svg_path).unwrap();
    assert!(svg.contains("<line class=\"wall\""), "no walls");
    assert!(svg.contains("<path class=\"swing\""), "no door swings");
    assert!(svg.contains("<line class=\"win\""), "no windows");
    assert!(svg.contains("living"), "no room labels");
    assert!(svg.contains("Ground floor"), "no floor label");
    fs::remove_dir_all(&dir).unwrap();
}
