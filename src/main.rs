//! BLE Example
//!
//! - starts Bluetooth advertising
//! - offers one service with three characteristics (one is read/write, one is write only, one is read/write/notify)
//! - pressing the boot-button on a dev-board will send a notification if it is subscribed

//% FEATURES: esp-wifi esp-wifi/ble
//% CHIPS: esp32 esp32s3 esp32c2 esp32c3 esp32c6 esp32h2

#![no_std]
#![no_main]

use core::time::Duration;

use bleps::{
    ad_structure::{
        create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
    },
    attribute_server::AttributeServer,
    gatt, Ble, HciConnector,
};
use esp_backtrace as _;
use esp_println as _;
use esp_hal::{
    analog::adc::{Adc, AdcCalLine, AdcChannel, AdcConfig, AdcPin, Attenuation},
    clock::ClockControl,
    delay::Delay,
    gpio::{AnalogPin, AnyOutput, Io, Level, Output},
    peripherals::*,
    prelude::*,
    rng::Rng,
    rtc_cntl::{sleep::TimerWakeupSource, Rtc},
    system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_wifi::{ble::controller::BleConnector, initialize, EspWifiInitFor};
use fugit::{MicrosDurationU32, MicrosDurationU64};

const MEASURE_DELAY: u64 = MicrosDurationU64::minutes(15).to_millis();

const HYGROMETER_WARMUP: u32 = MicrosDurationU32::millis(100).to_millis();
const HYGROMETER_SAMPLES: u8 = 64;

fn get_samples<PIN: AnalogPin + AdcChannel>(
    delay: &Delay,
    enable_sensor: &mut AnyOutput,
    adc1: &mut Adc<ADC1>,
    adcpin: &mut AdcPin<PIN, ADC1, AdcCalLine<ADC1>>,
) -> (f32, u16, u16) {
    let mut sample_sum = 0 as u32;
    let mut sample_min = u16::MAX;
    let mut sample_max = u16::MIN;

    enable_sensor.set_high();
    delay.delay_millis(HYGROMETER_WARMUP);
    for _ in 0..HYGROMETER_SAMPLES {
        let sample = adc1.read_blocking(adcpin);
        sample_max = sample_max.max(sample);
        sample_min = sample_min.min(sample);
        sample_sum += sample as u32;
    }
    enable_sensor.set_low();

    let sample_avg = sample_sum as f32 / HYGROMETER_SAMPLES as f32;
    (sample_avg, sample_min, sample_max)
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

    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    let mut wdt0 = timg0.wdt;
    wdt0.enable();
    wdt0.set_timeout(900000u64.secs());

    // Pin definitions
    let hygrometer_diode_pin = io.pins.gpio5;
    let mut hygrometer_diode = AnyOutput::new(hygrometer_diode_pin, Level::Low);
    let mut alarm = Output::new(io.pins.gpio7, Level::Low);
    alarm.set_drive_strength(esp_hal::gpio::DriveStrength::I40mA);

    let hygrometer_pin = io.pins.gpio6;
    type Cal = esp_hal::analog::adc::AdcCalLine<ADC1>;
    let mut hygrometer_adc_config = AdcConfig::new();
    let mut hygrometer_adc1_pin = hygrometer_adc_config
        .enable_pin_with_cal::<_, Cal>(hygrometer_pin, Attenuation::Attenuation11dB);
    let mut hygrometer_adc1 = Adc::new(peripherals.ADC1, hygrometer_adc_config);
    //

    for _ in 0..5 {
        alarm.set_high();
        delay.delay_millis(10);
        alarm.set_low();
        delay.delay_millis(150);
    }
    let (avg, min, max) = get_samples(
        &delay,
        &mut hygrometer_diode,
        &mut hygrometer_adc1,
        &mut hygrometer_adc1_pin,
    );
    let beeps = match avg as u32 {
        0..=650 => 1,
        651..=800 => 2,
        801..=1100 => 3,
        _ => 10,
    };
    for _ in 0..beeps {
        alarm.set_high();
        delay.delay_millis(10);
        alarm.set_low();
        delay.delay_millis(50);
    }

    let mut buf = [0u8; 256];
    let results = format_no_std::show(&mut buf, format_args!("{avg},{min},{max}")).unwrap();
    println!("results: '{results}'");

    let timer = TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
    delay.delay_millis(1000);
    let init = initialize(
        EspWifiInitFor::Ble,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let mut bluetooth = peripherals.BT;

    loop {
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

        let mut rf = |_offset: usize, data: &mut [u8]| {
            data[0..results.len()].copy_from_slice(results.as_bytes());
            results.len()
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

        let mut rng = bleps::no_rng::NoRng;
        let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut rng);

        for _ in 0..12 {
            srv.do_work().unwrap();
            delay.delay_millis(1000);
        }

        alarm.set_high();
        delay.delay_millis(500);
        alarm.set_low();

        let timer = TimerWakeupSource::new(Duration::from_millis(MEASURE_DELAY));
        delay.delay_millis(1000);
        rtc.sleep_deep(&[&timer], &mut delay);
    }
}
