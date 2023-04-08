use chrono::Utc;
use futures::prelude::*;
use influxdb2::Client;
use influxdb2_derive::WriteDataPoint;
use tokio::io::AsyncReadExt;
use tokio_serial::SerialPortBuilderExt;

#[derive(Default, WriteDataPoint)]
#[measurement = "soil_moisture"]
struct SoilMoisture {
    #[influxdb(tag)]
    plant: String,
    #[influxdb(field)]
    value: u64,
    #[influxdb(timestamp)]
    time: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tty_path = "/dev/ttyACM0";
    let mut serial_port = tokio_serial::new(tty_path, 9600).open_native_async()?;
    serial_port.set_exclusive(false).unwrap();

    let org = "snostorm";
    let bucket = "homelab";
    let influx_url = "http://localhost:8086";
    let token = std::env::var("INFLUXDB2_TOKEN").unwrap();
    let client = Client::new(influx_url, org, token);

    let mut read_buffer = [0u8; 256];
    let mut bytes_read = 0;
    let mut frame_start = 0;

    loop {
        if let Ok(n_bytes) = serial_port
            .read(&mut read_buffer[frame_start + bytes_read..])
            .await
        {
            bytes_read += n_bytes;
            while bytes_read >= 3
                && &read_buffer[frame_start..=frame_start + 2] != &[0xAA, 0xAA, 0xAA]
            {
                bytes_read -= 1;
                frame_start += 1;
            }

            if (bytes_read - frame_start) >= 13
                && &read_buffer[frame_start..=frame_start + 2] == &[0xAA, 0xAA, 0xAA]
            {
                let measure_start = frame_start + 3;
                let mut measurements = (0..5)
                    .into_iter()
                    .map(|id| &read_buffer[(measure_start + 2 * id)..=(measure_start + 2 * id + 1)])
                    .map(|measurement_raw| {
                        ((measurement_raw[0] as u16) << 8) + (measurement_raw[1] as u16)
                    })
                    .collect::<Vec<_>>();
                measurements.sort();
                let median = measurements[2];
                bytes_read = 0;
                frame_start = 0;

                if median > 0x3ff {
                    continue;
                }

                let points = vec![SoilMoisture {
                    plant: "gary".into(),
                    value: median.into(),
                    time: Utc::now().timestamp_nanos(),
                }];
                if let Ok(..) = client.write(bucket, stream::iter(points)).await {
                    println!("Writing data... {}", median);
                }
            } else {
                eprintln!(
                    "Uh oh, mid-sequence... And I'm lazy...\n{:x?}",
                    &read_buffer[frame_start..frame_start + bytes_read]
                );
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
        }
    }
}
