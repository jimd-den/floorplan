//! chunk: one window onto the unbounded neighborhood.
//!
//! A chunk is one lattice cell. Its content is derived only from the
//! world seed and its coordinates: the streets near the cell, the lots
//! whose front centre falls inside it, and one house per lot. There is
//! no block count and no fixed size — a 2x2 or 3x3 neighborhood is
//! simply four or nine windows, and the window can be moved anywhere in
//! `i32` space. Nothing here reads visit order, wall-clock time, or
//! player state, so a revisit rebuilds byte-identical content.

use crate::domain::chunk::{Chunk, ChunkState, DEFAULT_CHUNK_BLOCKS};
use crate::domain::curve::Pt;
use crate::domain::geometry::Rect;
use crate::domain::lattice::{LATTICE, Node};
use crate::domain::layout::Density;
use crate::domain::neighborhood::{Lot, Neighborhood, Street};
use crate::use_cases::road_signs::{EndKind, StreetEnds, signs_along};
use crate::use_cases::street_lights::lamps_along;
use crate::use_cases::world_lots::{inside, lay_streets, street_seed};
use crate::use_cases::world_streets::{node_degree, streets_in};

/// The world-space spacing between chunk origins, in metres: one
/// lattice cell. Adjacent chunks share the streets on their border.
pub const CHUNK_SPAN: f64 = LATTICE;

/// The seed a chunk generates from. Chunk (0,0) keeps the world seed
/// so the origin's neighborhood is the canonical one; every other
/// chunk mixes the seed with its coordinates.
pub fn chunk_seed(world: u32, cx: i32, cz: i32) -> u32 {
    if cx == 0 && cz == 0 {
        return world;
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in world
        .to_le_bytes()
        .iter()
        .chain(cx.to_le_bytes().iter())
        .chain(cz.to_le_bytes().iter())
    {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    (hash >> 32) as u32
}

/// The world-space origin of a chunk, in metres.
pub fn chunk_origin(cx: i32, cz: i32) -> (f64, f64) {
    (cx as f64 * CHUNK_SPAN, cz as f64 * CHUNK_SPAN)
}

/// The half-open rectangle a chunk owns. Every point of the plane
/// belongs to exactly one chunk, which is how lots are partitioned.
pub fn chunk_rect(cx: i32, cz: i32) -> Rect {
    let (x, y) = chunk_origin(cx, cz);
    Rect::new(x, y, CHUNK_SPAN, CHUNK_SPAN)
}

/// Builds a finished chunk. Deterministic in the world seed and the
/// coordinates, and independent of every other chunk.
pub fn build_chunk(world: u32, cx: i32, cz: i32) -> Chunk {
    let seed = chunk_seed(world, cx, cz);
    let rect = chunk_rect(cx, cz);
    let density = Density::from_seed(seed);
    let streets = streets_in(world, rect);
    let (lots, houses) = keep_owned(&streets, rect, density);
    let (lamps, signs) = keep_furniture(world, &streets, rect);
    Chunk {
        cx,
        cz,
        state: ChunkState::Ready,
        neighborhood: Some(Neighborhood {
            seed,
            blocks: DEFAULT_CHUNK_BLOCKS,
            streets,
            lots,
            houses,
            lamps,
            signs,
        }),
    }
}

/// The lamps and signs whose base falls in `rect`. Like lots, they
/// are placed from the street alone, so a chunk that can see the
/// street sees the same furniture; the half-open rect decides which
/// chunk keeps it.
fn keep_furniture(
    world: u32,
    streets: &[Street],
    rect: Rect,
) -> (
    Vec<crate::domain::street_furniture::StreetLamp>,
    Vec<crate::domain::street_furniture::RoadSign>,
) {
    let mut lamps = Vec::new();
    let mut signs = Vec::new();
    for street in streets {
        lamps.extend(lamps_along(street).into_iter().filter(|l| inside(rect, l.position)));
        let ends = ends_of(world, street);
        signs.extend(
            signs_along(street, street_seed(street), ends)
                .into_iter()
                .filter(|g| inside(rect, g.position)),
        );
    }
    (lamps, signs)
}

/// What a street's two ends are, from the lattice: a junction when
/// three or more kept streets meet the node, a dead end otherwise.
fn ends_of(seed: u32, street: &Street) -> StreetEnds {
    let last = *street.control.last().unwrap_or(&Pt::new(0.0, 0.0));
    StreetEnds { start: end_kind(seed, street.control[0]), end: end_kind(seed, last) }
}

fn end_kind(seed: u32, p: Pt) -> EndKind {
    let node = Node::new((p.x / LATTICE).round() as i32, (p.y / LATTICE).round() as i32);
    if node_degree(seed, node) >= 3 { EndKind::Junction } else { EndKind::DeadEnd }
}

/// The lots whose front centre falls in `rect`, with their houses
/// re-indexed onto the kept list. A lot is laid the same way in every
/// chunk that can see its street; only the owner keeps it.
///
/// The whole visible set is laid first, because the cross-street
/// conflict rule (`lay_streets`) has to see a lot that reaches into
/// the chunk even when its front centre is outside it. That rule is
/// keyed on street geometry alone, so every chunk that can see these
/// streets drops exactly the same lots.
fn keep_owned(
    streets: &[Street],
    rect: Rect,
    density: Density,
) -> (Vec<Lot>, Vec<crate::domain::neighborhood::PlacedHouse>) {
    let (laid, laid_houses) = lay_streets(streets, density);
    let mut lots: Vec<Lot> = Vec::new();
    let mut houses = Vec::new();
    for (lot, mut house) in laid.into_iter().zip(laid_houses) {
        if !inside(rect, front_centre(&streets[lot.street], &lot)) {
            continue;
        }
        house.lot = lots.len();
        lots.push(lot);
        houses.push(house);
    }
    (lots, houses)
}

/// The front centre of a lot: the point on its street's offset curve at
/// the lot's centre arc length. Unique per lot, so exactly one chunk
/// owns it.
pub fn front_centre(street: &Street, lot: &Lot) -> Pt {
    street
        .samples
        .point_at(lot.center_s)
        .add(street.samples.normal_at(lot.center_s).scale(4.0 * lot.side as f64))
}
