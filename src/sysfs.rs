use std::{thread, time};

use sysfs_gpio::{Pin, Direction};


const LOW: u8 = 0;
const HIGH: u8 = 1;

pub fn read(gpio_number: u64) {
    let pin = Pin::new(gpio_number);
    match pin.with_exported(|| {
        let mut transitions_made = 0;
        let mut data: Vec<u8> = Vec::new();
        // INIT
        println!("Doing init");
        pin.set_direction(Direction::High)?;
        pin.set_value(LOW)?;
        thread::sleep(time::Duration::from_millis(1));
        pin.set_value(HIGH)?;

        pin.set_direction(Direction::In)?;
        let mut now = time::Instant::now();
        let mut last_state: u8 = HIGH;

        for _ in 0..1000000 {
            let new_state = pin.get_value()?;
            if new_state != last_state {
                let since_last = now.elapsed().as_micros();
                transitions_made += 1;
                if last_state == HIGH { // We were on signal
                    let bit = if since_last > 35 { 1 } else { 0 };
                    data.push(bit);
                }
                last_state = new_state;
                now = time::Instant::now();
            }
        }

        println!("Transitions made: {:?}", transitions_made);
        println!("Data: {:?}", data);

        Ok(())
    }) {
        Ok(()) =>  println!("Done"),
        Err(err) => println!("OOps: {:?}", err),
    };
}