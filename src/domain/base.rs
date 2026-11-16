//! Base: the foundation slab a house stands on.
//!
//! A house is built base-first: one concrete slab under the whole
//! footprint (plus the garage), 0.3 m thick, its top at floor level.
//! The walls and floors sit on it; the porch sits on its own slab.
//! The mesh draws one box per slab rect; the JSON carries the rects
//! and the thickness.

use crate::domain::geometry::Rect;
use crate::domain::model::Footprint;

/// Thickness of the foundation slab, in metres.
pub const BASE_THICKNESS: f64 = 0.3;

/// The foundation slab: the footprint rects (plus the garage) at
/// the standard thickness. Its top is floor level (y = 0); the slab
/// reaches `thickness` below it.
#[derive(Debug, Clone, PartialEq)]
pub struct Base {
    pub rects: Vec<Rect>,
    pub thickness: f64,
}

impl Base {
    /// The slab for a footprint: its rects plus the garage, at the
    /// standard thickness. The porch sits on its own slab.
    pub fn from_footprint(fp: &Footprint) -> Base {
        let mut rects = fp.rects.clone();
        if let Some(g) = fp.garage {
            rects.push(g);
        }
        Base { rects, thickness: BASE_THICKNESS }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Segment;

    fn footprint() -> Footprint {
        Footprint {
            rects: vec![Rect::new(0.0, 0.0, 12.0, 8.0), Rect::new(0.0, 8.0, 6.0, 3.0)],
            joins: vec![Segment::new(0.0, 8.0, 6.0, 8.0)],
            porch: Some(Rect::new(6.0, 8.0, 4.0, 3.0)),
            garage: Some(Rect::new(12.0, 2.0, 6.0, 6.0)),
        }
    }

    #[test]
    fn the_slab_covers_every_footprint_rect_and_the_garage() {
        let fp = footprint();
        let base = Base::from_footprint(&fp);
        assert_eq!(base.rects.len(), 3, "two footprint rects plus the garage");
        for r in &fp.rects {
            assert!(base.rects.contains(r), "slab misses footprint rect {r:?}");
        }
        assert!(base.rects.contains(&fp.garage.unwrap()), "slab misses the garage");
    }

    #[test]
    fn the_slab_is_three_tenths_thick() {
        let base = Base::from_footprint(&footprint());
        assert!((base.thickness - BASE_THICKNESS).abs() < 1e-9, "thickness {}", base.thickness);
        assert!((base.thickness - 0.3).abs() < 1e-9, "a foundation slab is 0.3 m");
    }

    #[test]
    fn a_house_without_a_garage_slabs_only_its_footprint() {
        let mut fp = footprint();
        fp.garage = None;
        let base = Base::from_footprint(&fp);
        assert_eq!(base.rects, fp.rects, "no garage, no extra slab rect");
    }
}
