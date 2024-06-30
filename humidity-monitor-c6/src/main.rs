#![no_std]
#![no_main]

use bleps::{gatt, no_rng::NoRng, Ble, HciConnector};
use core::{cell::RefCell, time::Duration};
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcCalCurve, AdcConfig, Attenuation},
    clock::{ClockControl, CpuClock},
    delay::Delay,
    gpio::{Io, Level, Output},
    peripherals::*,
    prelude::*,
    rng::Rng,
    rtc_cntl::{sleep::TimerWakeupSource, Rtc},
    system::SystemControl,
    timer::systimer::SystemTimer,
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

mod blessed;

#[ram(rtc_fast)]
static SAMPLE_HISTORY: Mutex<RefCell<Historical<128, Summary<Hygrometer>>>> =
    Mutex::new(RefCell::new(Historical::new()));

const MEASURE_DELAY: u64 = MicrosDurationU64::minutes(15).to_millis();
const HYGROMETER_WARMUP: u32 = MillisDurationU32::millis(1000).to_millis();
const HYGROMETER_SAMPLES: u8 = 255;

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

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut rtc = Rtc::new(peripherals.LPWR, None);

    // Always run `configure`, do not rely on `boot_defaults`, which does
    // not seem to play with deep sleep.
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock80MHz).freeze();
    let delay = &mut Delay::new(&clocks);

    // Pin definitions
    let mut alarm = Output::new(io.pins.gpio15, Level::Low);
    let mut hygrometer_enable = Output::new(io.pins.gpio13, Level::Low);
    let mut hygrometer_adc_config = AdcConfig::new();
    let hygrometer_adc1_pin = &mut hygrometer_adc_config
        .enable_pin_with_cal::<_, AdcCalCurve<ADC1>>(io.pins.gpio3, Attenuation::Attenuation11dB);
    let hygrometer_adc1 = &mut Adc::new(peripherals.ADC1, hygrometer_adc_config);
    //

    for _ in 0..5 {
        delayed_pulse!(alarm, delay, 10, 15);
        delayed_pulse!(alarm, delay, 10, 25);
    }

    let mut toggle = || hygrometer_enable.toggle();
    let mut warmup = || delay.delay_millis(HYGROMETER_WARMUP);
    let mut read_adc = || match hygrometer_adc1.read_oneshot(hygrometer_adc1_pin) {
        Ok(sample) => sample,
        Err(err) => panic!("adc failure: {err:?}"),
    };

    let summary = sample::perform_sampling(
        HYGROMETER_SAMPLES,
        &mut toggle,
        &mut warmup,
        &mut read_adc,
        Hygrometer::HW390,
    );

    critical_section::with(|cs| {
        let mut history = SAMPLE_HISTORY.borrow_ref_mut(cs);
        history.store(summary);
    });

    let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let wifi_init = match esp_wifi::initialize(
        EspWifiInitFor::Ble,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    ) {
        Ok(init) => init,
        Err(err) => panic!("wifi initialization failure: {err:?}"),
    };

    let mut bluetooth = peripherals.BT;
    let connector = BleConnector::new(&wifi_init, &mut bluetooth);
    let hci = HciConnector::new(connector, esp_wifi::current_millis);
    let ble = &mut Ble::new(&hci);

    blessed::start(ble);
    if blessed::wait_for_connection(ble, delay) {
        let mut syncer = critical_section::with(|cs| {
            let history = SAMPLE_HISTORY.borrow_ref(cs);
            history.sync()
        });

        let mut read_historical = |_offset: usize, data: &mut [u8]| match syncer.write(data) {
            Ok(n) => n,
            Err(err) => {
                log::error!("cannot serialize historical data: {err:?}");
                0
            }
        };

        let mut read_last_sample =
            |_offset: usize, data: &mut [u8]| match serde::serialize(&summary, data) {
                Ok(n) => n,
                Err(err) => {
                    log::error!("cannot serialize last sample: {err:?}");
                    0
                }
            };

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
        blessed::work_until_disconnect(ble, &mut gatt_attributes, &mut rng)
    }

    pulse!(alarm, delay, 100);

    let timer = TimerWakeupSource::new(Duration::from_millis(MEASURE_DELAY));
    rtc.sleep_deep(&[&timer], delay);
}
