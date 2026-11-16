//! Polygon queries over plan points: area, containment, and
//! overlap. Lots and house footprints are polygons once the
//! streets curve, so the old rectangle checks (`Rect::overlaps`,
//! `contains_point`) no longer answer whether a house fits its
//! lot or two lots collide.

use crate::domain::curve::Pt;

/// The signed area (shoelace): positive when the vertices wind
/// clockwise as seen on a map (x east, y south). Zero for a
/// degenerate outline.
pub fn signed_area(poly: &[Pt]) -> f64 {
    let mut sum = 0.0;
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        sum += a.x * b.y - b.x * a.y;
    }
    sum / 2.0
}

/// True when `p` is inside the polygon (or on its edge). Ray
/// casting, even-odd rule.
pub fn point_in_poly(p: Pt, poly: &[Pt]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    if on_any_edge(p, poly) {
        return true;
    }
    ray_cast_inside(p, poly)
}

/// True when `p` is strictly inside the polygon (on an edge does
/// not count). The overlap test uses this so two polygons that
/// merely touch do not count as overlapping.
fn point_strictly_in_poly(p: Pt, poly: &[Pt]) -> bool {
    poly.len() >= 3 && !on_any_edge(p, poly) && ray_cast_inside(p, poly)
}

/// The even-odd ray cast: a ray from `p` toward +x crosses the
/// polygon an odd number of times exactly when `p` is inside.
fn ray_cast_inside(p: Pt, poly: &[Pt]) -> bool {
    let mut inside = false;
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        if (a.y > p.y) != (b.y > p.y)
            && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x
        {
            inside = !inside;
        }
    }
    inside
}

/// True when `p` lies on any edge of the polygon.
fn on_any_edge(p: Pt, poly: &[Pt]) -> bool {
    (0..poly.len()).any(|i| on_segment(p, poly[i], poly[(i + 1) % poly.len()]))
}

/// True when `p` lies on the segment a–b (within tolerance).
fn on_segment(p: Pt, a: Pt, b: Pt) -> bool {
    let ab = b.sub(a);
    let ap = p.sub(a);
    let cross = ab.x * ap.y - ab.y * ap.x;
    if cross.abs() > 1e-9 {
        return false;
    }
    let dot = ap.dot(ab);
    dot >= -1e-9 && dot <= ab.dot(ab) + 1e-9
}

/// The signed area of the triangle a–b–c (twice the area, sign
/// by winding). Zero when the three points are collinear.
fn orient(a: Pt, b: Pt, c: Pt) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// True when the two segments share any point, including a
/// touching endpoint.
pub fn segments_intersect(a: Pt, b: Pt, c: Pt, d: Pt) -> bool {
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);
    let eps = 1e-9;
    if ((o1 > eps && o2 < -eps) || (o1 < -eps && o2 > eps))
        && ((o3 > eps && o4 < -eps) || (o3 < -eps && o4 > eps))
    {
        return true;
    }
    o1.abs() <= eps && on_segment(c, a, b)
        || o2.abs() <= eps && on_segment(d, a, b)
        || o3.abs() <= eps && on_segment(a, c, d)
        || o4.abs() <= eps && on_segment(b, c, d)
}

/// True when the two segments cross at a point interior to both
/// (touching or collinear does not count). The overlap test uses
/// this so two polygons that merely touch do not count.
fn segments_cross(a: Pt, b: Pt, c: Pt, d: Pt) -> bool {
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);
    let eps = 1e-9;
    ((o1 > eps && o2 < -eps) || (o1 < -eps && o2 > eps))
        && ((o3 > eps && o4 < -eps) || (o3 < -eps && o4 > eps))
}

/// True when the two simple polygons share area. Sharing only an
/// edge or a point is not overlap — two lots may touch.
pub fn polys_overlap(a: &[Pt], b: &[Pt]) -> bool {
    if a.iter().any(|&p| point_strictly_in_poly(p, b)) {
        return true;
    }
    if b.iter().any(|&p| point_strictly_in_poly(p, a)) {
        return true;
    }
    for i in 0..a.len() {
        for j in 0..b.len() {
            if segments_cross(a[i], a[(i + 1) % a.len()], b[j], b[(j + 1) % b.len()]) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x: f64, y: f64, s: f64) -> Vec<Pt> {
        vec![
            Pt::new(x, y),
            Pt::new(x + s, y),
            Pt::new(x + s, y + s),
            Pt::new(x, y + s),
        ]
    }

    #[test]
    fn signed_area_is_width_times_depth_for_a_square() {
        assert!((signed_area(&square(0.0, 0.0, 10.0)) - 100.0).abs() < 1e-9);
        assert!((signed_area(&square(2.0, 3.0, 4.0)) - 16.0).abs() < 1e-9);
    }

    #[test]
    fn a_point_inside_is_inside_and_outside_is_outside() {
        let sq = square(0.0, 0.0, 10.0);
        assert!(point_in_poly(Pt::new(5.0, 5.0), &sq));
        assert!(!point_in_poly(Pt::new(15.0, 5.0), &sq));
        assert!(!point_in_poly(Pt::new(-1.0, -1.0), &sq));
    }

    #[test]
    fn a_point_on_the_edge_counts_as_inside() {
        let sq = square(0.0, 0.0, 10.0);
        assert!(point_in_poly(Pt::new(0.0, 5.0), &sq));
        assert!(point_in_poly(Pt::new(10.0, 10.0), &sq));
    }

    #[test]
    fn crossing_segments_intersect_and_parallel_ones_do_not() {
        let cross = segments_intersect(Pt::new(0.0, 0.0), Pt::new(10.0, 10.0), Pt::new(0.0, 10.0), Pt::new(10.0, 0.0));
        assert!(cross);
        let parallel = segments_intersect(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), Pt::new(0.0, 1.0), Pt::new(10.0, 1.0));
        assert!(!parallel);
        let touching = segments_intersect(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), Pt::new(10.0, 0.0), Pt::new(10.0, 10.0));
        assert!(touching);
    }

    #[test]
    fn overlapping_polygons_overlap_and_touching_ones_do_not() {
        assert!(polys_overlap(&square(0.0, 0.0, 10.0), &square(5.0, 5.0, 10.0)));
        assert!(polys_overlap(&square(0.0, 0.0, 10.0), &square(2.0, 2.0, 2.0)));
        assert!(!polys_overlap(&square(0.0, 0.0, 10.0), &square(20.0, 0.0, 10.0)));
        assert!(!polys_overlap(&square(0.0, 0.0, 10.0), &square(10.0, 0.0, 10.0)));
    }
}
