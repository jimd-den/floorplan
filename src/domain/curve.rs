//! Curved geometry: plan points and sampled polylines. Everything
//! is in plan metres (x east, y south). A polyline is a sampled
//! curve with arc lengths, so a street can be walked by distance
//! and offset by a fixed width; `domain::spline` turns a street's
//! control points into the samples this type holds.

/// A point in the plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

impl Pt {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Point addition. Method-call form, not the `Add` trait:
    /// the geometry code chains `add`/`sub`/`scale` in one
    /// expression, and the operator traits would need `Mul`
    /// too for the same chains. Clippy's `should_implement_trait`
    /// is silenced deliberately for this domain API.
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, o: Pt) -> Pt {
        Pt::new(self.x + o.x, self.y + o.y)
    }

    /// Point subtraction; see `add` for why this is a method.
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, o: Pt) -> Pt {
        Pt::new(self.x - o.x, self.y - o.y)
    }

    pub fn scale(self, s: f64) -> Pt {
        Pt::new(self.x * s, self.y * s)
    }

    pub fn dot(self, o: Pt) -> f64 {
        self.x * o.x + self.y * o.y
    }

    pub fn len(self) -> f64 {
        self.dot(self).sqrt()
    }

    /// The unit vector in this direction; (0, 0) stays (0, 0).
    pub fn unit(self) -> Pt {
        let l = self.len();
        if l < 1e-12 { Pt::new(0.0, 0.0) } else { self.scale(1.0 / l) }
    }

    /// The point rotated 90°: (-y, x). Offsetting a polyline by
    /// +d moves it to this side of the direction of travel.
    pub fn perp(self) -> Pt {
        Pt::new(-self.y, self.x)
    }

    pub fn dist(self, o: Pt) -> f64 {
        self.sub(o).len()
    }

    pub fn lerp(self, o: Pt, t: f64) -> Pt {
        Pt::new(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t)
    }
}

/// A sampled curve: points plus their cumulative arc lengths, so
/// any distance along the curve maps to a point and a tangent.
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    pub points: Vec<Pt>,
    pub lengths: Vec<f64>,
}

impl Polyline {
    pub fn new(points: Vec<Pt>) -> Self {
        let mut lengths = vec![0.0];
        for w in points.windows(2) {
            lengths.push(lengths.last().unwrap() + w[0].dist(w[1]));
        }
        Self { points, lengths }
    }

    /// Total arc length.
    pub fn len(&self) -> f64 {
        self.lengths.last().copied().unwrap_or(0.0)
    }

    /// The point `s` metres along the curve, clamped to the ends.
    pub fn point_at(&self, s: f64) -> Pt {
        let Some(first) = self.points.first() else { return Pt::new(0.0, 0.0) };
        if self.points.len() == 1 {
            return *first;
        }
        let s = s.clamp(0.0, self.len());
        let i = self
            .lengths
            .partition_point(|&l| l <= s)
            .saturating_sub(1)
            .min(self.points.len() - 2);
        let (a, b) = (self.points[i], self.points[i + 1]);
        let span = self.lengths[i + 1] - self.lengths[i];
        if span < 1e-12 {
            return b;
        }
        a.lerp(b, (s - self.lengths[i]) / span)
    }

    /// The unit tangent `s` metres along the curve (central
    /// difference, so it is smooth across a sample joint).
    pub fn tangent_at(&self, s: f64) -> Pt {
        self.point_at(s + 1.0).sub(self.point_at(s - 1.0)).unit()
    }

    /// The unit normal `s` metres along the curve (tangent rotated 90°).
    pub fn normal_at(&self, s: f64) -> Pt {
        self.tangent_at(s).perp()
    }

    /// The points offset by `d` metres along the normal: the
    /// parallel curve a sidewalk or a lot front follows.
    pub fn offset(&self, d: f64) -> Vec<Pt> {
        self.lengths
            .iter()
            .map(|&s| self.point_at(s).add(self.normal_at(s).scale(d)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> Polyline {
        Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), Pt::new(10.0, 10.0)])
    }

    #[test]
    fn point_at_interpolates_along_the_curve() {
        let p = line();
        assert_eq!(p.len(), 20.0);
        assert_eq!(p.point_at(5.0), Pt::new(5.0, 0.0));
        assert_eq!(p.point_at(15.0), Pt::new(10.0, 5.0));
        assert_eq!(p.point_at(-3.0), Pt::new(0.0, 0.0));
        assert_eq!(p.point_at(99.0), Pt::new(10.0, 10.0));
    }

    #[test]
    fn tangent_follows_the_curve() {
        let p = line();
        assert_eq!(p.tangent_at(5.0), Pt::new(1.0, 0.0));
        assert_eq!(p.tangent_at(15.0), Pt::new(0.0, 1.0));
    }

    #[test]
    fn offset_moves_to_the_normal_side() {
        let p = Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(10.0, 0.0)]);
        let off = p.offset(2.0);
        assert_eq!(off, vec![Pt::new(0.0, 2.0), Pt::new(10.0, 2.0)]);
    }
}
