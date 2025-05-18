use crate::infrastructure::ble::BLE;

use super::ui;

pub async fn list_devices<'data>(
    ble: &BLE,
    presenter: &mut impl ui::ListDevicesUI,
) -> Result<(), Box<dyn std::error::Error>> {
    let devices = ble.get_devices().await;
    presenter.render(&devices)
}
