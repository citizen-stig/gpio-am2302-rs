use crate::binutils::{convert, ConversionError};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Reading {
    temperature: f32,
    humidity: f32,
}

#[derive(Error, Debug, PartialEq)]
pub enum CreationError {
    // Wrong number of input bites, should be 40
    #[error("Wrong number of input bites, should be 40")]
    WrongBitsCount,

    // Something wrong with conversion to bytes
    #[error("Something wrong with conversion to bytes")]
    MalformedData,

    // Parity Bit Validation Failed
    #[error("Parity Bit Validation Failed")]
    ParityBitMismatch,

    // Value is outside of specification
    #[error("Value is outside of specification")]
    OutOfSpecValue,
}

impl Reading {
    pub fn from_binary_vector(data: &[u8]) -> Result<Self, CreationError> {
        if data.len() != 40 {
            return Err(CreationError::WrongBitsCount);
        }

        let bytes: Result<Vec<u8>, ConversionError> = data
            .chunks(8)
            .map(|chunk| -> Result<u8, ConversionError> { convert(chunk) })
            .collect();

        let bytes = match bytes {
            Ok(this_bytes) => this_bytes,
            Err(_e) => return Err(CreationError::MalformedData),
        };

        // let check_sum: u8 = bytes[..4].iter().sum();
        let check_sum: u8 = bytes[..4]
            .iter()
            .fold(0 as u8, |result, &value| result.overflowing_add(value).0);
        if check_sum != bytes[4] {
            return Err(CreationError::ParityBitMismatch);
        }

        let raw_humidity: u16 = (bytes[0] as u16) * 256 + bytes[1] as u16;
        let raw_temperature: i16 = if bytes[2] >= 128 {
            bytes[3] as i16 * -1
        } else {
            (bytes[2] as i16) * 256 + bytes[3] as i16
        };

        let humidity: f32 = raw_humidity as f32 / 10.0;
        let temperature: f32 = raw_temperature as f32 / 10.0;

        if temperature > 81.0 || temperature < -41.0 {
            return Err(CreationError::OutOfSpecValue);
        }
        if humidity < 0.0 || humidity > 99.9 {
            return Err(CreationError::OutOfSpecValue);
        }

        Ok(Reading {
            temperature,
            humidity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Partially based on documentation:
    // http://akizukidenshi.com/download/ds/aosong/AM2302.pdf
    // and
    // https://cdn-shop.adafruit.com/datasheets/Digital+humidity+and+temperature+sensor+AM2302.pdf

    #[test]
    fn not_enough_bits() {
        let result = Reading::from_binary_vector(&vec![]);
        assert_eq!(result, Err(CreationError::WrongBitsCount));
    }

    #[test]
    fn too_many_enough_bits() {
        let result = Reading::from_binary_vector(&vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ]);
        assert_eq!(result, Err(CreationError::WrongBitsCount));
    }

    #[test]
    fn malformed_input() {
        let result = Reading::from_binary_vector(&vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2,
        ]);
        assert_eq!(result, Err(CreationError::MalformedData));
    }

    #[test]
    fn wrong_parity_bit() {
        let result = Reading::from_binary_vector(&vec![
            0, 0, 0, 0, 0, 0, 1, 0, // humidity high
            1, 0, 0, 1, 0, 0, 1, 0, // humidity low
            0, 0, 0, 0, 0, 0, 0, 1, // temperature high
            0, 0, 0, 0, 1, 1, 0, 1, // temperature low
            1, 0, 1, 1, 0, 0, 1, 0, // parity
        ]);
        assert_eq!(result, Err(CreationError::ParityBitMismatch));
    }

    #[test]
    fn correct_reading() {
        let result = Reading::from_binary_vector(&vec![
            0, 0, 0, 0, 0, 0, 1, 0, // humidity high
            1, 0, 0, 1, 0, 0, 1, 0, // humidity low
            0, 0, 0, 0, 0, 0, 0, 1, // temperature high
            0, 0, 0, 0, 1, 1, 0, 1, // temperature low
            1, 0, 1, 0, 0, 0, 1, 0, // parity
        ]);

        let expected_reading = Reading {
            temperature: 26.9,
            humidity: 65.8,
        };

        assert_eq!(result, Ok(expected_reading));
    }

    #[test]
    fn negative_temperature() {
        let result = Reading::from_binary_vector(&vec![
            0, 0, 0, 0, 0, 0, 1, 0, // humidity high
            1, 0, 0, 1, 0, 0, 1, 0, // humidity low
            1, 0, 0, 0, 0, 0, 0, 0, // temperature high
            0, 1, 1, 0, 0, 1, 0, 1, // temperature low
            0, 1, 1, 1, 1, 0, 0, 1, // parity
        ]);

        let expected_reading = Reading {
            temperature: -10.1,
            humidity: 65.8,
        };

        assert_eq!(result, Ok(expected_reading));
    }

    #[test]
    fn add_with_overflow() {
        let result = Reading::from_binary_vector(&vec![
            1, 0, 0, 0, 0, 0, 0, 0, // humidity high
            1, 0, 0, 0, 0, 1, 0, 1, // humidity low
            0, 0, 0, 0, 0, 0, 0, 0, // temperature high
            1, 0, 0, 0, 1, 1, 1, 1, // temperature low
            0, 0, 0, 1, 0, 1, 0, 1, // parity
        ]);
        assert_eq!(result, Err(CreationError::ParityBitMismatch));
    }

    #[test]
    fn another_example() {
        let result = Reading::from_binary_vector(&vec![
            0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0,
        ]);
        let expected_reading = Reading {
            temperature: 35.1,
            humidity: 65.2,
        };

        assert_eq!(result, Ok(expected_reading));
    }

    #[test]
    fn another_example_negative_temp() {
        let result = Reading::from_binary_vector(&vec![
            0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
            1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1,
        ]);
        let expected_reading = Reading {
            temperature: -10.1,
            humidity: 65.2,
        };

        assert_eq!(result, Ok(expected_reading));
    }
}
