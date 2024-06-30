#![no_std]
#![no_main]

use bleps::{gatt, no_rng::NoRng, Ble, HciConnector};
use core::time::Duration;
use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcCalLine, AdcConfig, Attenuation},
    clock::ClockControl,
    delay::Delay,
    gpio::{DriveStrength, Io, Level, Output},
    peripherals::*,
    prelude::*,
    rng::Rng,
    rtc_cntl::{sleep::TimerWakeupSource, Rtc},
    system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_println as _;
use esp_wifi::{self, ble::controller::BleConnector, EspWifiInitFor};
use fugit::{MicrosDurationU64, MillisDurationU32};
use humidity_core::{
    historical::Historical,
    sample::{self, Summary},
    sensors::Hygrometer,
    serde,
};

#[ram(rtc_fast)]
static mut SAMPLE_HISTORY: Historical<128, Summary<Hygrometer>> = Historical::new();

const MEASURE_DELAY: u64 = MicrosDurationU64::minutes(15).to_millis();
const HYGROMETER_WARMUP: u32 = MillisDurationU32::millis(1000).to_millis();
const HYGROMETER_SAMPLES: u8 = u8::MAX;

macro_rules! pulse {
    ($output:ident, $delay:ident, $ms:expr) => {{
        $output.set_high();
        $delay.delay_millis($ms);
        $output.set_low();
    }};
}

macro_rules! delayed_pulse {
    ($output:ident, $delay:ident, $ms:expr, $delay_ms:expr) => {{
        $delay.delay_millis($delay_ms);
        pulse!($output, $delay, $ms);
    }};
}

mod blessed;

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut rtc = Rtc::new(peripherals.LPWR, None);
    let mut delay = Delay::new(&clocks);

    // Pin definitions
    let hygrometer_enable = &mut Output::new(io.pins.gpio4, Level::Low);
    hygrometer_enable.set_drive_strength(DriveStrength::I5mA);
    let alarm = &mut Output::new(io.pins.gpio6, Level::Low);
    alarm.set_drive_strength(esp_hal::gpio::DriveStrength::I5mA);

    let mut hygrometer_adc_config = AdcConfig::new();
    let hygrometer_adc1_pin = &mut hygrometer_adc_config
        .enable_pin_with_cal::<_, AdcCalLine<ADC1>>(io.pins.gpio5, Attenuation::Attenuation11dB);
    let hygrometer_adc1 = &mut Adc::new(peripherals.ADC1, hygrometer_adc_config);
    //

    for _ in 0..5 {
        delayed_pulse!(alarm, delay, 10, 15);
        delayed_pulse!(alarm, delay, 10, 25);
    }

    let mut toggle = || hygrometer_enable.toggle();
    let mut warmup = || delay.delay_millis(HYGROMETER_WARMUP);
    let mut read_adc = || hygrometer_adc1.read_oneshot(hygrometer_adc1_pin).unwrap();
    let summary = sample::perform_sampling(
        HYGROMETER_SAMPLES,
        &mut toggle,
        &mut warmup,
        &mut read_adc,
        Hygrometer::HW390,
    );

    unsafe { SAMPLE_HISTORY.store(summary) };

    let timer = TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
    let init = esp_wifi::initialize(
        EspWifiInitFor::Ble,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let mut hsync = unsafe { SAMPLE_HISTORY.sync() };
    let mut read_historical = |_offset: usize, data: &mut [u8]| hsync.write(data).unwrap();

    let mut bluetooth = peripherals.BT;
    let connector = BleConnector::new(&init, &mut bluetooth);
    let hci = HciConnector::new(connector, esp_wifi::current_millis);
    let ble = &mut Ble::new(&hci);

    blessed::start(ble);
    if blessed::wait_for_connection(ble, &mut delay) {
        let mut read_last_sample =
            |_offset: usize, data: &mut [u8]| serde::serialize(&summary, data).unwrap();

        gatt!([service {
            uuid: "937312e0-2354-11eb-9f10-fbc30a62cf00",
            characteristics: [
                characteristic {
                    name: "humidity",
                    uuid: "987312e0-2354-11eb-9f10-fbc30a62cf50",
                    read: read_last_sample,
                },
                characteristic {
                    name: "historical",
                    uuid: "987312e0-2354-11eb-9f10-fbc30a62cf60",
                    read: read_historical,
                },
            ]
        },]);

        let mut rng = NoRng;
        blessed::work_until_disconnect(ble, &mut gatt_attributes, &mut rng);
    }

    pulse!(alarm, delay, 100);

    let timer = TimerWakeupSource::new(Duration::from_millis(MEASURE_DELAY));
    rtc.sleep_deep(&[&timer], &mut delay);
}
