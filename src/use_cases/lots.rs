//! Lot laying: a block becomes lots, each lot gets one house
//! facing its street. Supports `Low`, `Medium`, and `High`
//! residential density while guaranteeing every house fits its lot
//! with setbacks (`FRONT_SETBACK`, `BACK_SETBACK`, `SIDE_SETBACK`).

use crate::domain::geometry::Rect;
use crate::domain::layout::Density;
use crate::domain::neighborhood::{Lot, PlacedHouse, Street, house_extent};
use crate::domain::style::Style;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::generate::generate_house;
use crate::use_cases::layout::BlockPlan;
use crate::use_cases::neighborhood::{FRONT_SETBACK, SIDE_LOT_D};
use crate::use_cases::quant;

/// Lays out one block at `Density::Low`.
pub fn lay_block(
    rng: &mut Rng,
    lots: &mut Vec<Lot>,
    houses: &mut Vec<PlacedHouse>,
    block_rect: Rect,
    streets: (usize, usize, usize, usize),
) {
    lay_block_with_density(rng, lots, houses, block_rect, streets, Density::Low);
}

/// Lays out one block at the given residential density.
pub fn lay_block_with_density(
    rng: &mut Rng,
    lots: &mut Vec<Lot>,
    houses: &mut Vec<PlacedHouse>,
    block_rect: Rect,
    streets: (usize, usize, usize, usize),
    density: Density,
) {
    let (north, south, west, east) = streets;
    let (x0, y0, w, d) = (block_rect.x, block_rect.y, block_rect.w, block_rect.d);
    let lot_d = match density {
        Density::Low => quant(rng.range_f64(24.0, 28.0)),
        Density::Medium => quant(rng.range_f64(24.0, 26.0)),
        Density::High => 24.0,
    };
    let per_side = match density {
        Density::Low => 1 + rng.range_usize(2),
        Density::Medium => 3,
        Density::High => 4 + rng.range_usize(2),
    };
    lay_strip(rng, lots, houses, Rect::new(x0, y0, w, lot_d), per_side, 180, north);
    lay_strip(rng, lots, houses, Rect::new(x0, y0 + d - lot_d, w, lot_d), per_side, 0, south);
    let mid_top = y0 + lot_d;
    let mid_h = d - 2.0 * lot_d;
    if mid_h >= 15.0 {
        let side_lots = match density {
            Density::Low => 1,
            Density::Medium | Density::High => if mid_h >= 30.0 { 2 } else { 1 },
        };
        let sub_h = mid_h / side_lots as f64;
        for k in 0..side_lots {
            let sy = mid_top + k as f64 * sub_h;
            let west_lot = Rect::new(x0, sy, SIDE_LOT_D, sub_h);
            place_house(rng, lots, houses, west_lot, west, 270, true);
            let east_lot = Rect::new(x0 + w - SIDE_LOT_D, sy, SIDE_LOT_D, sub_h);
            place_house(rng, lots, houses, east_lot, east, 90, true);
        }
    }
}

/// Lays out a cul-de-sac court block at `Density::Low`.
pub fn lay_court(
    rng: &mut Rng,
    lots: &mut Vec<Lot>,
    houses: &mut Vec<PlacedHouse>,
    block_plan: &BlockPlan,
    streets: &[Street],
) {
    lay_court_with_density(rng, lots, houses, block_plan, streets, Density::Low);
}

