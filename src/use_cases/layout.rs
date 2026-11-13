//! Street/block planning: the street network and the blocks for
//! a layout family. The planner returns streets and, per block,
//! the adjacent street indices, so the lot layer can lay houses
//! without knowing which family built the network. Grid consumes
//! no RNG draws, so a Grid neighborhood is byte-identical to the
//! original grid builder.

use crate::domain::geometry::Rect;
use crate::domain::layout::Layout;
use crate::domain::neighborhood::Street;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::neighborhood::STREET_W;

/// One block's place in the street network.
pub struct BlockPlan {
    pub rect: Rect,
    /// Adjacent street indices: north, south, west, east.
    pub north: usize,
    pub south: usize,
    pub west: usize,
    pub east: usize,
    /// A cul-de-sac court inside the block, if any.
    pub court: Option<CourtPlan>,
}

/// A cul-de-sac court: the stub street index and the bulb street
/// index in the plan's street list.
pub struct CourtPlan {
    pub stub: usize,
    pub bulb: usize,
}

/// The planned street network and blocks for one neighborhood.
pub struct LayoutPlan {
    pub streets: Vec<Street>,
    pub blocks: Vec<BlockPlan>,
}

/// Plans the street network and blocks. `block` is the uniform
/// block side in metres; `blocks` is (columns, rows).
pub fn plan_layout(rng: &mut Rng, layout: Layout, blocks: (u32, u32), block: f64) -> LayoutPlan {
    match layout {
        Layout::Grid => plan_grid(blocks, block),
        Layout::Offset => plan_offset(blocks, block),
        Layout::CulDeSac => plan_cul_de_sac(rng, blocks, block),
    }
}

fn plan_grid(blocks: (u32, u32), block: f64) -> LayoutPlan {
    let (bx, by) = (blocks.0 as f64, blocks.1 as f64);
    let total_w = (bx + 1.0) * STREET_W + bx * block;
    let total_h = (by + 1.0) * STREET_W + by * block;
    let mut streets = Vec::new();
    for k in 0..=blocks.0 {
        let r = Rect::new(k as f64 * (block + STREET_W), 0.0, STREET_W, total_h);
        streets.push(Street { rect: r, horizontal: false });
    }
    for k in 0..=blocks.1 {
        let r = Rect::new(0.0, k as f64 * (block + STREET_W), total_w, STREET_W);
        streets.push(Street { rect: r, horizontal: true });
    }
    let mut plan_blocks = Vec::new();
    for j in 0..blocks.1 {
        for i in 0..blocks.0 {
            let x0 = STREET_W + i as f64 * (block + STREET_W);
            let y0 = STREET_W + j as f64 * (block + STREET_W);
            plan_blocks.push(BlockPlan {
                rect: Rect::new(x0, y0, block, block),
                north: (blocks.0 + 1 + j) as usize,
                south: (blocks.0 + 2 + j) as usize,
                west: i as usize,
                east: (i + 1) as usize,
                court: None,
            });
        }
    }
    LayoutPlan { streets, blocks: plan_blocks }
}

/// Offset grid: alternate block rows shift east by half a block,
/// so the vertical streets jog at every horizontal street. The
/// horizontal streets stay full-width; the vertical streets are
/// per-row segments, and the odd rows extend half a block further
/// east.
fn plan_offset(blocks: (u32, u32), block: f64) -> LayoutPlan {
    let (cols, rows) = blocks;
    let shift = block / 2.0;
    let total_w = (cols as f64 + 1.0) * STREET_W
        + cols as f64 * block
        + if rows > 1 { shift } else { 0.0 };
    let mut streets = Vec::new();
    for j in 0..rows {
        let s = if j % 2 == 1 { shift } else { 0.0 };
        for k in 0..=cols {
            let r = Rect::new(
                s + k as f64 * (block + STREET_W),
                j as f64 * (block + STREET_W),
                STREET_W,
                block + STREET_W,
            );
            streets.push(Street { rect: r, horizontal: false });
        }
    }
    for k in 0..=rows {
        let r = Rect::new(0.0, k as f64 * (block + STREET_W), total_w, STREET_W);
        streets.push(Street { rect: r, horizontal: true });
    }
    let h0 = (rows * (cols + 1)) as usize;
    let mut plan_blocks = Vec::new();
    for j in 0..rows {
        let s = if j % 2 == 1 { shift } else { 0.0 };
        for i in 0..cols {
            let x0 = s + STREET_W + i as f64 * (block + STREET_W);
            let y0 = STREET_W + j as f64 * (block + STREET_W);
            plan_blocks.push(BlockPlan {
                rect: Rect::new(x0, y0, block, block),
                north: h0 + j as usize,
                south: h0 + j as usize + 1,
                west: (j * (cols + 1) + i) as usize,
                east: (j * (cols + 1) + i + 1) as usize,
                court: None,
            });
        }
    }
    LayoutPlan { streets, blocks: plan_blocks }
}

/// Cul-de-sac: a regular grid in which one or two blocks become
/// courts — a stub street enters the block from its south street
/// and ends in a 28 m square bulb. The stub and bulb are streets
/// in their own right, appended after the grid streets.
fn plan_cul_de_sac(rng: &mut Rng, blocks: (u32, u32), block: f64) -> LayoutPlan {
    let mut plan = plan_grid(blocks, block);
    let total = (blocks.0 * blocks.1) as usize;
    let n_courts = (1 + rng.range_usize(2)).min(total);
    let mut indices: Vec<usize> = (0..total).collect();
    rng.shuffle(&mut indices);
    for &bi in indices.iter().take(n_courts) {
        let b = &plan.blocks[bi];
        let (x0, y0) = (b.rect.x, b.rect.y);
        let half = block / 2.0;
        let stub = Rect::new(x0 + half - STREET_W / 2.0, y0 + half + 14.0, STREET_W, half - 14.0);
        let bulb = Rect::new(x0 + half - 14.0, y0 + half - 14.0, 28.0, 28.0);
        let stub_idx = plan.streets.len();
        plan.streets.push(Street { rect: stub, horizontal: false });
        plan.streets.push(Street { rect: bulb, horizontal: true });
        plan.blocks[bi].court = Some(CourtPlan { stub: stub_idx, bulb: stub_idx + 1 });
    }
    plan
}
