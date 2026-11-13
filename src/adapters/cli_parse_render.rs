//! CLI parsing for the render command.
//!
//! `floorplan render --seed <u32> [--style <name>] [--view exterior|
//! interior|street] [--blocks WxH] [--out <path.png>]`

use crate::adapters::cli::Command;
use crate::adapters::cli_parse::{style_list, value};
use crate::adapters::cli_parse_neighborhood::parse_blocks;
use crate::domain::style::Style;
use crate::use_cases::neighborhood::DEFAULT_BLOCKS;
use crate::use_cases::viewpoint::View;

/// Parses the arguments after `render`.
pub fn parse_render(rest: Vec<&String>) -> Result<Command, String> {
    let mut seed = None;
    let mut style = None;
    let mut view = View::Exterior;
    let mut blocks = DEFAULT_BLOCKS;
    let mut out = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--seed" => {
                let v = value(&rest, &mut i, "--seed")?;
                seed = Some(
                    v.parse::<u32>()
                        .map_err(|_| format!("--seed must be a number, got {v}"))?,
                );
            }
            "--style" => {
                let v = value(&rest, &mut i, "--style")?;
                style = Some(Style::from_name(&v).ok_or_else(|| {
                    format!("unknown style: {v} (styles: {})", style_list())
                })?);
            }
            "--view" => {
                let v = value(&rest, &mut i, "--view")?;
                view = parse_view(&v)?;
            }
            "--blocks" => {
                let v = value(&rest, &mut i, "--blocks")?;
                blocks = parse_blocks(&v)?;
            }
            "--out" => out = Some(value(&rest, &mut i, "--out")?),
            other => return Err(format!("unknown flag: {other}")),
        }
        i += 1;
    }
    let Some(seed) = seed else {
        return Err("--seed <u32> is required".to_string());
    };
    let out = Some(out.unwrap_or_else(|| format!("render-{seed}-{}.png", view.id())));
    Ok(Command::Render { seed, style, view, blocks, out })
}

/// Parses the view name. An unknown name lists the three.
fn parse_view(v: &str) -> Result<View, String> {
    match v {
        "exterior" => Ok(View::Exterior),
        "interior" => Ok(View::Interior),
        "street" => Ok(View::Street),
        other => Err(format!("unknown view: {other} (views: exterior, interior, street)")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::cli_parse::parse_args;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn render_parses_with_defaults() {
        let cmd = parse_args(&args(&["floorplan", "render", "--seed", "42"])).unwrap();
        assert_eq!(
            cmd,
            Command::Render {
                seed: 42,
                style: None,
                view: View::Exterior,
                blocks: DEFAULT_BLOCKS,
                out: Some("render-42-exterior.png".to_string()),
            }
        );
    }

    #[test]
    fn render_parses_style_view_blocks_and_out() {
        let cmd = parse_args(&args(&[
            "floorplan", "render", "--seed", "7", "--style", "modern", "--view", "street",
            "--blocks", "3x2", "--out", "docs/screenshots/street.png",
        ]))
        .unwrap();
        assert_eq!(
            cmd,
            Command::Render {
                seed: 7,
                style: Some(Style::Modern),
                view: View::Street,
                blocks: (3, 2),
                out: Some("docs/screenshots/street.png".to_string()),
            }
        );
    }

    #[test]
    fn render_rejects_a_bad_view() {
        let err = parse_args(&args(&["floorplan", "render", "--seed", "1", "--view", "side"]))
            .unwrap_err();
        assert!(err.contains("unknown view: side"), "got: {err}");
    }

    #[test]
    fn render_requires_a_seed() {
        assert!(parse_args(&args(&["floorplan", "render"])).is_err());
    }

    #[test]
    fn the_view_parser_names_all_three_views() {
        assert_eq!(parse_view("exterior").unwrap(), View::Exterior);
        assert_eq!(parse_view("interior").unwrap(), View::Interior);
        assert_eq!(parse_view("street").unwrap(), View::Street);
        assert!(parse_view("attic").is_err());
    }
}
