//! Lot laying: a block becomes lots, each lot gets one house
//! facing its street. The lot sizes are chosen so any generated
//! house fits its lot with setbacks (max house extent 26 x 16 m,
//! proven in domain::neighborhood; the small styles on the narrow
//! lots are narrower still).

use crate::domain::geometry::Rect;
use crate::domain::neighborhood::{Lot, PlacedHouse, Street, house_extent};
use crate::domain::style::Style;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::generate::generate_house;
use crate::use_cases::layout::BlockPlan;
use crate::use_cases::neighborhood::{FRONT_SETBACK, SIDE_LOT_D};
use crate::use_cases::quant;

/// Lays out one block: north/south strips plus east/west lots
/// in the middle band. `streets` is (north, south, west, east).
pub fn lay_block(
    rng: &mut Rng,
    lots: &mut Vec<Lot>,
    houses: &mut Vec<PlacedHouse>,
    block_rect: Rect,
    streets: (usize, usize, usize, usize),
) {
    let (north, south, west, east) = streets;
    let (x0, y0, block) = (block_rect.x, block_rect.y, block_rect.w);
    let lot_d = quant(rng.range_f64(24.0, 28.0));
    let per_side = 1 + rng.range_usize(2);
    lay_strip(rng, lots, houses, Rect::new(x0, y0, block, lot_d), per_side, 180, north);
    lay_strip(rng, lots, houses, Rect::new(x0, y0 + block - lot_d, block, lot_d), per_side, 0, south);
    let mid_top = y0 + lot_d;
    let mid_h = block - 2.0 * lot_d;
    if mid_h >= 15.0 {
        let west_lot = Rect::new(x0, mid_top, SIDE_LOT_D, mid_h);
        place_house(rng, lots, houses, west_lot, west, 270, true);
        let east_lot = Rect::new(x0 + block - SIDE_LOT_D, mid_top, SIDE_LOT_D, mid_h);
        place_house(rng, lots, houses, east_lot, east, 90, true);
    }
}

/// Lays out a cul-de-sac court block: a north strip facing south
/// onto the bulb, a lot on each side of the bulb, and a lot on
/// each side of the stub. The stub and bulb rects come from the
/// plan's streets, and the lots are derived from them, so they
/// always fit around the streets.
pub fn lay_court(
    rng: &mut Rng,
    lots: &mut Vec<Lot>,
    houses: &mut Vec<PlacedHouse>,
    block_plan: &BlockPlan,
    streets: &[Street],
) {
    let Some(court) = &block_plan.court else { return };
    let block = block_plan.rect;
    let stub = streets[court.stub].rect;
    let bulb = streets[court.bulb].rect;
    let stub_idx = court.stub;
    let bulb_idx = court.bulb;
    let per_side = 1 + rng.range_usize(2);
    let north_strip = Rect::new(block.x, block.y, block.w, bulb.y - block.y);
    lay_strip(rng, lots, houses, north_strip, per_side, 0, bulb_idx);
    let west_mid = Rect::new(block.x, bulb.y, bulb.x - block.x, bulb.d);
    place_house(rng, lots, houses, west_mid, bulb_idx, 90, true);
    let east_mid = Rect::new(bulb.right(), bulb.y, block.right() - bulb.right(), bulb.d);
    place_house(rng, lots, houses, east_mid, bulb_idx, 270, true);
    let south_west = Rect::new(block.x, bulb.bottom(), stub.x - block.x, block.bottom() - bulb.bottom());
    place_house(rng, lots, houses, south_west, stub_idx, 90, true);
    let south_east = Rect::new(stub.right(), bulb.bottom(), block.right() - stub.right(), block.bottom() - bulb.bottom());
    place_house(rng, lots, houses, south_east, stub_idx, 270, true);
}

/// Lays one strip of lots across a rectangle, each facing the
/// given street. `per_side` is drawn once by the caller so the
/// two strips of a block share it.
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
/// `small` marks the small east/west lots, which get small styles.
fn place_house(
    rng: &mut Rng,
    lots: &mut Vec<Lot>,
    houses: &mut Vec<PlacedHouse>,
    lot: Rect,
    street: usize,
    rotation: u16,
    small: bool,
) {
    let style = style_for_lot(rng, lot.w, small);
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

/// The style for a lot: small lots get Cottage/Townhouse;
/// wide lots get the larger styles.
fn style_for_lot(rng: &mut Rng, lot_w: f64, small: bool) -> Style {
    if small || lot_w < 40.0 {
        *rng.pick(&[Style::Cottage, Style::Townhouse])
    } else {
        *rng.pick(&[Style::Ranch, Style::Colonial, Style::Modern, Style::Craftsman])
    }
}
