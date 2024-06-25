use bleps::{
    ad_structure::{
        create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
    },
    att::Uuid,
    attribute::Attribute,
    attribute_server::{AttributeServer, WorkResult},
    event::EventType,
    no_rng::NoRng,
    Ble, PollResult,
};
use esp_hal::delay::Delay;
use esp_println::println;
use humidity_core::shared;

pub fn start(ble: &mut Ble) {
    println!("{:?}", ble.init());
    println!("{:?}", ble.cmd_set_le_advertising_parameters());
    println!(
        "{:?}",
        ble.cmd_set_le_advertising_data(
            create_advertising_data(&[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
                AdStructure::CompleteLocalName(shared::BLE_DEVICE_NAME),
            ])
            .unwrap()
        )
    );
    println!("{:?}", ble.cmd_set_le_advertise_enable(true));
}

pub fn wait_for_connection(ble: &mut Ble, delay: &mut Delay) -> bool {
    let mut connected = false;
    for _ in 0..50 {
        if let Some(result) = ble.poll() {
            match result {
                PollResult::Event(evt) => {
                    if let EventType::ConnectionComplete { .. } = evt {
                        connected = true;
                        break;
                    }
                }
                PollResult::AsyncData(_) => {}
            }
        }
        delay.delay_millis(100);
    }
    println!("{:?}", ble.cmd_set_le_advertise_enable(false));

    connected
}

pub fn work_until_disconnect<'a>(
    ble: &'a mut Ble<'a>,
    gatt_attributes: &'a mut [Attribute<'a>],
    rng: &'a mut NoRng,
) {
    let mut srv = AttributeServer::new(ble, gatt_attributes, rng);
    while let WorkResult::DidWork = srv.do_work().unwrap() {}
}
