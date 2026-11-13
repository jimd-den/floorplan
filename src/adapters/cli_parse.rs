//! CLI parsing: argv into a Command.

use crate::adapters::cli::Command;
use crate::domain::style::Style;

/// Parses argv (including the program name at [0]).
pub fn parse_args(args: &[String]) -> Result<Command, String> {
    let mut it = args.iter().skip(1);
    let Some(first) = it.next() else {
        return Err(
            "usage: floorplan generate --seed <u32> | floorplan neighborhood --seed <u32> | floorplan --list-styles".to_string(),
        );
    };
    if first == "--list-styles" {
        if it.next().is_some() {
            return Err("--list-styles takes no other arguments".to_string());
        }
        return Ok(Command::ListStyles);
    }
    if first == "neighborhood" {
        return crate::adapters::cli_parse_neighborhood::parse_neighborhood(it.collect());
    }
    if first != "generate" {
        return Err(format!("unknown command: {first} (expected generate, neighborhood, or --list-styles)"));
    }
    let mut seed: Option<u32> = None;
    let mut style = None;
    let mut out = None;
    let mut json: Option<Option<String>> = None;
    let mut count = None;
    let mut dir = None;
    let mut obj = None;
    let mut gltf = None;
    let rest: Vec<&String> = it.collect();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--seed" => {
                let v = value(&rest, &mut i, "--seed")?;
                seed = Some(v.parse::<u32>().map_err(|_| format!("--seed must be a number, got {v}"))?);
            }
            "--style" => {
                let v = value(&rest, &mut i, "--style")?;
                style = Some(Style::from_name(&v).ok_or_else(|| {
                    format!("unknown style: {v} (styles: {})", style_list())
                })?);
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
            "--count" => {
                let v = value(&rest, &mut i, "--count")?;
                count =
                    Some(v.parse::<u32>().map_err(|_| format!("--count must be a number, got {v}"))?);
            }
            "--dir" => dir = Some(value(&rest, &mut i, "--dir")?),
            "--obj" => obj = Some(value(&rest, &mut i, "--obj")?),
            "--gltf" => gltf = Some(value(&rest, &mut i, "--gltf")?),
            other => return Err(format!("unknown flag: {other}")),
        }
        i += 1;
    }
    let Some(seed) = seed else {
        return Err("--seed <u32> is required".to_string());
    };
    if count.is_some() && out.is_some() {
        return Err("--out cannot be combined with --count".to_string());
    }
    if count.is_some() && matches!(json, Some(Some(_))) {
        return Err("--json <path> cannot be combined with --count; use bare --json".to_string());
    }
    if count.is_some() && (obj.is_some() || gltf.is_some()) {
        return Err("--obj/--gltf cannot be combined with --count".to_string());
    }
    Ok(Command::Generate { seed, style, out, json, count, dir, obj, gltf })
}

/// The value after a flag, or an error when the flag is last.
pub(crate) fn value(rest: &[&String], i: &mut usize, flag: &str) -> Result<String, String> {
    if *i + 1 >= rest.len() {
        return Err(format!("{flag} needs a value"));
    }
    *i += 1;
    Ok(rest[*i].clone())
}

fn style_list() -> String {
    Style::all().iter().map(|s| s.id()).collect::<Vec<_>>().join(", ")
}


#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    /// The generate fields as a tuple, for compact assertions.
    type Fields = (u32, Option<Style>, Option<Option<String>>, Option<u32>, Option<String>, Option<String>, Option<String>);

    fn fields(cmd: Command) -> Fields {
        match cmd {
            Command::Generate { seed, style, json, count, dir, obj, gltf, .. } => {
                (seed, style, json, count, dir, obj, gltf)
            }
            _ => panic!("expected a generate command"),
        }
    }

    #[test]
    fn list_styles_parses() {
        assert_eq!(parse_args(&args(&["floorplan", "--list-styles"])), Ok(Command::ListStyles));
    }

    #[test]
    fn generate_requires_a_seed() {
        assert!(parse_args(&args(&["floorplan", "generate"])).is_err());
    }

    #[test]
    fn generate_parses_seed_and_style() {
        let cmd = parse_args(&args(&["floorplan", "generate", "--seed", "42", "--style", "ranch"])).unwrap();
        assert_eq!(fields(cmd), (42, Some(Style::Ranch), None, None, None, None, None));
    }

    #[test]
    fn obj_and_gltf_parse() {
        let cmd = parse_args(&args(&[
            "floorplan", "generate", "--seed", "1", "--obj", "h.obj", "--gltf", "h.gltf",
        ]))
        .unwrap();
        assert_eq!(
            fields(cmd),
            (1, None, None, None, None, Some("h.obj".to_string()), Some("h.gltf".to_string()))
        );
    }

    #[test]
    fn obj_and_gltf_reject_count() {
        let err = parse_args(&args(&[
            "floorplan", "generate", "--seed", "1", "--count", "2", "--obj", "h.obj",
        ]))
        .unwrap_err();
        assert!(err.contains("--count"));
    }

    #[test]
    fn unknown_style_is_an_error_naming_the_styles() {
        let err = parse_args(&args(&["floorplan", "generate", "--seed", "1", "--style", "modernist"]))
            .unwrap_err();
        assert!(err.contains("modernist"));
        assert!(err.contains("ranch"));
    }

    #[test]
    fn bad_input_is_an_error() {
        assert!(parse_args(&args(&["floorplan", "generate", "--seed", "abc"])).is_err());
        assert!(parse_args(&args(&["floorplan", "generate", "--seed", "1", "--frobnicate"])).is_err());
    }

    #[test]
    fn json_parses_bare_and_with_a_path() {
        let bare = parse_args(&args(&["floorplan", "generate", "--seed", "1", "--json"])).unwrap();
        assert_eq!(fields(bare), (1, None, Some(None), None, None, None, None));
        let path = parse_args(&args(&["floorplan", "generate", "--seed", "1", "--json", "out.json"])).unwrap();
        assert_eq!(fields(path), (1, None, Some(Some("out.json".to_string())), None, None, None, None));
    }

    #[test]
    fn count_and_dir_parse() {
        let cmd = parse_args(&args(&[
            "floorplan", "generate", "--seed", "1", "--count", "5", "--dir", "out",
        ]))
        .unwrap();
        assert_eq!(fields(cmd), (1, None, None, Some(5), Some("out".to_string()), None, None));
    }
}
