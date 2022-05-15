mod am2302;
mod binutils;
mod cdev;

use am2302::{CreationError, Reading};
use cdev::push_pull;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Am2302ReadError {
    #[error("Could not read AM2302")]
    ReadError,
}

pub fn try_read(gpio_number: u32) -> Result<Reading, Am2302ReadError> {
    let all_data = push_pull(gpio_number);
    for data in all_data.windows(40) {
        let result = Reading::from_binary_vector(&data);
        match result {
            Ok(reading) => return Ok(reading),
            Err(e) => (),
        }
    }
    Err(Am2302ReadError::ReadError)
}
