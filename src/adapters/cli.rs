//! CLI: hand-rolled argument parsing and the generate command.
//!
//! `floorplan generate --seed <u32> [--style <name>] [--out <path.svg>]`
//! `[--json [<path.json>]] [--count N] [--dir <dir>]`
//! `[--obj <path.obj>] [--gltf <path.gltf>]`
//! `floorplan neighborhood --seed <u32> [--blocks WxH] [--out <path.svg>]`
//! `[--json [<path.json>]]`
//! `floorplan --list-styles`

use crate::adapters::cli_parse::parse_args;
use crate::domain::style::Style;

/// What the user asked for.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    ListStyles,
    Generate {
        seed: u32,
        style: Option<Style>,
        out: Option<String>,
        /// None: no JSON. Some(None): bare --json, default path.
        /// Some(Some(path)): JSON to that path.
        json: Option<Option<String>>,
        count: Option<u32>,
        dir: Option<String>,
        /// OBJ path; the .mtl is written next to it, same stem.
        obj: Option<String>,
        /// glTF path (single .gltf, buffer embedded as base64).
        gltf: Option<String>,
    },
    Neighborhood {
        seed: u32,
        /// (columns, rows) of blocks; default 2x2.
        blocks: (u32, u32),
        out: Option<String>,
        /// None: no JSON. Some(None): bare --json, default path.
        /// Some(Some(path)): JSON to that path.
        json: Option<Option<String>>,
    },
}

/// Runs the CLI, returning the process exit code.
pub fn run(args: &[String]) -> i32 {
    match parse_args(args) {
        Err(e) => {
            eprintln!("floorplan: {e}");
            2
        }
        Ok(Command::ListStyles) => {
            for s in Style::all() {
                println!("{} — {}", s.id(), s.name());
            }
            0
        }
        Ok(Command::Generate { seed, style, out, json, count, dir, obj, gltf }) => {
            let mesh = crate::adapters::cli_generate::MeshOutputs { obj, gltf };
            match crate::adapters::cli_generate::generate(seed, style, out, json, count, dir, mesh) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("floorplan: {e}");
                    1
                }
            }
        }
        Ok(Command::Neighborhood { seed, blocks, out, json }) => {
            match crate::adapters::cli_neighborhood::neighborhood(seed, blocks, out, json) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("floorplan: {e}");
                    1
                }
            }
        }
    }
}

