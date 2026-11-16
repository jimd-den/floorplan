//! SVG renderer: one neighborhood as one readable SVG document.
//!
//! Units are metres (viewBox in metres). Streets are dark ribbons,
//! lots are pale outlines, each house's footprint is drawn in
//! place (footprint rectangles + garage, rotated to face its
//! street) and labelled with style and seed.

use crate::domain::curve::Pt;
use crate::domain::neighborhood::Neighborhood;
use crate::domain::placement::{house_corners, house_extent, placed_point};

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
        let left = street.samples.offset(4.0);
        let right = street.samples.offset(-4.0);
        let mut pts: Vec<Pt> = left;
        pts.extend(right.iter().rev().copied());
        s.push_str(&format!("<polygon class=\"street\" points=\"{}\"/>\n", points_attr(&pts)));
    }
    for lot in &n.lots {
        s.push_str(&format!("<polygon class=\"lot\" points=\"{}\"/>\n", points_attr(&lot.outline)));
    }
    for h in &n.houses {
        let extent = house_extent(&h.house);
        for r in h.house.footprint.rects.iter() {
            let q = quad_for(*r, extent, h);
            s.push_str(&format!("<polygon class=\"house\" points=\"{}\"/>\n", points_attr(&q)));
        }
        if let Some(g) = h.house.footprint.garage {
            let q = quad_for(g, extent, h);
            s.push_str(&format!("<polygon class=\"garage\" points=\"{}\"/>\n", points_attr(&q)));
        }
        s.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\">{} #{}</text>\n",
            h.center.x,
            h.center.y,
            h.house.style.name(),
            h.house.seed
        ));
    }
    s.push_str("</svg>\n");
    s
}

/// The four placed corners of a house-local rectangle.
fn quad_for(r: crate::domain::geometry::Rect, extent: crate::domain::geometry::Rect, h: &crate::domain::neighborhood::PlacedHouse) -> [Pt; 4] {
    [
        placed_point(h.center, h.rotation, extent, Pt::new(r.x, r.y)),
        placed_point(h.center, h.rotation, extent, Pt::new(r.right(), r.y)),
        placed_point(h.center, h.rotation, extent, Pt::new(r.right(), r.bottom())),
        placed_point(h.center, h.rotation, extent, Pt::new(r.x, r.bottom())),
    ]
}

fn points_attr(pts: &[Pt]) -> String {
    pts.iter().map(|p| format!("{:.2},{:.2}", p.x, p.y)).collect::<Vec<_>>().join(" ")
}

/// The neighborhood's total extent, from its streets, lots, and houses.
fn total_extent(n: &Neighborhood) -> (f64, f64) {
    let mut w = 0.0f64;
    let mut h = 0.0f64;
    let mut take = |p: Pt| {
        w = w.max(p.x);
        h = h.max(p.y);
    };
    for st in &n.streets {
        for p in &st.samples.points {
            take(*p);
        }
    }
    for lot in &n.lots {
        for p in &lot.outline {
            take(*p);
        }
    }
    for house in &n.houses {
        for p in house_corners(house.center, house.rotation, house_extent(&house.house)) {
            take(p);
        }
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
