//! Combinatorial variety floors.
//!
//! Each bound is the product of the independent decisions, every
//! factor taken at its minimum, so the result is a floor and not an
//! estimate. The hashes and the sampled checks live in
//! `use_cases::variety`; nothing here generates anything.

/// Conservative lower bound on distinct houses across the six styles:
/// 6 styles × 861 footprints × 2 room counts × 40 first cuts × 12 second cuts ×
/// 10 third cuts × 50 door positions × 8 entry positions × 27 window widths ×
/// 11 window positions = 5,891,719,680,000 (> 5.89 × 10¹² > 10¹²).
pub fn house_variety_lower_bound() -> u64 {
    6 * 861 * 2 * 40 * 12 * 10 * 50 * 8 * 27 * 11
}

/// Conservative lower bound on distinct chunk layouts from the
/// lattice alone (ADR 0011): the cell's four corner nodes each
/// jitter 61 x 61 ways on the 1 m grid (`JITTER = 30`), and each of
/// the four cell edges is kept or dropped. The geometry *is* that
/// tuple — the jitter is applied directly and the edge decision is a
/// boolean — so every combination is a distinct layout:
/// 61^8 x 2^4 = 3,067,317,007,956,496. The 32-bit seed cannot
/// address more than 2^32 layouts at one cell position; the map is
/// unbounded, so the world's layout count has no such cap. See
/// `tests/variety_bounds.rs` for the sampled distinctness check.
pub fn neighborhood_variety_lower_bound() -> u64 {
    61u64.pow(8) * 16
}
