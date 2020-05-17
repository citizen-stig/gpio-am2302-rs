mod binutils;
mod am2302;
mod cdev;

use std::{thread, time};
use cdev::push_pull;
use am2302::Reading;


fn try_read(gpio_number: u32) -> Option<Reading> {
    let mut final_result = None;
    let all_data = push_pull(gpio_number);
    if all_data.len() < 40 {
        println!("Saad, read not enough data");
        return final_result;
    }
    for data in all_data.windows(40) {
        let result = Reading::from_binary_vector(&data);
        match result {
            Ok(reading) => {
                final_result = Some(reading);
                break;
            }
            Err(e) => {
                println!("Error: {:?}", e)
            }
        }
    }
    final_result
}

fn main() {
    let gpio_number = 4;  // GPIO4  (7)
    let sleep_time = time::Duration::from_secs(5);
    for _ in 1..30 {
        println!("Sleeping for another {:?}, to be sure that device is ready", sleep_time);
        thread::sleep(sleep_time);
        match try_read(gpio_number) {
            Some(reading) => println!("Reading: {:?}", reading),
            None => println!("Unable to get the data"),
        }
    }
}