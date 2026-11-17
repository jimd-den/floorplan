//! Use cases: generation. Each step is a small pure function so it
//! can be tested on its own: footprint -> rooms -> programs -> doors
//! -> windows. `generate` wires them together.

pub mod alcove;
pub mod basement_mesh;
pub mod basement_pose;
pub mod bsp;
pub mod cap_cut;
pub mod caps;
pub mod chunk;
pub mod circulation;
pub mod distribute;
pub mod door_panel;
pub mod extrude;
pub mod floor;
pub mod framing;
pub mod framing_mesh;
pub mod furnish_clear;
pub mod ground;
pub mod haunt;
pub mod haunt_events;
pub mod haunt_tasks;
pub mod interact;
pub mod light;
pub mod light_dread;
pub mod lining;
pub mod lot_geometry;
pub mod lots;
pub mod materials;
pub mod place_house;
pub mod place_mesh;
pub mod placement;
pub mod relief;
pub mod repair;
pub mod repair_window;
pub mod retile;
pub mod road_signs;
pub mod ribbon;
pub mod roof;
pub mod roof_detail;
pub mod roof_frame;
pub mod scene;
pub mod skeleton;
pub mod stairs;
pub mod stream;
pub mod street_furniture_mesh;
pub mod street_lights;
pub mod street_network;
pub mod traverse;
pub mod doors;
pub mod door_width;
pub mod footprint;
pub mod foundation;
pub mod garage;
pub mod generate;
pub mod grounds;
pub mod neighborhood;
pub mod neighborhood_mesh;
pub mod porch;
pub mod programs;
pub mod trim;
pub mod upper;
pub mod variety;
pub mod variety_bounds;
pub mod viewpoint;
pub mod walkway;
pub mod world_lots;
pub mod world_streets;
pub mod wall_block;
pub mod walls;
pub mod window_clear;
pub mod windows;

/// Quantizes a metre value to 0.1 m. Every seeded dimension in the
/// generator goes through this, so plans stay readable and the variety
/// count can be computed from finite decision sets. Defined in the
/// domain (`domain::geometry`); re-exported here for the use cases.
pub use crate::domain::geometry::quant;
