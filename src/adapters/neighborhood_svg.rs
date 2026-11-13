//! SVG renderer: one neighborhood as one readable SVG document.
//!
//! Units are metres (viewBox in metres). Streets are dark bands,
//! lots are pale outlines, each house's footprint is drawn in
//! place (footprint rectangles + garage, rotated to face its
//! street) and labelled with style and seed.

use crate::domain::neighborhood::{Neighborhood, house_extent, place_rect};

/// Renders a whole neighborhood as an SVG document.
pub fn render_neighborhood_svg(n: &Neighborhood) -> String {
    let (w, h) = total_extent(n);
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"{:.2} {:.2} {:.2} {:.2}\">\n",
        (w * 5.0) as u32,
        (h * 5.0) as u32,
        -1.0,
        -1.0,
        w + 2.0,
        h + 2.0
    ));
    s.push_str("<style>text{font-family:sans-serif;font-size:2px;fill:#222} .lot{fill:#f4f1e8;stroke:#999;stroke-width:0.3} .street{fill:#666} .house{fill:#b8c4d4;stroke:#334;stroke-width:0.3} .garage{fill:#9aa4b4;stroke:#334;stroke-width:0.3}</style>\n");
    for street in &n.streets {
        let r = street.rect;
        s.push_str(&format!(
            "<rect class=\"street\" x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\"/>\n",
            r.x, r.y, r.w, r.d
        ));
    }
    for lot in &n.lots {
        let r = lot.rect;
        s.push_str(&format!(
            "<rect class=\"lot\" x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\"/>\n",
            r.x, r.y, r.w, r.d
        ));
    }
    for h in &n.houses {
        let extent = house_extent(&h.house);
        for r in h.house.footprint.rects.iter() {
            let q = place_rect(*r, extent, h.rect, h.rotation);
            s.push_str(&format!(
                "<rect class=\"house\" x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\"/>\n",
                q.x, q.y, q.w, q.d
            ));
        }
        if let Some(g) = h.house.footprint.garage {
            let q = place_rect(g, extent, h.rect, h.rotation);
            s.push_str(&format!(
                "<rect class=\"garage\" x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\"/>\n",
                q.x, q.y, q.w, q.d
            ));
        }
        let (cx, cy) = h.rect.center();
        s.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\">{} #{}</text>\n",
            cx,
            cy,
            h.house.style.name(),
            h.house.seed
        ));
    }
    s.push_str("</svg>\n");
    s
}

/// The neighborhood's total extent, from its streets.
fn total_extent(n: &Neighborhood) -> (f64, f64) {
    let mut w = 0.0f64;
    let mut h = 0.0f64;
    for st in &n.streets {
        w = w.max(st.rect.right());
        h = h.max(st.rect.bottom());
    }
    (w, h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::neighborhood::generate_neighborhood;

    #[test]
    fn the_svg_has_streets_lots_and_houses() {
        let svg = render_neighborhood_svg(&generate_neighborhood(42, (2, 2)));
        assert!(svg.contains("class=\"street\""), "no streets");
        assert!(svg.contains("class=\"lot\""), "no lots");
        assert!(svg.contains("class=\"house\""), "no houses");
    }

    #[test]
    fn the_svg_labels_houses_with_style_and_seed() {
        let n = generate_neighborhood(42, (2, 2));
        let svg = render_neighborhood_svg(&n);
        let first = &n.houses[0];
        assert!(svg.contains(first.house.style.name()), "style label missing");
        assert!(svg.contains(&format!("#{}", first.house.seed)), "seed label missing");
    }

    #[test]
    fn the_svg_is_deterministic() {
        let n = generate_neighborhood(42, (2, 2));
        assert_eq!(render_neighborhood_svg(&n), render_neighborhood_svg(&n));
    }
}
