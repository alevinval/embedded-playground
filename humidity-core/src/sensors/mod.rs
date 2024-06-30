//! # Sensors and calibrations
//!
//! Defines the [`Sensor`] trait for all sensors to implement
//!
//! ## TODO
//! - Introduce a mechanism to override calibrations, specially to support different
//! calibrations based on different environments.
//!
//! ## Supported sensors
//!
//! - Hygrometers
//!   - [`Hygrometer::YL69`]
//!   - [`Hygrometer::HW390`]
//!
//! ## Examples
//!
//! ```rust
//! use humidity_core::sensors::{Hygrometer, Sensor};
//! let sensor = Hygrometer::HW390;
//! println!("sensor reading: {}", sensor.percentage(1200));
//! ```

pub use hygrometer::Hygrometer;

use crate::serde;

mod hygrometer;

/// Defines common behaviour for all sensors, such as getting the calibrated low
/// and high values, and provides a function to compute where a value
/// fits within the calibrated boundaries.
pub trait Sensor: serde::Serializable + serde::Deserializable {
    /// Returns the calibrated low reading.
    fn low(&self) -> u16;
    /// Returns the calibrated high reading.
    fn high(&self) -> u16;
    /// Given a value, returns the percentage it falls within the calibrated boundaries.
    fn percentage(&self, value: u16) -> f32 {
        (value - self.low()) as f32 / (self.high() - self.low()) as f32
    }
}
