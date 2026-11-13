//! Footprint: the outline rectangles that tile the house, plus the
//! optional porch and garage. Shapes are rectangle, L, U and T.

use crate::domain::geometry::{Rect, Segment};
use crate::domain::model::Footprint;
use crate::domain::style::{FootprintShape, PorchKind, Style};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::alcove::alcove_fits;
use crate::use_cases::quant;

/// Smallest room side the BSP will allow, in metres.
pub const MIN_SIDE: f64 = 2.4;

/// Builds the footprint for a style: shape, dimensions, porch, garage.
pub fn footprint(rng: &mut Rng, style: Style) -> Footprint {
    let shape = *rng.pick(style.spec().shapes);
    footprint_with_shape(rng, style, shape)
}

/// Builds the footprint for a fixed shape (the testable core).
/// Retries until at least one rect admits a bath alcove: a
/// footprint no rect of which can hold a 2.4 x 2.5 bath leaf
/// cannot host a compliant bath, and no re-tiling can fix it.
/// Bounded so a pathological style cannot loop forever; the
/// fallback accepts the last draw, which the re-tiling loop
/// then reports as its residual.
pub fn footprint_with_shape(rng: &mut Rng, style: Style, shape: FootprintShape) -> Footprint {
    for _ in 0..20 {
        let fp = build_footprint(rng, style, shape);
        if fp.rects.iter().any(alcove_fits) {
            return fp;
        }
    }
    build_footprint(rng, style, shape)
}

fn build_footprint(rng: &mut Rng, style: Style, shape: FootprintShape) -> Footprint {
    let spec = style.spec();
    let w = dim(rng, spec.width.0, spec.width.1);
    let d = dim(rng, spec.depth.0, spec.depth.1);
    let (rects, mut joins) = match shape {
        FootprintShape::Rectangle => (vec![Rect::new(0.0, 0.0, w, d)], vec![]),
        FootprintShape::L => {
            let ww = dim(rng, 3.0, (w * 0.5).min(w - 3.0));
            let wd = dim(rng, 3.0, (d * 0.4).max(3.0));
            let main_d = d - wd;
            (
                vec![Rect::new(0.0, 0.0, w, main_d), Rect::new(0.0, main_d, ww, wd)],
                vec![Segment::new(0.0, main_d, ww, main_d)],
            )
        }
        FootprintShape::U => {
            let d1 = dim(rng, d * 0.35, d * 0.6);
            let ww1 = dim(rng, 3.0, (w * 0.3).max(3.0));
            let ww2 = dim(rng, 3.0, (w * 0.3).max(3.0));
            (
                vec![
                    Rect::new(0.0, 0.0, w, d1),
                    Rect::new(0.0, d1, ww1, d - d1),
                    Rect::new(w - ww2, d1, ww2, d - d1),
                ],
                vec![
                    Segment::new(0.0, d1, ww1, d1),
                    Segment::new(w - ww2, d1, w, d1),
                ],
            )
        }
        FootprintShape::T => {
            let d1 = dim(rng, d * 0.4, d * 0.6);
            let sw = dim(rng, 3.0, w * 0.5);
            let sx = quant((w - sw) / 2.0);
            (
                vec![Rect::new(0.0, 0.0, w, d1), Rect::new(sx, d1, sw, d - d1)],
                vec![Segment::new(sx, d1, sx + sw, d1)],
            )
        }
    };
    let porch = porch_for(rng, style, w, d);
    let mut garage = None;
    if spec.garage && rng.chance(0.6) {
        let gy = dim(rng, 0.0, d - 6.0);
        garage = Some(Rect::new(w, gy, 6.0, 6.0));
        joins.push(Segment::new(w, gy, w, gy + 6.0));
    }
    Footprint { rects, joins, porch, garage }
}

/// A dimension drawn uniformly from the 0.1 m grid over [lo, hi].
/// An empty range yields the minimum: small houses still get a wing.
fn dim(rng: &mut Rng, lo: f64, hi: f64) -> f64 {
    if hi <= lo {
        return lo;
    }
    let steps = ((hi - lo) * 10.0).round() as i64;
    lo + rng.range_i64(0, steps + 1) as f64 * 0.1
}

/// The porch rectangle, if this style and seed carry one.
fn porch_for(rng: &mut Rng, style: Style, w: f64, d: f64) -> Option<Rect> {
    let spec = style.spec();
    let want = match spec.porch {
        PorchKind::Wide => true,
        PorchKind::Optional => rng.chance(0.5),
        PorchKind::None => false,
    };
    if !want {
        return None;
    }
    let pd = dim(rng, 1.8, 2.4);
    let pw = match spec.porch {
        PorchKind::Wide => w - 1.0,
        _ => dim(rng, w * 0.5, w * 0.8),
    };
    Some(Rect::new(quant((w - pw) / 2.0), d, pw, pd))
}
