use crate::binutils::{convert, ConversionError};
use crate::am2302::CreationError::OutOfSpecValue;

#[derive(Debug, PartialEq)]
pub struct Reading {
    temperature: f32,
    humidity: f32,
}

#[derive(Debug, PartialEq)]
pub enum CreationError {
    WrongBitsCount,
    // Wrong number of input bites, should be 40
    MalformedData,
    // Something wrong with conversion to bytes
    ParityBitMismatch,
    // Parity Bit Validation Failed
    OutOfSpecValue,       // Value is outside of specification
}

impl Reading {
    pub fn from_binary_vector(data: &[u8]) -> Result<Self, CreationError> {
        if data.len() != 40 {
            return Err(CreationError::WrongBitsCount);
        }

        let bytes: Result<Vec<u8>, ConversionError> = data
            .chunks(8)
            .map(|chunk| -> Result<u8, ConversionError> {
                convert(chunk)
            })
            .collect();

        let bytes = match bytes {
            Ok(this_bytes) => this_bytes,
            Err(e) => return Err(CreationError::MalformedData),
        };

        let check_sum: u8 = bytes[..4].iter().sum();
        if check_sum != bytes[4] {
            return Err(CreationError::ParityBitMismatch);
        }

        if bytes[2] > 1 {   // Temperature too high
            return Err(OutOfSpecValue)
        }

        let raw_humidity: u16 = (bytes[0] as u16) * 256 + bytes[1] as u16;
        let raw_temperature: f32 = ((bytes[2] as u16) * 256 + bytes[3] as u16) as f32 / 10.0;
        let temperature = if bytes[2] == 0 {
           raw_temperature * -1.0
        } else {
            raw_temperature
        };

        // TODO: Check value according to specification

        Ok(Reading {
            temperature,
            humidity: raw_humidity as f32 / 10.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Partially based on documentation:
    // http://akizukidenshi.com/download/ds/aosong/AM2302.pdf

    #[test]
    fn not_enough_bits() {
        let result = Reading::from_binary_vector(&vec![]);
        assert_eq!(result, Err(CreationError::WrongBitsCount));
    }

    #[test]
    fn too_many_enough_bits() {
        let result = Reading::from_binary_vector(
            &vec![
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1,
            ]
        );
        assert_eq!(result, Err(CreationError::WrongBitsCount));
    }

    #[test]
    fn malformed_input() {
        let result = Reading::from_binary_vector(
            &vec![
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 2,
            ]
        );
        assert_eq!(result, Err(CreationError::MalformedData));
    }

    #[test]
    fn wrong_parity_bit() {
        let result = Reading::from_binary_vector(
            &vec![
                0, 0, 0, 0, 0, 0, 1, 0,  // humidity high
                1, 0, 0, 1, 0, 0, 1, 0,  // humidity high
                0, 0, 0, 0, 0, 0, 0, 1,  // temperature high
                0, 0, 0, 0, 1, 1, 0, 1,  // temperature low
                1, 0, 1, 1, 0, 0, 1, 0,  // parity
            ]
        );
        assert_eq!(result, Err(CreationError::ParityBitMismatch));
    }

    #[test]
    fn correct_reading() {
        let result = Reading::from_binary_vector(
            &vec![
                0, 0, 0, 0, 0, 0, 1, 0,  // humidity high
                1, 0, 0, 1, 0, 0, 1, 0,  // humidity high
                0, 0, 0, 0, 0, 0, 0, 1,  // temperature high
                0, 0, 0, 0, 1, 1, 0, 1,  // temperature low
                1, 0, 1, 0, 0, 0, 1, 0,  // parity
            ]
        );

        let expected_reading = Reading {
            temperature: 26.9,
            humidity: 65.8,
        };

        assert_eq!(result, Ok(expected_reading));
    }

    #[test]
    fn negative_temperature() {
        let result = Reading::from_binary_vector(
            &vec![
                0, 0, 0, 0, 0, 0, 1, 0,  // humidity high
                1, 0, 0, 1, 0, 0, 1, 0,  // humidity high
                0, 0, 0, 0, 0, 0, 0, 0,  // temperature high
                0, 1, 1, 0, 0, 1, 0, 1,  // temperature low
                1, 1, 1, 1, 1, 0, 0, 1,  // parity
            ]
        );

        let expected_reading = Reading {
            temperature: -10.1,
            humidity: 65.8,
        };

        assert_eq!(result, Ok(expected_reading));
    }
}

