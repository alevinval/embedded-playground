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
    let humidity = Uuid::parse_str("987312e0-2354-11eb-9f10-fbc30a62cf50").unwrap();
    let historical = Uuid::parse_str("987312e0-2354-11eb-9f10-fbc30a62cf60").unwrap();
    let manager = Manager::new().await?;
    let central = get_central(&manager).await;
    central.start_scan(ScanFilter::default()).await.expect("failed starting scan");

    loop {
        let device = find_esp32(&central).await;
        if let Some(esp32) = device {
            println!("connecting to esp32s3");
            let d = Instant::now();
            esp32.connect().await?;
            println!("connected to esp32s3");

            let chars = esp32.characteristics();
            let cmd_humidity = chars.iter().find(|c| c.uuid == humidity).unwrap();
            let cmd_historical = chars.iter().find(|c| c.uuid == historical).unwrap();

            let single_read = esp32.read(cmd_humidity).await.unwrap();
            // Read historical
            let mut historical_read = Vec::<u8>::new();
            loop {
                let mut chunk = esp32.read(cmd_historical).await.unwrap();
                println!(" => chunk: {:?}", chunk);
                if chunk.len() == 0 {
                    break;
                }
                historical_read.append(&mut chunk);
            }
            esp32.disconnect().await?;
            println!("esp32 disconnected, elapsed: {:?}", d.elapsed());

            let mut de = sample::Deserializer::new(&single_read);
            let sample = de.deserialize().unwrap();
            println!("latest sample: {sample:?} dryness: {}", sample.dryness());
            println!("fetched historical buffer: {:?}", historical_read);

            let mut open = OpenOptions::new();
            let mut output = open.write(true).append(true).open("data.csv").await?;
            let now = Local::now().format("%Y-%m-%d %H:%M:%s");
            output
                .write_all(
                    format!("{now},{},{},{}\n", sample.avg, sample.min, sample.max).as_bytes(),
                )
                .await?;
            continue;
        }
        sleep(Duration::from_secs(1)).await;
    }
}
