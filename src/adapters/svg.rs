//! SVG renderer: one house as one readable SVG document.
//!
//! Units are metres (viewBox in metres); walls are lines with a real
//! 0.15 m stroke, doors are quarter-circle swings, windows are
//! double lines, rooms are labelled with program and dimensions.

use crate::adapters::svg_openings::{door_svg, window_svg};
use crate::domain::model::{House, OpeningKind};
use crate::use_cases::walls::wall_segments;

/// Renders a whole house (all floors, stacked) as an SVG document.
pub fn render_svg(house: &House) -> String {
    let (min_x, min_y, w, h) = bounds(house);
    let gap = 2.0;
    let n = house.floors.len() as f64;
    let floors_h = h * n + gap * (n - 1.0).max(0.0);
    let vw = w + 2.0;
    let vh = floors_h + 2.0;
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"{:.2} {:.2} {:.2} {:.2}\">\n",
        (vw * 40.0) as u32,
        (vh * 40.0) as u32,
        min_x - 1.0,
        min_y - 1.0,
        vw,
        vh
    ));
    s.push_str("<style>text{font-family:sans-serif;font-size:0.35px;fill:#222} .dim{font-size:0.28px;fill:#555} .wall{stroke:#111;stroke-width:0.15} .win{stroke:#258;stroke-width:0.06} .gap{stroke:#fff;stroke-width:0.16} .swing{fill:none;stroke:#888;stroke-width:0.03} .porch{fill:none;stroke:#888;stroke-width:0.05;stroke-dasharray:0.3 0.2}</style>\n");
    for (fi, floor) in house.floors.iter().enumerate() {
        let oy = fi as f64 * (h + gap);
        s.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\">{}</text>\n",
            min_x,
            min_y + oy - 0.35,
            floor_label(floor.level)
        ));
        if fi == 0
            && let Some(p) = house.footprint.porch
        {
            s.push_str(&format!(
                "<rect class=\"porch\" x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\"/>\n",
                p.x, p.y + oy, p.w, p.d
            ));
        }
        for seg in wall_segments(&floor.rooms) {
            s.push_str(&format!(
                "<line class=\"wall\" x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
                seg.x1,
                seg.y1 + oy,
                seg.x2,
                seg.y2 + oy
            ));
        }
        for o in &floor.openings {
            match o.kind {
                OpeningKind::Door | OpeningKind::EntryDoor => s.push_str(&door_svg(floor, o, oy)),
                OpeningKind::Window => s.push_str(&window_svg(o, oy)),
            }
        }
        for room in &floor.rooms {
            let (cx, cy) = room.rect.center();
            s.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\">{}</text>\n",
                cx,
                cy - 0.1 + oy,
                room.program.name()
            ));
            s.push_str(&format!(
                "<text class=\"dim\" x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\">{}×{} m</text>\n",
                cx,
                cy + 0.3 + oy,
                fmt_m(room.rect.w),
                fmt_m(room.rect.d)
            ));
        }
    }
    let nx = min_x + w - 0.3;
    let ny = min_y - 0.6;
    s.push_str(&format!(
        "<polygon points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"#111\"/>\n",
        nx, ny, nx - 0.15, ny + 0.5, nx + 0.15, ny + 0.5
    ));
    s.push_str(&format!("<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\">N</text>\n", nx, ny + 0.9));
    s.push_str(&format!(
        "<text x=\"{:.2}\" y=\"{:.2}\">Scale: 1 unit = 1 m — {} — seed {}</text>\n",
        min_x,
        min_y - 1.0 + vh - 0.3,
        house.style.name(),
        house.seed
    ));
    s.push_str("</svg>\n");
    s
}

/// Bounding box of the footprint, garage and porch: (min_x, min_y, w, h).
fn bounds(house: &House) -> (f64, f64, f64, f64) {
    let mut rects = house.footprint.rects.clone();
    if let Some(g) = house.footprint.garage {
        rects.push(g);
    }
    if let Some(p) = house.footprint.porch {
        rects.push(p);
    }
    let min_x = rects.iter().map(|r| r.x).fold(f64::INFINITY, f64::min);
    let min_y = rects.iter().map(|r| r.y).fold(f64::INFINITY, f64::min);
    let max_x = rects.iter().map(|r| r.right()).fold(f64::NEG_INFINITY, f64::max);
    let max_y = rects.iter().map(|r| r.bottom()).fold(f64::NEG_INFINITY, f64::max);
    (min_x, min_y, max_x - min_x, max_y - min_y)
}

fn floor_label(level: u8) -> String {
    match level {
        0 => "Ground floor".to_string(),
        1 => "Upper floor".to_string(),
        n => format!("Floor {n}"),
    }
}

/// Metres with trailing zeros trimmed: 4.20 -> "4.2", 4.00 -> "4".
fn fmt_m(v: f64) -> String {
    format!("{v:.2}").trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;

    #[test]
    fn svg_has_xmlns_and_a_metre_viewbox() {
        let svg = render_svg(&generate_house(7, Style::Ranch));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("viewBox=\""), "viewBox missing");
        assert!(svg.starts_with("<?xml"));
    }

    #[test]
    fn svg_has_walls_doors_windows_and_labels() {
        let svg = render_svg(&generate_house(7, Style::Ranch));
        assert!(svg.contains("<line"), "no wall lines");
        assert!(svg.contains("<path"), "no door swing arcs");
        assert!(svg.contains("living"), "no living label");
        assert!(svg.contains('×'), "no room dimensions");
    }

    #[test]
    fn two_floors_stack_with_labels() {
        let svg = render_svg(&generate_house(7, Style::Colonial));
        assert!(svg.contains("Ground floor"));
        assert!(svg.contains("Upper floor"));
    }

    #[test]
    fn svg_has_a_north_arrow_and_scale_note() {
        let svg = render_svg(&generate_house(7, Style::Ranch));
        assert!(svg.contains('N'), "no north arrow");
        assert!(svg.contains("1 unit = 1 m"), "no scale note");
    }
}
