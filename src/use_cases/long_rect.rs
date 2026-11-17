//! Dividing a long rect into aspect-valid pieces.
//!
//! A single BSP cut cannot fix a long thin leaf (one child
//! would still be too long), so the alcove strip remainder — a
//! 2.4 m strip off a deep rect — is divided here into as many
//! equal pieces as the 2.0 aspect cap needs. The pieces tile
//! the rect exactly and each keeps aspect <= 2.0, and each is
//! small enough to fit the Bath cap (6.0 sq m): a piece over
//! it is over-cap for a Bath and room-sized for a Hall, and
//! too squat to strip-cut once the rect is full of strips.

use crate::domain::geometry::Rect;
use crate::domain::room_size::{ASPECT_HARD_CAP, aspect};
use crate::use_cases::split_pieces::cut_into;
use crate::use_cases::footprint::MIN_SIDE;
use crate::use_cases::quant;

/// Splits a rect along its long side into equal pieces, each
/// keeping aspect <= 2.0 and area <= the Bath cap (6.0 sq m).
/// A rect already under the cap is returned unchanged. When an
/// even split cannot make every piece bath-sized, an uneven
/// split is tried instead; the split with more bath-sized
/// pieces wins, ties going to more habitable pieces, then more
/// foyer-sized pieces, then the even split.
pub(crate) fn split_long_rect(rect: Rect) -> Vec<Rect> {
    if aspect(&rect) <= ASPECT_HARD_CAP {
        return vec![rect];
    }
    let vertical = rect.w >= rect.d;
    let (long, short) = if vertical { (rect.w, rect.d) } else { (rect.d, rect.w) };
    let bath_cap =
        crate::use_cases::caps::cap_for(crate::domain::model::Program::Bath).unwrap_or(f64::INFINITY);
    let equal = equal_split(rect, vertical, long, short, bath_cap);
    let unequal = unequal_split(rect, vertical, long, short, bath_cap);
    // Prefer the split with more bath-sized pieces (<= 6.0 sq
    // m); then more habitable pieces (>= 6.5 sq m); then more
    // foyer-sized pieces (< 6.5 sq m, the room-sized threshold
    // for a hall or entry). ADR 0015: a foyer piece in the dead
    // zone (6.0, 6.5) is worse than a habitable piece — a 2.6 m
    // rect's smallest valid piece is 6.24 sq m, dead for a bath
    // and for a bedroom, so the foyer tie-break must not prefer
    // it over a split that yields real rooms.
    let score = |pieces: &[Rect]| {
        let bath = pieces.iter().filter(|p| p.area() <= bath_cap + 1e-9).count();
        let habitable = pieces
            .iter()
            .filter(|p| p.area() >= crate::domain::room_size::HABITABLE_MIN_AREA - 1e-9)
            .count();
        let foyer = pieces
            .iter()
            .filter(|p| p.area() < crate::domain::room_size::HABITABLE_MIN_AREA - 1e-9)
            .count();
        (bath, habitable, foyer)
    };
    let valid = pieces_valid;
    if valid(&unequal) && score(&unequal) > score(&equal) {
        unequal
    } else {
        equal
    }
}

/// True when every piece keeps aspect <= 2.0 and sides >= 2.4 m.
fn pieces_valid(pieces: &[Rect]) -> bool {
    pieces.iter().all(|p| {
        aspect(p) <= ASPECT_HARD_CAP + 1e-9
            && p.w >= MIN_SIDE - 1e-9
            && p.d >= MIN_SIDE - 1e-9
    })
}

/// The even split: the fewest pieces that fit the aspect cap
/// and the Bath cap, capped by the depth floor. When the Bath
/// cap cannot be met — a deep rect whose bath-sized pieces
/// would be under the 2.4 m floor (seed 8 Craftsman: a 13.1 x
/// 5.7 rest, where 5 pieces are 2.18:1) — the fallback is the
/// aspect count, never the invalid area count.
fn equal_split(rect: Rect, vertical: bool, long: f64, short: f64, bath_cap: f64) -> Vec<Rect> {
    let k_aspect = (long / (ASPECT_HARD_CAP * short)).ceil() as usize;
    let k_area = (rect.area() / bath_cap).ceil() as usize;
    let k_max = (long / MIN_SIDE).floor() as usize;
    let k0 = k_aspect.max(k_area).min(k_max.max(1));
    for k in k0..k0 + 8 {
        let pieces = cut_into(rect, vertical, k);
        if pieces_valid(&pieces) {
            return pieces;
        }
    }
    cut_into(rect, vertical, k_aspect.max(1))
}

