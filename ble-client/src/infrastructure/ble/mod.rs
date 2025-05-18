use std::error::Error;

use btleplug::{
    api::{self, Central, Manager as _, Peripheral, ScanFilter},
    platform::{self, Adapter, Manager},
};
use futures::future;

pub struct BLE {
    central: Adapter,
}

#[derive(Clone)]
pub struct Device {
    pub id: String,
    pub name: String,

    peripheral: platform::Peripheral,
}

impl Device {
    async fn new(peripheral: &platform::Peripheral) -> Self {
        let id = peripheral.id().to_string();
        let name = api::Peripheral::properties(peripheral)
            .await
            .ok()
            .flatten()
            .map(|props| props.local_name)
            .flatten()
            .unwrap_or_default();

        Device { id, name, peripheral: peripheral.clone() }
    }

    pub fn is_named(&self) -> bool {
        !self.name.is_empty()
    }
}

impl BLE {
    pub async fn new() -> Self {
        let manager = Manager::new().await.unwrap();
        let adapters = manager.adapters().await.unwrap();
        let central = adapters.into_iter().next().unwrap();
        Self { central }
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        self.central.start_scan(ScanFilter::default()).await?;
        Ok(())
    }

    pub async fn get_devices(&self) -> Vec<Device> {
        let peripherals = self.central.peripherals().await.unwrap();
        let devices_futures = peripherals.iter().map(Device::new);
        future::join_all(devices_futures).await
    }
}
