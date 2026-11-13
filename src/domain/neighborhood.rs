//! The neighborhood model: streets, lots, and placed houses.
//!
//! A neighborhood is a street network of blocks, in one of three
//! layout families (see domain::layout). Each block is subdivided
//! into lots along its streets; one house stands on each lot,
//! facing its street. All coordinates are metres, x east, y
//! south. A house's local frame has its front at the south edge
//! (largest y), matching the plan generator.

use crate::domain::geometry::Rect;
use crate::domain::layout::Layout;
use crate::domain::model::House;

/// A street: a band of asphalt. `horizontal` runs east-west.
#[derive(Debug, Clone, PartialEq)]
pub struct Street {
    pub rect: Rect,
    pub horizontal: bool,
}

/// A lot: a rectangle of land fronting one street.
#[derive(Debug, Clone, PartialEq)]
pub struct Lot {
    pub rect: Rect,
    /// Index of the street this lot fronts.
    pub street: usize,
}

/// A house placed on a lot, facing its street.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedHouse {
    pub house: House,
    /// Index of the lot this house stands on.
    pub lot: usize,
    /// Rotation of the house's local frame, in degrees:
    /// 0 faces south, 90 faces east, 180 faces north, 270 faces west.
    pub rotation: u16,
    /// The placed extent rectangle (footprint + garage + porch).
    pub rect: Rect,
}

/// A whole neighborhood.
#[derive(Debug, Clone, PartialEq)]
pub struct Neighborhood {
    pub seed: u32,
    pub blocks: (u32, u32),
    /// The street/block pattern family.
    pub layout: Layout,
    pub streets: Vec<Street>,
    pub lots: Vec<Lot>,
    pub houses: Vec<PlacedHouse>,
}

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

/// Maps a rectangle from a house's local frame into its placed
/// position, rotating the house to face its street. `placed` is
/// the placed extent rectangle; `extent` is the local extent.
pub fn place_rect(local: Rect, extent: Rect, placed: Rect, rotation: u16) -> Rect {
    match rotation {
        0 => Rect::new(placed.x + local.x, placed.y + local.y, local.w, local.d),
        90 => Rect::new(placed.x + local.y, placed.y + local.x, local.d, local.w),
        180 => Rect::new(
            placed.x + extent.w - local.x - local.w,
            placed.y + extent.d - local.y - local.d,
            local.w,
            local.d,
        ),
        270 => Rect::new(
            placed.x + extent.d - local.y - local.d,
            placed.y + local.x,
            local.d,
            local.w,
        ),
        _ => panic!("rotation must be 0, 90, 180, or 270"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Segment;
    use crate::domain::model::Footprint;
    use crate::domain::style::Style;

    fn a_house() -> House {
        House {
            seed: 42,
            style: Style::Colonial,
            footprint: Footprint {
                rects: vec![
                    Rect::new(0.0, 0.0, 12.0, 8.0),
                    Rect::new(0.0, 8.0, 6.0, 3.0),
                ],
                joins: vec![Segment::new(0.0, 8.0, 6.0, 8.0)],
                porch: Some(Rect::new(6.0, 8.0, 4.0, 3.0)),
                garage: Some(Rect::new(12.0, 2.0, 6.0, 6.0)),
            },
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
    fn placing_the_whole_extent_returns_the_placed_rect() {
        let house = a_house();
        let extent = house_extent(&house);
        let placed = Rect::new(100.0, 200.0, extent.w, extent.d);
        for rotation in [0, 180] {
            assert_eq!(place_rect(extent, extent, placed, rotation), placed);
        }
        let placed90 = Rect::new(100.0, 200.0, extent.d, extent.w);
        for rotation in [90, 270] {
            assert_eq!(place_rect(extent, extent, placed90, rotation), placed90);
        }
    }

    #[test]
    fn a_placed_footprint_rect_stays_inside_the_placed_extent() {
        let house = a_house();
        let extent = house_extent(&house);
        let placed = Rect::new(100.0, 200.0, extent.w, extent.d);
        let placed90 = Rect::new(100.0, 200.0, extent.d, extent.w);
        for r in house.footprint.rects.iter() {
            for (rotation, p) in [(0, placed), (180, placed), (90, placed90), (270, placed90)] {
                let q = place_rect(*r, extent, p, rotation);
                assert!(q.x >= p.x - 1e-9 && q.right() <= p.right() + 1e-9);
                assert!(q.y >= p.y - 1e-9 && q.bottom() <= p.bottom() + 1e-9);
                assert!((q.area() - r.area()).abs() < 1e-9, "area changed");
            }
        }
    }

    #[test]
    fn a_180_rotation_puts_the_front_on_the_opposite_side() {
        // The porch is on the south (front) edge locally; at 180 it
        // must sit on the north edge of the placed extent.
        let house = a_house();
        let Some(porch) = house.footprint.porch else { return };
        let extent = house_extent(&house);
        let placed = Rect::new(0.0, 0.0, extent.w, extent.d);
        let q = place_rect(porch, extent, placed, 180);
        assert!((q.y - placed.y).abs() < 1e-9, "porch not on the north edge");
    }
}
