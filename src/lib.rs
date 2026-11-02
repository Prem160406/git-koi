//! Draw pixel-art characters from a GitHub account's public activity.
//!
//! The original koi pipeline remains intact. The Wukong prototype adds a separate,
//! deterministic evolution model that maps lifetime contribution totals to five forms.
//! Rendering and recent-activity behaviour will be integrated in subsequent steps.
//!
//! Everything after collection is deterministic: it reads no clock, performs no I/O, and
//! mutates no input. Callers own files and publication. Breaking changes to this API bump
//! the major version.
#![warn(missing_docs)]

pub mod behavior;
pub mod coat;
pub mod collect;
pub mod history;
pub mod koi;
pub mod markings;
pub mod motion;
pub mod output;
pub mod render;
pub mod wukong;
