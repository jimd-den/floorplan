//! JSON writer: the neighborhood as a hand-rolled JSON document.
//!
//! Fixed field order, two-decimal floats, no whitespace — the same
//! neighborhood always renders byte-identical JSON. Each house
//! carries its plan (seed-free), so the document is complete and
//! the variety hash can be taken over it. Protocol v3 (ADR 0010):
//! streets are spline control points, lots are polygons, houses
//! carry their centre, rotation, and walkway path.

use crate::adapters::json::render_plan_json;
use crate::domain::curve::Pt;
use crate::domain::neighborhood::Neighborhood;

/// Renders a whole neighborhood as a single-line JSON document.
pub fn render_neighborhood_json(n: &Neighborhood) -> String {
    render(n, true)
}

/// Renders the neighborhood without seeds — the canonical form
/// the variety hash is taken over, so distinct hashes mean
/// distinct neighborhoods rather than distinct seeds.
pub fn render_neighborhood_plan_json(n: &Neighborhood) -> String {
    render(n, false)
}

fn points(pts: &[Pt]) -> String {
    let mut s = String::from("[");
    for (i, p) in pts.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("[{:.2},{:.2}]", p.x, p.y));
    }
    s.push(']');
    s
}

fn render(n: &Neighborhood, with_seed: bool) -> String {
    let mut s = String::new();
    if with_seed {
        s.push_str(&format!("{{\"seed\":{},\"blocks\":[{},{}],", n.seed, n.blocks.0, n.blocks.1));
    } else {
        s.push_str(&format!("{{\"blocks\":[{},{}],", n.blocks.0, n.blocks.1));
    }
    s.push_str("\"streets\":[");
    for (i, st) in n.streets.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("{{\"control\":{},\"bulb\":{}", points(&st.control), st.bulb_at.is_some()));
        if let Some(c) = st.bulb_center {
            s.push_str(&format!(",\"bulb_center\":[{:.2},{:.2}]", c.x, c.y));
        }
        s.push('}');
    }
    s.push_str("],\"lots\":[");
    for (i, lot) in n.lots.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"points\":{},\"street\":{},\"center_s\":{:.2},\"side\":{}}}",
            points(&lot.outline),
            lot.street,
            lot.center_s,
            lot.side
        ));
    }
    s.push_str("],\"houses\":[");
    for (i, h) in n.houses.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"lot\":{},\"rotation\":{:.2},\"cx\":{:.2},\"cy\":{:.2},\"style\":\"{}\",\"walkway\":{}",
            h.lot, h.rotation, h.center.x, h.center.y, h.house.style.id(), points(&h.walkway)
        ));
        if with_seed {
            s.push_str(&format!(",\"seed\":{}", h.house.seed));
        }
        s.push_str(&format!(",\"plan\":{}}}", render_plan_json(&h.house)));
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::neighborhood::generate_neighborhood;

    #[test]
    fn json_carries_seed_and_blocks() {
        let json = render_neighborhood_json(&generate_neighborhood(42, (2, 2)));
        assert!(json.contains("\"seed\":42"));
        assert!(json.contains("\"blocks\":[2,2]"));
    }

    #[test]
    fn json_has_streets_lots_and_houses() {
        let json = render_neighborhood_json(&generate_neighborhood(42, (2, 2)));
        assert!(json.contains("\"streets\":["), "no streets");
        assert!(json.contains("\"lots\":["), "no lots");
        assert!(json.contains("\"houses\":["), "no houses");
        assert!(json.contains("\"rotation\":"), "no rotation");
        assert!(json.contains("\"walkway\":"), "no walkway");
        assert!(json.contains("\"control\":"), "no street control points");
    }

    #[test]
    fn the_seed_free_form_omits_both_seeds() {
        let json = render_neighborhood_plan_json(&generate_neighborhood(42, (2, 2)));
        assert!(!json.contains("\"seed\":"), "seed leaked into the plan form");
        assert!(json.contains("\"style\":"), "style missing");
    }

    #[test]
    fn json_is_deterministic() {
        let n = generate_neighborhood(42, (2, 2));
        assert_eq!(render_neighborhood_json(&n), render_neighborhood_json(&n));
    }
}