/// Lays out a cul-de-sac court block at the given residential density.
pub fn lay_court_with_density(
    rng: &mut Rng,
    lots: &mut Vec<Lot>,
    houses: &mut Vec<PlacedHouse>,
    block_plan: &BlockPlan,
    streets: &[Street],
    density: Density,
) {
    let Some(court) = &block_plan.court else { return };
    let block = block_plan.rect;
    let stub = streets[court.stub].rect;
    let bulb = streets[court.bulb].rect;
    let stub_idx = court.stub;
    let bulb_idx = court.bulb;
    let per_side = match density {
        Density::Low => 1 + rng.range_usize(2),
        Density::Medium => 3,
        Density::High => 4 + rng.range_usize(2),
    };
    let north_strip = Rect::new(block.x, block.y, block.w, bulb.y - block.y);
    lay_strip(rng, lots, houses, north_strip, per_side, 0, bulb_idx);
    let mid_splits = if density == Density::High && bulb.d >= 26.0 { 2 } else { 1 };
    let sub_d = bulb.d / mid_splits as f64;
    for k in 0..mid_splits {
        let by = bulb.y + k as f64 * sub_d;
        let west_mid = Rect::new(block.x, by, bulb.x - block.x, sub_d);
        place_house(rng, lots, houses, west_mid, bulb_idx, 90, true);
        let east_mid = Rect::new(bulb.right(), by, block.right() - bulb.right(), sub_d);
        place_house(rng, lots, houses, east_mid, bulb_idx, 270, true);
    }
    let south_west = Rect::new(block.x, bulb.bottom(), stub.x - block.x, block.bottom() - bulb.bottom());
    place_house(rng, lots, houses, south_west, stub_idx, 90, true);
    let south_east = Rect::new(stub.right(), bulb.bottom(), block.right() - stub.right(), block.bottom() - bulb.bottom());
    place_house(rng, lots, houses, south_east, stub_idx, 270, true);
}

/// Lays one strip of lots across a rectangle, each facing the street.
fn lay_strip(
    rng: &mut Rng,
    lots: &mut Vec<Lot>,
    houses: &mut Vec<PlacedHouse>,
    strip: Rect,
    per_side: usize,
    rotation: u16,
    street: usize,
) {
    let lot_w = strip.w / per_side as f64;
    for k in 0..per_side {
        let lot = Rect::new(strip.x + k as f64 * lot_w, strip.y, lot_w, strip.d);
        place_house(rng, lots, houses, lot, street, rotation, false);
    }
}

/// Generates one house for a lot and places it facing its street.
fn place_house(
    rng: &mut Rng,
    lots: &mut Vec<Lot>,
    houses: &mut Vec<PlacedHouse>,
    lot: Rect,
    street: usize,
    rotation: u16,
    small: bool,
) {
    let street_span = if matches!(rotation, 0 | 180) { lot.w } else { lot.d };
    let style = style_for_lot(rng, street_span, small);
    let house = generate_house(rng.next_u32(), style);
    let extent = house_extent(&house);
    let (pw, pd) = match rotation {
        0 | 180 => (extent.w, extent.d),
        _ => (extent.d, extent.w),
    };
    let (px, py) = match rotation {
        0 => (lot.x + (lot.w - pw) / 2.0, lot.y + lot.d - FRONT_SETBACK - pd),
        180 => (lot.x + (lot.w - pw) / 2.0, lot.y + FRONT_SETBACK),
        90 => (lot.x + lot.w - FRONT_SETBACK - pw, lot.y + (lot.d - pd) / 2.0),
        _ => (lot.x + FRONT_SETBACK, lot.y + (lot.d - pd) / 2.0),
    };
    lots.push(Lot { rect: lot, street });
    let rect = Rect::new(px, py, pw, pd);
    houses.push(PlacedHouse { house, lot: lots.len() - 1, rotation, rect });
}

/// Narrow lots (< 15 m) get Townhouse; medium/side lots get
/// Cottage/Townhouse; wide lots get the larger styles.
fn style_for_lot(rng: &mut Rng, street_span: f64, small: bool) -> Style {
    if street_span < 15.0 {
        Style::Townhouse
    } else if small || street_span < 40.0 {
        *rng.pick(&[Style::Cottage, Style::Townhouse])
    } else {
        *rng.pick(&[Style::Ranch, Style::Colonial, Style::Modern, Style::Craftsman])
    }
}
