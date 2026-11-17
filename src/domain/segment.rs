//! A wall segment: one axis-aligned line, stored min-corner first.
//! Segments are the edges walls run along and the spans openings
//! cut into. Split out of `geometry.rs` when that file passed the
//! line ceiling; the rectangle keeps the shared-edge computation.

use crate::domain::geometry::EPS;

/// One axis-aligned wall segment, stored min-corner first.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Segment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl Segment {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        if (x1, y1) <= (x2, y2) {
            Self { x1, y1, x2, y2 }
        } else {
            Self { x1: x2, y1: y2, x2: x1, y2: y1 }
        }
    }

    pub fn len(&self) -> f64 {
        ((self.x2 - self.x1).powi(2) + (self.y2 - self.y1).powi(2)).sqrt()
    }

    pub fn is_horizontal(&self) -> bool {
        (self.y2 - self.y1).abs() < EPS
    }

    pub fn is_vertical(&self) -> bool {
        (self.x2 - self.x1).abs() < EPS
    }

    /// The collinear overlap of two segments, if they share a length.
    pub fn overlap(&self, other: &Segment) -> Option<Segment> {
        if self.is_horizontal() && other.is_horizontal() && (self.y1 - other.y1).abs() < EPS {
            let lo = self.x1.max(other.x1);
            let hi = self.x2.min(other.x2);
            if hi - lo > EPS {
                return Some(Segment::new(lo, self.y1, hi, self.y1));
            }
        }
        if self.is_vertical() && other.is_vertical() && (self.x1 - other.x1).abs() < EPS {
            let lo = self.y1.max(other.y1);
            let hi = self.y2.min(other.y2);
            if hi - lo > EPS {
                return Some(Segment::new(self.x1, lo, self.x1, hi));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_overlap_returns_the_shared_part() {
        let a = Segment::new(0.0, 3.0, 10.0, 3.0);
        let b = Segment::new(4.0, 3.0, 7.0, 3.0);
        assert_eq!(a.overlap(&b), Some(Segment::new(4.0, 3.0, 7.0, 3.0)));
        let c = Segment::new(0.0, 4.0, 10.0, 4.0);
        assert_eq!(a.overlap(&c), None);
    }
}
