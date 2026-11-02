//! Shared code for the sound-hole generators.
//!
//! The generators were JavaScript until 2026-11-02. Every shipped SVG was drawn
//! by Node, and .repro checks that each one redraws byte-identically. `jsfmt`
//! is JavaScript's number-to-text rules, which the files need: Rust formats
//! numbers differently. `jsmath` is V8's own sin/cos/atan2/hypot, which they
//! turn out NOT to need -- std gives the same 13 files -- but which makes the
//! port compute exactly what Node did for any settings; see its header.
//! Nothing here lets the compiler reorder or fuse floating-point operations.

pub mod contour;
pub mod jsfmt;
pub mod jsmath;
