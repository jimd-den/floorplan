//! Placing a house on a lot: extent, rotation, and corners.
//!
//! A house's local frame has its front at the south edge
//! (largest y), matching the plan generator. A placed house is
//! a rotation plus a centre: `rotation` is degrees, clockwise
//! from south on a map (0 faces south, 90 faces west, 180 faces
//! north, -90 faces east). The placement is a proper rotation
//! about the extent centre, so it never flips winding — the
//! old 90°/270° placement was a reflection, which mirrored
//! east- and west-facing houses.

use crate::domain::curve::Pt;
use crate::domain::geometry::Rect;
use crate::domain::model::House;

/// The bounding box of a house's footprint, garage, and porch,
/// in the house's local frame (origin at its north-west corner).
pub fn house_extent(house: &House) -> Rect {
    let mut w = 0.0f64;
    let mut d = 0.0f64;
    for r in house
        .footprint
        .rects
        .iter()
        .chain(house.footprint.garage.iter())
        .chain(house.footprint.porch.iter())
    {
        w = w.max(r.right());
        d = d.max(r.bottom());
    }
    Rect::new(0.0, 0.0, w, d)
}

/// Maps a house-local point into its placed position: rotate
/// about the extent centre by `rotation`, then translate to
/// `center`. A proper rotation, so it never flips winding.
pub fn placed_point(center: Pt, rotation: f64, extent: Rect, local: Pt) -> Pt {
    let th = rotation.to_radians();
    let (c, s) = (th.cos(), th.sin());
    let rel = Pt::new(local.x - extent.w / 2.0, local.y - extent.d / 2.0);
    Pt::new(center.x + rel.x * c - rel.y * s, center.y + rel.x * s + rel.y * c)
}

/// The placed corners of a house extent, in order: front-left,
/// front-right, back-right, back-left (front is the edge toward
/// the street).
pub fn house_corners(center: Pt, rotation: f64, extent: Rect) -> [Pt; 4] {
    let (w, d) = (extent.w, extent.d);
    [
        placed_point(center, rotation, extent, Pt::new(0.0, d)),
        placed_point(center, rotation, extent, Pt::new(w, d)),
        placed_point(center, rotation, extent, Pt::new(w, 0.0)),
        placed_point(center, rotation, extent, Pt::new(0.0, 0.0)),
    ]
}

/// The rotation (degrees, clockwise from south) whose front
/// direction is the unit vector `front`. Canonical range
/// (-180, 180]: north is 180, never -180 (atan2 returns -π
/// for the negative-zero case, which would break the range).
pub fn rotation_for(front: Pt) -> f64 {
    let deg = (-front.x).atan2(front.y).to_degrees();
    if deg <= -180.0 { deg + 360.0 } else { deg }
}

/// The front direction (unit vector) of a rotation in degrees.
pub fn front_for(rotation: f64) -> Pt {
    let th = rotation.to_radians();
    Pt::new(-th.sin(), th.cos())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Segment;
    use crate::domain::model::Footprint;
    use crate::domain::style::Style;

    fn a_house() -> House {
        let footprint = Footprint {
            rects: vec![
                Rect::new(0.0, 0.0, 12.0, 8.0),
                Rect::new(0.0, 8.0, 6.0, 3.0),
            ],
            joins: vec![Segment::new(0.0, 8.0, 6.0, 8.0)],
            porch: Some(Rect::new(6.0, 8.0, 4.0, 3.0)),
            garage: Some(Rect::new(12.0, 2.0, 6.0, 6.0)),
        };
        House {
            seed: 42,
            style: Style::Colonial,
            footprint: footprint.clone(),
            base: crate::domain::base::Base::from_footprint(&footprint),
            floors: Vec::new(),
            stairs: None,
            lights: Vec::new(),
        }
    }

    #[test]
    fn the_extent_covers_the_footprint_garage_and_porch() {
        let house = a_house();
        let e = house_extent(&house);
        for r in house.footprint.rects.iter() {
            assert!(r.right() <= e.right() + 1e-9 && r.bottom() <= e.bottom() + 1e-9);
        }
        if let Some(g) = house.footprint.garage {
            assert!(g.right() <= e.right() + 1e-9 && g.bottom() <= e.bottom() + 1e-9);
        }
        if let Some(p) = house.footprint.porch {
            assert!(p.right() <= e.right() + 1e-9 && p.bottom() <= e.bottom() + 1e-9);
        }
        assert!(e.w > 0.0 && e.d > 0.0, "empty extent");
    }

    #[test]
    fn rotation_for_and_front_for_are_inverses() {
        let q = std::f64::consts::FRAC_1_SQRT_2;
        for (deg, front) in [
            (0.0, Pt::new(0.0, 1.0)),
            (-90.0, Pt::new(1.0, 0.0)),
            (180.0, Pt::new(0.0, -1.0)),
            (90.0, Pt::new(-1.0, 0.0)),
            (45.0, Pt::new(-q, q)),
        ] {
            assert!((rotation_for(front) - deg).abs() < 0.01, "{deg}°");
            assert!(front_for(deg).sub(front).len() < 0.01, "{deg}°");
        }
    }

    #[test]
    fn a_180_rotation_puts_the_front_on_the_opposite_side() {
        // The porch's south edge is the extent's south edge; at
        // 180 it must sit on the north edge of the placed extent.
        let house = a_house();
        let Some(porch) = house.footprint.porch else { return };
        let extent = house_extent(&house);
        let center = Pt::new(100.0, 200.0);
        let q = placed_point(center, 180.0, extent, Pt::new(porch.x, porch.bottom()));
        assert!((q.y - (center.y - extent.d / 2.0)).abs() < 1e-9, "porch not on the north edge");
    }

    #[test]
    fn a_90_rotation_swaps_the_extent_axes() {
        let extent = Rect::new(0.0, 0.0, 2.0, 3.0);
        let corners = house_corners(Pt::new(0.0, 0.0), 90.0, extent);
        let xs: Vec<f64> = corners.iter().map(|p| p.x).collect();
        let ys: Vec<f64> = corners.iter().map(|p| p.y).collect();
        let (w, d) = (xs.iter().fold(f64::MIN, |a, &b| a.max(b)) - xs.iter().fold(f64::MAX, |a, &b| a.min(b)), ys.iter().fold(f64::MIN, |a, &b| a.max(b)) - ys.iter().fold(f64::MAX, |a, &b| a.min(b)));
        assert!((w - 3.0).abs() < 1e-9 && (d - 2.0).abs() < 1e-9, "rotated extent is {w}x{d}");
    }
}
