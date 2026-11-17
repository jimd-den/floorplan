//! Domain: plan geometry. Pure data and pure functions, no I/O.
//!
//! Coordinates are metres. x runs east, y runs south (down in SVG).
//! The front of the house is the south edge (largest y).

pub mod base;
pub mod basement;
pub mod chunk;
pub mod curve;
pub mod framing;
pub mod furniture;
pub mod geometry;
pub mod haunt;
pub mod interaction;
pub mod lattice;
pub mod layout;
pub mod light;
pub mod mesh;
pub mod model;
pub mod neighborhood;
pub mod placement;
pub mod polygon;
pub mod segment;
pub mod skeleton;
pub mod spline;
pub mod street_furniture;
pub mod style;
