//! Door and window SVG: swings, gaps and double lines.

use crate::domain::geometry::Axis;
use crate::domain::model::{Floor, Opening};

/// A door: a white gap in the wall, a quarter-circle swing into the
/// `from` room, and the open leaf. The sweep flag comes from the
/// cross product of the swing's two arms.
pub(crate) fn door_svg(floor: &Floor, o: &Opening, oy: f64) -> String {
    let w = o.width;
    let (hx, hy) = if o.axis == Axis::Horizontal { (o.x - w / 2.0, o.y) } else { (o.x, o.y - w / 2.0) };
    let (sx, sy) = if o.axis == Axis::Horizontal { (o.x + w / 2.0, o.y) } else { (o.x, o.y + w / 2.0) };
    let (dx, dy) = floor
        .rooms
        .iter()
        .find(|r| r.id == o.from)
        .map(|r| {
            if o.axis == Axis::Horizontal {
                if (r.rect.bottom() - o.y).abs() < 1e-9 { (0.0, -1.0) } else { (0.0, 1.0) }
            } else if (r.rect.right() - o.x).abs() < 1e-9 {
                (-1.0, 0.0)
            } else {
                (1.0, 0.0)
            }
        })
        .unwrap_or((0.0, 1.0));
    let (ex, ey) = (hx + dx * w, hy + dy * w);
    let cross = (sx - hx) * (ey - hy) - (sy - hy) * (ex - hx);
    let sweep = if cross > 0.0 { 1 } else { 0 };
    let mut s = String::new();
    if o.axis == Axis::Horizontal {
        s.push_str(&gap_line(o.x - w / 2.0, o.y + oy, o.x + w / 2.0, o.y + oy));
    } else {
        s.push_str(&gap_line(o.x, o.y - w / 2.0 + oy, o.x, o.y + w / 2.0 + oy));
    }
    s.push_str(&format!(
        "<path class=\"swing\" d=\"M {:.2} {:.2} A {:.2} {:.2} 0 0 {} {:.2} {:.2}\"/>\n",
        sx, sy + oy, w, w, sweep, ex, ey + oy
    ));
    s.push_str(&format!(
        "<line class=\"swing\" x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
        hx, hy + oy, ex, ey + oy
    ));
    s
}

/// A window: a white gap in the wall with two parallel lines.
pub(crate) fn window_svg(o: &Opening, oy: f64) -> String {
    let w = o.width;
    let mut s = String::new();
    if o.axis == Axis::Horizontal {
        s.push_str(&gap_line(o.x - w / 2.0, o.y + oy, o.x + w / 2.0, o.y + oy));
        s.push_str(&win_line(o.x - w / 2.0, o.y - 0.06 + oy, o.x + w / 2.0, o.y - 0.06 + oy));
        s.push_str(&win_line(o.x - w / 2.0, o.y + 0.06 + oy, o.x + w / 2.0, o.y + 0.06 + oy));
    } else {
        s.push_str(&gap_line(o.x, o.y - w / 2.0 + oy, o.x, o.y + w / 2.0 + oy));
        s.push_str(&win_line(o.x - 0.06, o.y - w / 2.0 + oy, o.x - 0.06, o.y + w / 2.0 + oy));
        s.push_str(&win_line(o.x + 0.06, o.y - w / 2.0 + oy, o.x + 0.06, o.y + w / 2.0 + oy));
    }
    s
}

fn gap_line(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    format!("<line class=\"gap\" x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\"/>\n")
}

fn win_line(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    format!("<line class=\"win\" x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\"/>\n")
}
