use btleplug::api::Peripheral;

use infrastructure::term;
use std::error::Error;
use tokio::time::Instant;
use uuid::Uuid;

mod application;
mod infrastructure;

async fn collect_data(device: impl Peripheral) -> Result<(), Box<dyn Error>> {
    let pong = Uuid::try_parse("987312e0-2354-11eb-9f10-fbc30a62cf50")?;
    // let _historical = Uuid::try_parse("987312e0-2354-11eb-9f10-fbc30a62cf60")?;

    // println!("connecting to {}", device.id());
    let since_connecting = Instant::now();
    device.connect().await?;
    // device.discover_services().await?;

    // println!("connected to {}", device.id());

    let characteristics = device.characteristics();
    // println!("characteristics: {:?}", characteristics);

    let pong_cmd = characteristics.iter().find(|c| c.uuid == pong).unwrap();
    // // let _cmd_historical = chars.iter().find(|c| c.uuid == _historical).unwrap();

    let single_read = device.read(pong_cmd).await.unwrap();
    // println!("reading: {:?}", single_read);

    device.disconnect().await?;
    // println!("disconnected, elapsed: {:?}", since_connecting.elapsed());

    // let summary: Summary<Hygrometer> = serde::deserialize(&single_read).unwrap();
    // println!("latest sample: {summary:?} dryness: {}", summary.sensor.percentage(summary.avg));

    // let mut open = OpenOptions::new();
    // let mut output = open.write(true).append(true).open("data.csv").await?;
    // let now = Local::now().format("%Y-%m-%dT%H:%M:%SZ");
    // output
    //     .write_all(format!("{now},{},{},{}\n", summary.avg, summary.min, summary.max).as_bytes())
    //     .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    term::main().await
}
