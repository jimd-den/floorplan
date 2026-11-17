//! Cutting a rect into equal pieces and recording the
//! boundaries between them as splits, so the skeleton walls
//! them. Shared by `bsp` (the aspect post-pass), `alcove`
//! (strip remainders) and `long_rect` (uneven splits). The
//! piece-boundary sense is `split_long_rect`'s, not the
//! alcove's (see `record_piece_splits`).

use crate::domain::geometry::{Axis, Rect, Segment};
use crate::use_cases::bsp::Split;
use crate::use_cases::quant;

/// Records the boundary between each pair of pieces as a split,
/// so the skeleton walls it. `vertical` is split_long_rect's
/// sense: the pieces vary in x when true, in y when false. A
/// wide leaf's boundaries are vertical lines; a tall leaf's are
/// horizontal. (The alcove's `vertical` means the strip is
/// vertical, the opposite sense — do not copy it.)
pub(crate) fn record_piece_splits(splits: &mut Vec<Split>, pieces: &[Rect], vertical: bool) {
    for (j, piece) in pieces.iter().enumerate() {
        if j == 0 {
            continue;
        }
        let (axis, at, line) = if vertical {
            let at = piece.x;
            (Axis::Vertical, at, Segment::new(at, piece.y, at, piece.bottom()))
        } else {
            let at = piece.y;
            (Axis::Horizontal, at, Segment::new(piece.x, at, piece.right(), at))
        };
        splits.push(Split { axis, at, line });
    }
}

/// Cuts `rect` into `k` equal pieces along its long side
/// (`vertical` cuts across x). Boundaries are quantized to
/// 0.1 m; consecutive boundaries tile the rect exactly.
pub(crate) fn cut_into(rect: Rect, vertical: bool, k: usize) -> Vec<Rect> {
    let (start, len) = if vertical { (rect.x, rect.w) } else { (rect.y, rect.d) };
    let mut bounds = vec![start];
    for i in 1..k {
        bounds.push(quant(start + i as f64 * len / k as f64));
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
