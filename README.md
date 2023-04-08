# plant-thirst
* `plant-thirst-sensor.ino` - Arduino script which reads an analog pin 5 times and sends the data over the serial port every 5 seconds.
* `src/main.rs` - Tiny Rust program which reads the serial data measurements, takes the median, and writes it into a local Influx database.