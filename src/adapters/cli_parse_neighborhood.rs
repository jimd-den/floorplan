//! CLI parsing for the neighborhood command.

use crate::adapters::cli::Command;
use crate::adapters::cli_parse::value;
use crate::use_cases::neighborhood::DEFAULT_BLOCKS;

/// Parses `floorplan neighborhood --seed <u32> [--blocks WxH]
/// [--out <path.svg>] [--json [<path.json>]]`.
pub fn parse_neighborhood(rest: Vec<&String>) -> Result<Command, String> {
    let mut seed = None;
    let mut blocks = DEFAULT_BLOCKS;
    let mut out = None;
    let mut json: Option<Option<String>> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--seed" => {
                let v = value(&rest, &mut i, "--seed")?;
                seed = Some(v.parse::<u32>().map_err(|_| format!("--seed must be a number, got {v}"))?);
            }
            "--blocks" => {
                let v = value(&rest, &mut i, "--blocks")?;
                blocks = parse_blocks(&v)?;
            }
            "--out" => out = Some(value(&rest, &mut i, "--out")?),
            "--json" => {
                if i + 1 < rest.len() && !rest[i + 1].starts_with("--") {
                    i += 1;
                    json = Some(Some(rest[i].clone()));
                } else {
                    json = Some(None);
                }
            }
            other => return Err(format!("unknown flag: {other}")),
        }
        i += 1;
    }
    let Some(seed) = seed else {
        return Err("--seed <u32> is required".to_string());
    };
    Ok(Command::Neighborhood { seed, blocks, out, json })
}

/// Parses `WxH` into (columns, rows), each 1..=10.
pub(crate) fn parse_blocks(v: &str) -> Result<(u32, u32), String> {
    let (a, b) = v.split_once('x').ok_or_else(|| format!("--blocks must be WxH, got {v}"))?;
    let w: u32 = a.parse().map_err(|_| format!("--blocks must be WxH, got {v}"))?;
    let h: u32 = b.parse().map_err(|_| format!("--blocks must be WxH, got {v}"))?;
    if w == 0 || h == 0 || w > 10 || h > 10 {
        return Err(format!("--blocks must be 1x1..10x10, got {v}"));
    }
    Ok((w, h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::cli_parse::parse_args;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn neighborhood_parses_with_defaults() {
        let cmd = parse_args(&args(&["floorplan", "neighborhood", "--seed", "42"])).unwrap();
        assert_eq!(
            cmd,
            Command::Neighborhood { seed: 42, blocks: (2, 2), out: None, json: None }
        );
    }

    #[test]
    fn neighborhood_parses_blocks_out_and_json() {
        let cmd = parse_args(&args(&[
            "floorplan", "neighborhood", "--seed", "7", "--blocks", "3x2", "--out", "n.svg",
            "--json", "n.json",
        ]))
        .unwrap();
        assert_eq!(
            cmd,
            Command::Neighborhood {
                seed: 7,
                blocks: (3, 2),
                out: Some("n.svg".to_string()),
                json: Some(Some("n.json".to_string())),
            }
        );
    }

    #[test]
    fn neighborhood_rejects_bad_blocks() {
        assert!(parse_args(&args(&["floorplan", "neighborhood", "--seed", "1", "--blocks", "3"])).is_err());
        assert!(parse_args(&args(&["floorplan", "neighborhood", "--seed", "1", "--blocks", "0x2"])).is_err());
        assert!(parse_args(&args(&["floorplan", "neighborhood", "--seed", "1", "--blocks", "11x2"])).is_err());
    }

    #[test]
    fn neighborhood_requires_a_seed() {
        assert!(parse_args(&args(&["floorplan", "neighborhood"])).is_err());
    }
}
