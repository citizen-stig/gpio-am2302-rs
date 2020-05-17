// extern crate sysfs_gpio;

mod binutils;
mod am2302;
mod cdev;


use std::{thread, time};
use cdev::push_pull;
use am2302::Reading;


fn try_read(gpio_number: u32) {
    let all_data = push_pull(gpio_number);
    if all_data.len() < 40 {
        println!("Saad, read not enough data");
    }
    for data in all_data.windows(40) {
        let result = Reading::from_binary_vector(&data);
        match result {
            Ok(reading) => {
                println!("Unbelievable, it is done: {:?}", reading);
            }
            Err(e) => {
                println!("Error: {:?}", e)
            }
        }
    }
}

fn main() {
    // let gpio_number = 17;    // GPIO17 (11)
    let gpio_number = 4;  // GPIO4  (7)
    for _ in 1..30 {
        println!("Sleeping for another 30 seconds, to be sure, that device is ready");
        thread::sleep(time::Duration::from_secs(10));
        try_read(gpio_number);
    }


    // events_sub(gpio_number);
    // read(gpio_number);
}