/// The uneven split: bath-sized pieces (`bath_cap / short`
/// deep) while the remainder exceeds one aspect-capped piece,
/// then the remainder as the last piece. The bath depth is
/// clamped to the 2.4 m floor: a wider rect (2.6 m) cannot hold
/// a bath-sized piece at all, but a floor-sized piece (2.4 m
/// deep, 6.24 sq m) is still foyer-sized for a hall or entry.
fn unequal_split(rect: Rect, vertical: bool, long: f64, short: f64, bath_cap: f64) -> Vec<Rect> {
    let fit_depth = (bath_cap / short).max(MIN_SIDE);
    let max_depth = ASPECT_HARD_CAP * short;
    let mut depths = Vec::new();
    let mut rest = long;
    while rest > max_depth + 1e-9 {
        depths.push(fit_depth);
        rest -= fit_depth;
    }
    depths.push(rest);
    cut_depths(rect, vertical, &depths)
}

/// Cuts `rect` into pieces of the given depths along its long
/// side (`vertical` cuts across x). Boundaries are quantized to
/// 0.1 m; consecutive boundaries tile the rect exactly.
fn cut_depths(rect: Rect, vertical: bool, depths: &[f64]) -> Vec<Rect> {
    let (start, len) = if vertical { (rect.x, rect.w) } else { (rect.y, rect.d) };
    let mut bounds = vec![start];
    let mut at = start;
    for &d in &depths[..depths.len().saturating_sub(1)] {
        at += d;
        bounds.push(quant(at));
    }
    bounds.push(start + len);
    bounds
        .windows(2)
        .map(|w| {
            if vertical {
                Rect::new(w[0], rect.y, w[1] - w[0], rect.d)
            } else {
                Rect::new(rect.x, w[0], rect.w, w[1] - w[0])
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_thin_rect_splits_into_aspect_valid_pieces() {
        // The Colonial alcove strip remainder: 2.4 x 10.3 m.
        let rect = Rect::new(0.0, 2.5, 2.4, 10.3);
        let pieces = split_long_rect(rect);
        assert!(pieces.len() > 1);
        for p in &pieces {
            assert!(aspect(p) <= ASPECT_HARD_CAP);
            assert!(p.w >= MIN_SIDE && p.d >= MIN_SIDE);
        }
        let area: f64 = pieces.iter().map(Rect::area).sum();
        assert!((area - rect.area()).abs() < 1e-9);
    }

    #[test]
    fn an_aspect_valid_rect_is_returned_unchanged() {
        let rect = Rect::new(1.0, 2.0, 4.0, 5.0);
        assert_eq!(split_long_rect(rect), vec![rect]);
    }

    #[test]
    fn pieces_are_small_enough_for_a_bath() {
        // A strip remainder piece must fit the Bath cap (6.0 sq
        // m): a 2.4 x 3.2 piece is 7.68 sq m, over-cap for a Bath
        // and room-sized for a Hall, and too squat to strip-cut
        // once the rect is full of strips (seed 91 Colonial).
        let rect = Rect::new(0.0, 2.5, 2.4, 9.7);
        let pieces = split_long_rect(rect);
        assert!(pieces.len() > 1);
        for p in &pieces {
            assert!(p.area() <= 6.0 + 1e-9, "piece {p:?} is {:.2} sq m", p.area());
            assert!(p.w >= MIN_SIDE - 1e-9 && p.d >= MIN_SIDE - 1e-9);
        }
    }

    #[test]
    fn an_uneven_remainder_yields_bath_sized_pieces() {
        // A 2.4 x 8.9 remainder cannot split evenly into
        // bath-sized pieces (4 pieces would be 2.2 deep, under
        // the 2.4 floor; 3 even pieces are 2.97 deep, 7.1 sq m).
        // An uneven split — 2.5, 2.5, 3.9 — gives two bath-sized
        // pieces and one bedroom-sized piece (seed 109 Colonial:
        // a third bath had no 6.0 leaf and took a 6.96 piece).
        let rect = Rect::new(0.0, 2.5, 2.4, 8.9);
        let pieces = split_long_rect(rect);
        let fitting = pieces.iter().filter(|p| p.area() <= 6.0 + 1e-9).count();
        assert!(fitting >= 2, "only {fitting} of {} pieces fit a bath", pieces.len());
        for p in &pieces {
            assert!(aspect(p) <= ASPECT_HARD_CAP);
            assert!(p.w >= MIN_SIDE - 1e-9 && p.d >= MIN_SIDE - 1e-9);
        }
        let area: f64 = pieces.iter().map(Rect::area).sum();
        assert!((area - rect.area()).abs() < 1e-9);
    }
}
