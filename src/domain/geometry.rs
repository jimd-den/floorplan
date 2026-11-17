//! Rectangles and wall segments. Everything is axis-aligned.
//!
//! `Segment` lives in `domain::segment` (split out when this file
//! passed the line ceiling); it is re-exported here so existing
//! `geometry::Segment` paths keep working.

/// Tolerance for comparing coordinates that come from the same BSP cuts.
pub const EPS: f64 = 1e-9;

pub use crate::domain::segment::Segment;

/// Quantizes a metre value to 0.1 m. Every seeded dimension in the
/// generator goes through this, so plans stay readable and the
/// variety count can be computed from finite decision sets. It lives
/// in the domain because the frame (domain) quantizes its member
/// positions; the use-case layer re-exports it.
pub fn quant(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// A split direction. `Vertical` cuts with a vertical line (left/right);
/// `Horizontal` cuts with a horizontal line (top/bottom).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Vertical,
    Horizontal,
}

/// An axis-aligned rectangle: origin (x, y) is the north-west corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub d: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, d: f64) -> Self {
        Self { x, y, w, d }
    }

    pub fn area(&self) -> f64 {
        self.w * self.d
    }

    pub fn right(&self) -> f64 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.d
    }

    pub fn center(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.d / 2.0)
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x - EPS && x <= self.right() + EPS && y >= self.y - EPS && y <= self.bottom() + EPS
    }

    /// True when the two rectangles share a positive area.
    pub fn overlaps(&self, other: &Rect) -> bool {
        let x = self.right().min(other.right()) - self.x.max(other.x);
        let y = self.bottom().min(other.bottom()) - self.y.max(other.y);
        x > EPS && y > EPS
    }

    /// Splits the rectangle into two along `axis` at coordinate `at`.
    pub fn split(&self, axis: Axis, at: f64) -> (Rect, Rect) {
        match axis {
            Axis::Vertical => (
                Rect::new(self.x, self.y, at - self.x, self.d),
                Rect::new(at, self.y, self.right() - at, self.d),
            ),
            Axis::Horizontal => (
                Rect::new(self.x, self.y, self.w, at - self.y),
                Rect::new(self.x, at, self.w, self.bottom() - at),
            ),
        }
    }

    /// The four wall segments: north, east, south, west.
    pub fn edges(&self) -> [Segment; 4] {
        [
            Segment::new(self.x, self.y, self.right(), self.y),
            Segment::new(self.right(), self.y, self.right(), self.bottom()),
            Segment::new(self.x, self.bottom(), self.right(), self.bottom()),
            Segment::new(self.x, self.y, self.x, self.bottom()),
        ]
    }
}

/// The wall segment two rectangles share, if their edges touch.
pub fn shared_edge(a: &Rect, b: &Rect) -> Option<Segment> {
    // a east vs b west, a west vs b east, a south vs b north, a north vs b south.
    let pairs = [(1, 3), (3, 1), (2, 0), (0, 2)];
    for (ea, eb) in pairs {
        if let Some(seg) = a.edges()[ea].overlap(&b.edges()[eb]) {
            return Some(seg);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_is_width_times_depth() {
        assert_eq!(Rect::new(0.0, 0.0, 6.0, 4.0).area(), 24.0);
    }

    #[test]
    fn split_vertical_cuts_left_and_right() {
        let (a, b) = Rect::new(0.0, 0.0, 10.0, 6.0).split(Axis::Vertical, 4.0);
        assert_eq!(a, Rect::new(0.0, 0.0, 4.0, 6.0));
        assert_eq!(b, Rect::new(4.0, 0.0, 6.0, 6.0));
    }

    #[test]
    fn split_horizontal_cuts_top_and_bottom() {
        let (a, b) = Rect::new(0.0, 0.0, 10.0, 6.0).split(Axis::Horizontal, 2.5);
        assert_eq!(a, Rect::new(0.0, 0.0, 10.0, 2.5));
        assert_eq!(b, Rect::new(0.0, 2.5, 10.0, 3.5));
    }

    #[test]
    fn overlaps_needs_shared_area_not_just_an_edge() {
        let r = Rect::new(0.0, 0.0, 4.0, 4.0);
        assert!(r.overlaps(&Rect::new(2.0, 2.0, 4.0, 4.0)));
        assert!(!r.overlaps(&Rect::new(4.0, 0.0, 4.0, 4.0)));
        assert!(!r.overlaps(&Rect::new(5.0, 5.0, 2.0, 2.0)));
    }

    #[test]
    fn shared_edge_finds_the_touching_wall() {
        let a = Rect::new(0.0, 0.0, 4.0, 4.0);
        let b = Rect::new(4.0, 1.0, 3.0, 2.0);
        assert_eq!(
            shared_edge(&a, &b),
            Some(Segment::new(4.0, 1.0, 4.0, 3.0))
        );
        let c = Rect::new(0.0, 4.0, 4.0, 4.0);
        assert_eq!(
            shared_edge(&a, &c),
            Some(Segment::new(0.0, 4.0, 4.0, 4.0))
        );
        let d = Rect::new(9.0, 9.0, 2.0, 2.0);
        assert_eq!(shared_edge(&a, &d), None);
    }
}
