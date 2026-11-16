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
    s.push_str("],\"lamps\":[");
    for (i, l) in n.lamps.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let light = l.light(i as u32);
        s.push_str(&format!(
            "{{\"x\":{:.2},\"y\":{:.2},\"rotation\":{:.2},\"light\":{{\"id\":{},\"kind\":\"{}\",\"radius\":{:.2},\"state\":\"{}\"}}}}",
            l.position.x,
            l.position.y,
            l.rotation,
            light.id,
            light.kind.id(),
            light.radius,
            light.state.id()
        ));
    }
    // Sign text comes from a fixed alphabet (street names, STOP, a
    // speed limit), so the hand-rolled writer needs no escaping.
    s.push_str("],\"signs\":[");
    for (i, g) in n.signs.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"x\":{:.2},\"y\":{:.2},\"rotation\":{:.2},\"kind\":\"{}\",\"text\":\"{}\"}}",
            g.position.x,
            g.position.y,
            g.rotation,
            g.kind.id(),
            g.text
        ));
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
    fn json_carries_lamps_with_lights_and_signs_with_text() {
        let json = render_neighborhood_json(&generate_neighborhood(42, (2, 2)));
        assert!(json.contains("\"lamps\":["), "no lamps");
        assert!(json.contains("\"light\":{\"id\":0,\"kind\":\"lamp\""), "no lamp light");
        assert!(json.contains("\"signs\":["), "no signs");
        assert!(json.contains("\"kind\":\"stop\",\"text\":\"STOP\""), "no stop sign");
        assert!(json.contains("\"kind\":\"street_name\""), "no street-name sign");
        assert!(json.contains("\"kind\":\"speed_limit\""), "no speed-limit sign");
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
