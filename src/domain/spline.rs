//! Catmull-Rom splines: control points become dense samples.
//!
//! The uniform form, endpoints duplicated so the curve starts and
//! ends on its first and last control points. A street's
//! centerline is one of these, sampled about every 2 m; the
//! samples become a `Polyline`, which the lot and mesh layers
//! walk by arc length.

use crate::domain::curve::Pt;

/// Samples a Catmull-Rom spline through `control`, about `step`
/// metres apart. The curve passes through every control point.
pub fn sample_catmull_rom(control: &[Pt], step: f64) -> Vec<Pt> {
    if control.len() < 2 {
        return control.to_vec();
    }
    let mut out = Vec::new();
    for i in 0..control.len() - 1 {
        let p0 = if i == 0 { control[0] } else { control[i - 1] };
        let p1 = control[i];
        let p2 = control[i + 1];
        let p3 = if i + 2 >= control.len() { control[control.len() - 1] } else { control[i + 2] };
        let n = (p1.dist(p2) / step).ceil().max(1.0) as usize;
        for k in 0..n {
            out.push(catrom(p0, p1, p2, p3, k as f64 / n as f64));
        }
    }
    out.push(*control.last().unwrap());
    out
}

/// One Catmull-Rom point: the uniform form, endpoints duplicated
/// by the caller so the curve starts and ends on its controls.
fn catrom(p0: Pt, p1: Pt, p2: Pt, p3: Pt, t: f64) -> Pt {
    let t2 = t * t;
    let t3 = t2 * t;
    p1.scale(2.0)
        .add(p2.sub(p0).scale(t))
        .add(p0.scale(2.0).sub(p1.scale(5.0)).add(p2.scale(4.0)).sub(p3).scale(t2))
        .add(p3.sub(p0).add(p1.scale(3.0)).sub(p2.scale(3.0)).scale(t3))
        .scale(0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catmull_rom_passes_through_every_control_point() {
        let control = vec![Pt::new(0.0, 0.0), Pt::new(30.0, 10.0), Pt::new(60.0, 0.0)];
        let pts = sample_catmull_rom(&control, 2.0);
        for c in &control {
            assert!(pts.iter().any(|p| p.dist(*c) < 1e-9), "missed {c:?}");
        }
    }

    #[test]
    fn catmull_rom_samples_are_dense() {
        let control = vec![Pt::new(0.0, 0.0), Pt::new(30.0, 10.0), Pt::new(60.0, 0.0)];
        let pts = sample_catmull_rom(&control, 2.0);
        assert!(pts.len() > 20, "only {} samples", pts.len());
        for w in pts.windows(2) {
            assert!(w[0].dist(w[1]) <= 2.5, "gap {}", w[0].dist(w[1]));
        }
    }

    #[test]
    fn catmull_rom_rounds_a_sharp_corner() {
        // A 90° turn in the control points must not stay a 90°
        // kink: the sharpest turn between sampled segments is
        // well under the corner itself.
        let control = vec![Pt::new(0.0, 0.0), Pt::new(30.0, 0.0), Pt::new(30.0, 30.0)];
        let pts = sample_catmull_rom(&control, 2.0);
        let mut worst = 0.0f64;
        for w in pts.windows(3) {
            let a = w[1].sub(w[0]).unit();
            let b = w[2].sub(w[1]).unit();
            let turn = a.dot(b).clamp(-1.0, 1.0).acos().to_degrees();
            worst = worst.max(turn);
        }
        assert!(worst < 45.0, "sharpest turn {worst:.1}°");
    }
}
