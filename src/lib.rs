//! Shared code for the sound-hole generators.
//!
//! The generators were JavaScript until 2026-11-02. Every shipped SVG was drawn
//! by Node, and .repro checks that each one redraws byte-identically, so the
//! port keeps Node's arithmetic exactly: `jsmath` is V8's own sin/cos/atan2/
//! hypot, `jsfmt` is JavaScript's number-to-text rules, and nothing here lets
//! the compiler reorder or fuse floating-point operations.

pub mod contour;
pub mod jsfmt;
pub mod jsmath;
