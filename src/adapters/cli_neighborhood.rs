//! The neighborhood command: build a neighborhood and write its
//! SVG and optionally its JSON.

use crate::adapters::neighborhood_json::render_neighborhood_json;
use crate::adapters::neighborhood_svg::render_neighborhood_svg;
use crate::use_cases::neighborhood::generate_neighborhood;

pub(crate) fn neighborhood(
    seed: u32,
    blocks: (u32, u32),
    out: Option<String>,
    json: Option<Option<String>>,
) -> Result<(), String> {
    let n = generate_neighborhood(seed, blocks);
    let svg_path = out.unwrap_or_else(|| format!("neighborhood-{seed}.svg"));
    write(&svg_path, &render_neighborhood_svg(&n))?;
    match json {
        Some(Some(path)) => write(&path, &render_neighborhood_json(&n)),
        Some(None) => {
            let path = format!("neighborhood-{seed}.json");
            write(&path, &render_neighborhood_json(&n))
        }
        None => Ok(()),
    }
}

fn write(path: &str, contents: &str) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|e| format!("cannot write {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmpdir() -> PathBuf {
        let d = std::env::temp_dir().join(format!("fp-nb-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn neighborhood_writes_svg_and_json() {
        let d = tmpdir();
        let svg = d.join("n.svg");
        let json = d.join("n.json");
        neighborhood(42, (2, 2), Some(svg.to_str().unwrap().to_string()), Some(Some(json.to_str().unwrap().to_string()))).unwrap();
        assert!(std::fs::read_to_string(&svg).unwrap().contains("class=\"street\""));
        assert!(std::fs::read_to_string(&json).unwrap().contains("\"seed\":42"));
    }

    #[test]
    fn neighborhood_uses_default_paths_when_none_given() {
        let d = tmpdir();
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&d).unwrap();
        neighborhood(43, (1, 1), None, Some(None)).unwrap();
        assert!(d.join("neighborhood-43.svg").exists());
        assert!(d.join("neighborhood-43.json").exists());
        std::env::set_current_dir(cwd).unwrap();
    }
}
