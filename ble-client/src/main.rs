use btleplug::{
    api::{self, Central, Manager as _, Peripheral, ScanFilter},
    platform::{self, Adapter, Manager},
};
use chrono::Local;
use humidity_core::sample;
use std::{error::Error, time::Duration};
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    time::{sleep, Instant},
};
use uuid::Uuid;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().next().unwrap()
}

async fn find_esp32(central: &Adapter) -> Option<platform::Peripheral> {
    for p in central.peripherals().await.unwrap() {
        if api::Peripheral::properties(&p)
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| name.contains("esp32s3"))
        {
            return Some(p);
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let humidity = Uuid::parse_str("987312e0-2354-11eb-9f10-fbc30a62cf38").unwrap();
    let manager = Manager::new().await?;
    let central = get_central(&manager).await;
    central.start_scan(ScanFilter::default()).await.expect("failed starting scan");

    loop {
        let device = find_esp32(&central).await;
        if let Some(esp32) = device {
            let d = Instant::now();
            println!("connecting to esp32s3");
            esp32.connect().await?;
            println!("connected to esp32s3");

            esp32.discover_services().await?;
            let chars = esp32.characteristics();
            let cmd_humidity = chars.iter().find(|c| c.uuid == humidity).unwrap();
            let data = esp32.read(cmd_humidity).await.unwrap();

            let mut de = sample::de::Deserializer::default();
            let sample = de.deserialize(&data).unwrap();
            println!("sample: {sample:?}");
            println!("elapsed {:?}", d.elapsed());

            let mut open = OpenOptions::new();
            let mut output = open.write(true).append(true).open("data.csv").await?;
            let now = Local::now().format("%Y-%m-%d %H:%M:%s");
            output
                .write_all(
                    format!("{now},{},{},{}\n", sample.avg, sample.min, sample.max).as_bytes(),
                )
                .await?;
            sleep(Duration::from_secs(10)).await;
            continue;
        }
        sleep(Duration::from_secs(1)).await
    }
}
