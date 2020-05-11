// extern crate sysfs_gpio;

mod binutils;
mod am2302;
mod cdev;


use std::{thread, time};
use cdev::{push_pull, events_sub};

fn main() {
    // let gpio_number = 17;    // GPIO17 (11)
    let gpio_number = 4;  // GPIO4  (7)
    // push_pull(gpio_number);
    println!("Sleeping for another 30 seconds, to be sure, that device is ready");
    thread::sleep(time::Duration::from_secs(30));
    events_sub(gpio_number);
    // read(gpio_number);
}