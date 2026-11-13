//! Chunk JSON: the chunk coordinates and state, plus the neighborhood
//! text the CLI already emits. The neighborhood document is reused
//! verbatim, so chunk (0,0) carries exactly the fixed neighborhood's
//! JSON and the shell parses one shape for every chunk.

use crate::adapters::neighborhood_json::render_neighborhood_json;
use crate::domain::chunk::Chunk;

/// Renders a chunk as one JSON document. A chunk without content
/// (still requested or generating) has nothing to render.
pub fn render_chunk_json(chunk: &Chunk) -> Option<String> {
    let neighborhood = chunk.neighborhood.as_ref()?;
    Some(format!(
        "{{\"chunk\":[{},{}],\"state\":\"{}\",\"neighborhood\":{}}}",
        chunk.cx,
        chunk.cz,
        chunk.state.id(),
        render_neighborhood_json(neighborhood)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::chunk::build_chunk;

    #[test]
    fn chunk_json_carries_coordinates_and_the_neighborhood() {
        let chunk = build_chunk(42, 1, -2);
        let json = render_chunk_json(&chunk).expect("a ready chunk has content");
        assert!(json.contains("\"chunk\":[1,-2]"));
        assert!(json.contains("\"state\":\"ready\""));
        assert!(json.contains("\"neighborhood\":{\"seed\":"));
    }

    #[test]
    fn chunk_json_is_deterministic() {
        let a = render_chunk_json(&build_chunk(42, 3, 4));
        let b = render_chunk_json(&build_chunk(42, 3, 4));
        assert_eq!(a, b);
    }
}
