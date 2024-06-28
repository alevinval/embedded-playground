//! # Sampling results
//!
//! Establish a common ground to work with the results of a sampling operation.
//! Uses [`Summary`] to hold the results of a sampling operation.
//!
//! ## TODO
//!
//! - For the moment, only works with [`Hygrometer`] as sensor, should be
//! reworked to work with any [`crate::sensors::Sensor`].

pub use result::Summary;

use crate::sensors::Hygrometer;

mod result;

pub fn perform_sampling(
    n: u8,
    toggle_sensor: &mut impl FnMut() -> (),
    warmup_delay: &mut impl FnMut() -> (),
    adc_read: &mut impl FnMut() -> u16,
    hygrometer: Hygrometer,
) -> Summary {
    let mut sum = 0u32;
    let mut min = u16::MAX;
    let mut max = u16::MIN;

    toggle_sensor();
    warmup_delay();
    for _ in 0..n {
        let sample = adc_read();
        max = max.max(sample);
        min = min.min(sample);
        sum += sample as u32;
    }
    toggle_sensor();

    let avg = (sum as f32 / n as f32) as u16;
    Summary { n, avg, min, max, hygrometer }
}
