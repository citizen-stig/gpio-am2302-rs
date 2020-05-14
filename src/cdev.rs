use std::{thread, time};
use gpio_cdev::{Chip, LineRequestFlags, EventType, EventRequestFlags, Line, LineEvent};

enum ThisEventType {
    RisingEdge,
    FallingEdge
}

struct Event {
    timestamp: u64,
    event_type: ThisEventType,
}



impl Event {
    fn from_line_event(line_event: &LineEvent) -> Self {
        Event {
            timestamp: line_event.timestamp().clone(),
            event_type: match line_event.event_type() {
                EventType::RisingEdge => ThisEventType::RisingEdge,
                EventType::FallingEdge => ThisEventType::FallingEdge,
            },
        }
    }
}

const LOW: u8 = 0;
const HIGH: u8 = 1;

fn get_line(gpio_number: u32) -> Line {
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    chip.get_line(gpio_number).unwrap()
}

fn do_init(line: &Line) {
    let output = line.request(
        LineRequestFlags::OUTPUT,
        HIGH,
        "pull-down").unwrap();

    // https://cdn-shop.adafruit.com/datasheets/Digital+humidity+and+temperature+sensor+AM2302.pdf
    // Step 1: MCU send out start signal to AM2302 and AM2302 send response signal to MCU

    // MCU will pull low data-bus and this process must beyond at least 1~10ms
    // to ensure AM2302 could detect MCU's signal
    output.set_value(LOW).unwrap();
    thread::sleep(time::Duration::from_millis(3));
    println!("pull low: {:?}", output.get_value().unwrap());

    // then MCU will pulls up and wait 20-40us for AM2302's response.
    output.set_value(HIGH).unwrap();
    println!("pull high: {:?}", output.get_value().unwrap());
    // HM?
    // thread::sleep(time::Duration::from_micros(20));
    // output.set_value(LOW).unwrap();
}

pub fn push_pull(gpio_number: u32) -> Vec<u8> {
    let line = get_line(gpio_number);
    println!("Line: {:?}", line);
    let mut transitions_made = 0;
    let mut data: Vec<u8> = Vec::new();
    do_init(&line);
    let input = line.request(
        LineRequestFlags::INPUT,
        HIGH,
        "read-data").unwrap();
    // println!("init: {:?}", last_state);

    let mut last_state = input.get_value().unwrap();
    let mut now = time::Instant::now();

    for _ in 0..1000000 {
        let new_state = input.get_value().unwrap();
        if new_state != last_state {
            let since_last = now.elapsed().as_micros();
            transitions_made += 1;
            // print!("Transition {:?} from {:?} to {:?} in {:?}us. step {:?}",
            //          transitions_made, last_state, new_state, since_last, i);
            if last_state == HIGH { // We were on signal
                let bit = if since_last > 35 { 1 } else { 0 };
                data.push(bit);
                // println!(". And it was data: {:?}", bit);
            }
            last_state = new_state;
            now = time::Instant::now();
        }
        // thread::sleep(time::Duration::from_micros(1));
    }
    println!("Transitions made: {:?}", transitions_made);
    println!("Data: {:?}", data);
    return data;

    // CHECK CONFIRMATION
    // When AM2302 detect the start signal,
    // AM2302 will pull low the bus 80us as response signal,
    // then AM2302 pulls up 80us for preparation to send data. See below figure:
    // println!("response 1: {:?}", input.get_value().unwrap());
    // thread::sleep(time::Duration::from_micros(80));
    // println!("response 2: {:?}", input.get_value().unwrap());
    // thread::sleep(time::Duration::from_micros(80));


    // for _ in 0..200 {
    //     print!("{:?}", input.get_value().unwrap());
    //     thread::sleep(time::Duration::from_micros(1));
    // }

    // for _ in 0..65000 {
    //     input.get_value().unwrap();
    //     thread::sleep(time::Duration::from_micros(5));
    // }


    // Does not work ...
    // for event in line.events(
    //     LineRequestFlags::INPUT,
    //     EventRequestFlags::BOTH_EDGES,
    //     "mirror-gpio",
    // ).unwrap() {
    //     let evt = event.unwrap();
    //     // println!("{:?}", evt);
    //     match evt.event_type() {
    //         EventType::RisingEdge => {
    //             print!("1");
    //         }
    //         EventType::FallingEdge => {
    //             print!("0");
    //         }
    //     }
    // }

    // let data = [0; 5];
    //

    // let mut last_state = input.get_value().unwrap();
    // println!("initial state: {:?}", last_state);

    // for _ in 0..65000 {
    //     let new_state = input.get_value().unwrap();
    //     if new_state != last_state {
    //         println!("Transition from {:?} to {:?}", last_state, new_state);
    //         last_state = new_state;
    //     }
    //     thread::sleep(time::Duration::from_micros(1));
    // }

    //
    // let mut j = 0;
    // for i in 0..MAX_TIMINGS {
    //     println!("I: {:?}", i);
    //     println!("Last state: {:?}", last_state);
    //     let mut counter: u8 = 0;
    //     while input.get_value().unwrap() == last_state {
    //         counter += 1;
    //         thread::sleep(time::Duration::from_micros(1));
    //         if counter == 255 {
    //             break;
    //         }
    //     }
    //     last_state = input.get_value().unwrap();
    //
    //     println!("Counter: {:?}", counter);
    //     if counter == 255 {
    //         break;
    //     }
    //     // print!("{:?}", last_state);
    //     if i >= 4 && i % 2 == 0 {
    //         println!("We are doing something here!");
    //         data[j / 8] <<= 1;
    //         if counter > 16 {
    //             data[j / 8] |= 1;
    //         }
    //         j += 1;
    //     }
    // }
    // println!("DATA: {:?}", data);
}

pub fn events_sub(gpio_number: u32) {
    println!("Using events sub");
    let line = get_line(gpio_number);

    let mut transitions_made = 0;
    let mut data: Vec<u8> = Vec::new();
    let mut recorded: Vec<Event> = Vec::new();
    do_init(&line);
    let events = line.events(
        LineRequestFlags::INPUT,
        EventRequestFlags::BOTH_EDGES,
        // EventRequestFlags::FALLING_EDGE,
        "try_read_gpio",
    ).unwrap();

    // let now = time::Instant::now();
    let mut last_timestamp: Option<u64> = None;
    for event in events {
        let evt = event.unwrap();
        // let since_last = evt.timestamp() - last_timestamp;
        let since_last = match last_timestamp {
            Some (last_timestamp) => evt.timestamp() - last_timestamp,
            None => 0,
        };
        if since_last > 1_000_000_000 {
            println!("Timeout. early break...");
            break;
        }
        println!("{:?}", evt);
        println!("Since last: {:?}", since_last);

        // let bit: u8 = if since_last > 68000 { 1 } else { 0 };
        // data.push(bit);
        last_timestamp = Some(evt.timestamp());
        transitions_made += 1;
        println!("Transitions made: {:?}", transitions_made);
        if transitions_made > 84 {
            break;
        }
    }
    println!("Transitions: {:?}", transitions_made);
    println!("DONE: {:?}", data);
}