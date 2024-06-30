//! # Sampling results
//!
//! Establish a common ground to work with the results of a sampling operation.
//! Uses [`Summary`] to hold the results of a sampling operation.

pub use summary::Summary;

use crate::sensors;

mod summary;

pub fn perform_sampling<SENSOR: sensors::Sensor>(
    n: u8,
    toggle_sensor: &mut impl FnMut(),
    warmup_delay: &mut impl FnMut(),
    adc_read: &mut impl FnMut() -> u16,
    sensor: SENSOR,
) -> Summary<SENSOR> {
    let mut sum = 0u32;
    let mut min = u16::MAX;
    let mut max = u16::MIN;

    toggle_sensor();
    warmup_delay();

    // Trigger 3 warm-up reads.
    adc_read();
    adc_read();
    adc_read();

    // Proceed with sampling.
    for _ in 0..n {
        let sample = adc_read();
        max = max.max(sample);
        min = min.min(sample);
        sum += sample as u32;
    }
    toggle_sensor();

    let avg = sum.div_ceil(n as u32) as u16;
    Summary::<SENSOR> { n, avg, min, max, sensor }
}
