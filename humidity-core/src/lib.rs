//! # Core library for the hygrometer project.
//!
//! Provides a common set of bare-metal utilities tailored for the soil moisture
//! project, however the plan is that this crate becomes a general toolbox with
//! useful tooling that can be re-used across a variety of embedded projects
//! without having to re-invent the wheel each time.
//!
//! This package purposedly does not depend on any other package.
//!
#![no_std]

pub mod historical;
pub mod sample;
pub mod sensors;
pub mod serde;
