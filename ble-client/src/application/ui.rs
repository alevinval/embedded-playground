use crate::infrastructure::ble::Device;

pub trait ListDevicesUI {
    fn render(&mut self, devices: &[Device]) -> Result<(), Box<dyn std::error::Error>>;
}
