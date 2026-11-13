//! Use cases: generation. Each step is a small pure function so it
//! can be tested on its own: footprint -> rooms -> programs -> doors
//! -> windows. `generate` wires them together.

pub mod alcove;
pub mod bsp;
pub mod cap_cut;
pub mod caps;
pub mod circulation;
pub mod distribute;
pub mod door_panel;
pub mod extrude;
pub mod floor;
pub mod furnish;
pub mod furnish_clear;
pub mod furniture_mesh;
pub mod furniture_parts;
pub mod ground;
pub mod haunt;
pub mod haunt_events;
pub mod haunt_tasks;
pub mod interact;
pub mod layout;
pub mod lots;
pub mod placement;
pub mod repair;
pub mod repair_window;
pub mod retile;
pub mod doors;
pub mod footprint;
pub mod garage;
pub mod generate;
pub mod neighborhood;
pub mod neighborhood_mesh;
pub mod programs;
pub mod upper;
pub mod variety;
pub mod walls;
pub mod windows;

/// Quantizes a metre value to 0.1 m. Every seeded dimension in the
/// generator goes through this, so plans stay readable and the variety
/// count can be computed from finite decision sets.
pub fn quant(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}
