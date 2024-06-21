#![no_std]
#![no_main]

use core::{convert::Infallible, time::Duration};

use bleps::{
    ad_structure::{
        create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
    },
    attribute_server::AttributeServer,
    gatt,
    no_rng::NoRng,
    Ble, HciConnector,
};
use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcCalLine, AdcChannel, AdcConfig, AdcPin, Attenuation},
    clock::ClockControl,
    delay::Delay,
    gpio::{AnalogPin, DriveStrength, Io, Level, Output},
    peripherals::*,
    prelude::*,
    rng::Rng,
    rtc_cntl::{sleep::TimerWakeupSource, Rtc},
    system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_wifi::{self, ble::controller::BleConnector, EspWifiInitFor};
use fugit::{MicrosDurationU32, MicrosDurationU64};
use humidity_core::sample::{self, SampleResult};
use toolbox::format;

const MEASURE_DELAY: u64 = MicrosDurationU64::minutes(15).to_millis();
const HYGROMETER_WARMUP: u32 = MicrosDurationU32::millis(100).to_millis();
const HYGROMETER_SAMPLES: u8 = 64;

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

fn get_samples<PIN: AnalogPin + AdcChannel>(
    delay: &Delay,
    mut enable_sensor: impl embedded_hal::digital::v2::OutputPin<Error = Infallible>,
    adc1: &mut Adc<ADC1>,
    adcpin: &mut AdcPin<PIN, ADC1, AdcCalLine<ADC1>>,
) -> SampleResult {
    let mut sample_sum = 0 as u32;
    let mut sample_min = u16::MAX;
    let mut sample_max = u16::MIN;

    enable_sensor.set_high().unwrap();
    delay.delay_millis(HYGROMETER_WARMUP);
    for _ in 0..HYGROMETER_SAMPLES {
        let sample = adc1.read_blocking(adcpin);
        sample_max = sample_max.max(sample);
        sample_min = sample_min.min(sample);
        sample_sum += sample as u32;
    }
    enable_sensor.set_low().unwrap();

    let sample_avg = (sample_sum as f32 / HYGROMETER_SAMPLES as f32) as u16;
    SampleResult { avg: sample_avg, min: sample_min, max: sample_max }
}

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
    let hygrometer_diode_pin = io.pins.gpio5;
    let mut hygrometer_enable = Output::new(hygrometer_diode_pin, Level::Low);
    hygrometer_enable.set_drive_strength(DriveStrength::I5mA);
    let mut alarm = Output::new(io.pins.gpio7, Level::Low);
    alarm.set_drive_strength(esp_hal::gpio::DriveStrength::I5mA);

    let hygrometer_pin = io.pins.gpio6;
    let mut hygrometer_adc_config = AdcConfig::new();
    let mut hygrometer_adc1_pin = hygrometer_adc_config
        .enable_pin_with_cal::<_, AdcCalLine<ADC1>>(hygrometer_pin, Attenuation::Attenuation11dB);
    let mut hygrometer_adc1 = Adc::new(peripherals.ADC1, hygrometer_adc_config);
    //

    for _ in 0..5 {
        delayed_pulse!(alarm, delay, 10, 15);
        delayed_pulse!(alarm, delay, 10, 25);
    }

    let sample_result =
        get_samples(&delay, hygrometer_enable, &mut hygrometer_adc1, &mut hygrometer_adc1_pin);

    let beeps = match sample_result.avg {
        0..=650 => 1,
        651..=800 => 2,
        801..=1100 => 3,
        _ => 10,
    };

    for _ in 0..beeps {
        delayed_pulse!(alarm, delay, 10, 150);
    }

    let mut buf = [0u8; 256];
    let results = format!(buf, "{},{},{}", sample_result.avg, sample_result.min, sample_result.max);
    println!("results: '{results}'");

    let timer = TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
    let init = esp_wifi::initialize(
        EspWifiInitFor::Ble,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let mut bluetooth = peripherals.BT;
    let connector = BleConnector::new(&init, &mut bluetooth);
    let hci = HciConnector::new(connector, esp_wifi::current_millis);
    let mut ble = Ble::new(&hci);

    println!("{:?}", ble.init());
    println!("{:?}", ble.cmd_set_le_advertising_parameters());
    println!(
        "{:?}",
        ble.cmd_set_le_advertising_data(
            create_advertising_data(&[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
                AdStructure::CompleteLocalName(esp_hal::chip!()),
            ])
            .unwrap()
        )
    );
    println!("{:?}", ble.cmd_set_le_advertise_enable(true));

    let mut rf = |_offset: usize, mut data: &mut [u8]| {
        let mut ser = sample::ser::Serializer::default();
        ser.serialize(&sample_result, &mut data).unwrap()
    };

    gatt!([service {
        uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
        characteristics: [characteristic {
            name: "humidity",
            uuid: "987312e0-2354-11eb-9f10-fbc30a62cf38",
            notify: true,
            read: rf,
        },],
    },]);

    let mut rng = NoRng;
    let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut rng);

    for _ in 0..12 {
        srv.do_work().unwrap();
        delay.delay_millis(1000);
    }

    pulse!(alarm, delay, 100);

    let timer = TimerWakeupSource::new(Duration::from_millis(MEASURE_DELAY));
    rtc.sleep_deep(&[&timer], &mut delay);
}
