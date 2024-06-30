use btleplug::{
    api::{self, Central, Manager as _, Peripheral, ScanFilter},
    platform::{self, Adapter, Manager},
};
use chrono::Local;
use humidity_core::{
    sample::Summary,
    sensors::{Hygrometer, Sensor},
    serde, shared,
};
use std::{error::Error, time::Duration};
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    time::{sleep, timeout, Instant},
};
use uuid::Uuid;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().next().unwrap()
}

async fn find_device(central: &Adapter) -> Option<platform::Peripheral> {
    for p in central.peripherals().await.unwrap() {
        if api::Peripheral::properties(&p)
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| name.contains(shared::BLE_DEVICE_NAME))
        {
            return Some(p);
        }
    }
    None
}

async fn collect_data(device: impl Peripheral) -> Result<(), Box<dyn Error>> {
    let humidity = Uuid::try_parse("987312e0-2354-11eb-9f10-fbc30a62cf50")?;
    let _historical = Uuid::try_parse("987312e0-2354-11eb-9f10-fbc30a62cf60")?;

    println!("connecting to {}", device.id());
    let since_connecting = Instant::now();
    device.connect().await?;
    println!("connected");

    let chars = device.characteristics();
    let cmd_humidity = chars.iter().find(|c| c.uuid == humidity).unwrap();
    let _cmd_historical = chars.iter().find(|c| c.uuid == _historical).unwrap();

    let single_read = device.read(cmd_humidity).await.unwrap();
    device.disconnect().await?;
    println!("disconnected, elapsed: {:?}", since_connecting.elapsed());

    let summary: Summary<Hygrometer> = serde::deserialize(&single_read).unwrap();
    println!("latest sample: {summary:?} dryness: {}", summary.sensor.percentage(summary.avg));

    let mut open = OpenOptions::new();
    let mut output = open.write(true).append(true).open("data.csv").await?;
    let now = Local::now().format("%Y-%m-%dT%H:%M:%SZ");
    output
        .write_all(format!("{now},{},{},{}\n", summary.avg, summary.min, summary.max).as_bytes())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;
    let central = get_central(&manager).await;
    central.start_scan(ScanFilter::default()).await.expect("failed starting scan");

    loop {
        if let Some(device) = find_device(&central).await {
            let collect_future = collect_data(device);
            let _ = timeout(Duration::from_secs(10), collect_future).await;
        }
        sleep(Duration::from_secs(1)).await;
    }
}
