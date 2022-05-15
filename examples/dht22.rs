use gpio_am2302_rs::try_read;
use std::{thread, time};

fn main() {
    let gpio_number = 4; // GPIO4  (7)
    let sleep_time = time::Duration::from_secs(5);
    for _ in 1..30 {
        println!(
            "Sleeping for another {:?}, to be sure that device is ready",
            sleep_time
        );
        thread::sleep(sleep_time);
        match try_read(gpio_number) {
            Ok(reading) => println!("Reading: {:?}", reading),
            _ => println!("Unable to get the data"),
        }
    }
}
