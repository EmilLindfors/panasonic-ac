// encoder.rs
//! IR signal encoding and decoding for Panasonic AC protocol

use crate::constants::{
    PANASONIC_AC_CHECKSUM_INIT,
    PANASONIC_HDR_MARK,
    PANASONIC_HDR_SPACE,
    PANASONIC_BIT_MARK,
    PANASONIC_ONE_SPACE,
    PANASONIC_ZERO_SPACE,
    PANASONIC_END_GAP,
    PANASONIC_FREQ
};
use crate::error::{Error, Result};
use log::{debug, trace};

/// Represents an IR signal with timing information
#[derive(Debug, Clone)]
pub struct IrSignal {
    /// Raw data bytes
    pub data: Vec<u8>,
    /// Carrier frequency in Hz
    pub carrier_frequency: u32,
    /// Timing information for marks and spaces
    pub timings: Option<Vec<u16>>,
}

/// Trait for encoding data into IR signals
pub trait Encoder {
    /// Encode data into an IR signal
    fn encode(&self, data: &[u8]) -> Result<IrSignal>;

    /// Encode a specific command into an IR signal
    fn encode_command(&self, command: u32) -> Result<IrSignal>;
}

/// Trait for decoding IR signals into data
pub trait Decoder {
    /// Decode an IR signal into data
    fn decode(&self, signal: &IrSignal) -> Result<Vec<u8>>;

    /// Extract a command from a decoded signal
    fn extract_command(&self, data: &[u8]) -> Result<u32>;
}

/// Implementation of Panasonic IR protocol encoder
pub struct PanasonicEncoder;

impl Default for PanasonicEncoder {
    fn default() -> Self {
        PanasonicEncoder
    }
}

impl PanasonicEncoder {
    /// Create a new PanasonicEncoder
    pub fn new() -> Self {
        PanasonicEncoder
    }

    /// Calculate the checksum for a Panasonic AC data block
    pub fn calculate_checksum(data: &[u8], length: usize) -> u8 {
        if length == 0 {
            return PANASONIC_AC_CHECKSUM_INIT;
        }

        crate::utils::sum_bytes(data, length - 1, PANASONIC_AC_CHECKSUM_INIT)
    }

    /// Verify if the checksum in the data is valid
    pub fn verify_checksum(data: &[u8], length: usize) -> bool {
        if length < 2 || data.len() < length {
            return false;
        }

        data[length - 1] == Self::calculate_checksum(data, length)
    }

    /// Encode a 48-bit Panasonic command
    pub fn encode_panasonic64(manufacturer: u16, device: u8, subdevice: u8, function: u8) -> u64 {
        let checksum = device ^ subdevice ^ function;

        ((manufacturer as u64) << 32)
            | ((device as u64) << 24)
            | ((subdevice as u64) << 16)
            | ((function as u64) << 8)
            | (checksum as u64)
    }

    /// Generate the timing sequence for a Panasonic IR signal
    fn generate_timings(&self, data: &[u8]) -> Vec<u16> {
        let mut timings = Vec::with_capacity(2 + data.len() * 16 + 2);

        // Header
        timings.push(PANASONIC_HDR_MARK);
        timings.push(PANASONIC_HDR_SPACE);

        // Data
        for byte in data {
            for bit in (0..8).rev() {
                timings.push(PANASONIC_BIT_MARK);
                if (*byte & (1 << bit)) != 0 {
                    timings.push(PANASONIC_ONE_SPACE);
                } else {
                    timings.push(PANASONIC_ZERO_SPACE);
                }
            }
        }

        // Footer
        timings.push(PANASONIC_BIT_MARK);
        timings.push(PANASONIC_END_GAP);

        timings
    }
}

impl Encoder for PanasonicEncoder {
    fn encode(&self, data: &[u8]) -> Result<IrSignal> {
        debug!("Encoding {} bytes of Panasonic IR data", data.len());
        let timings = self.generate_timings(data);

        Ok(IrSignal {
            data: data.to_vec(),
            carrier_frequency: PANASONIC_FREQ,
            timings: Some(timings),
        })
    }

    fn encode_command(&self, command: u32) -> Result<IrSignal> {
        let command_bytes = command.to_be_bytes();
        self.encode(&command_bytes)
    }
}

/// Implementation of Panasonic IR protocol decoder
pub struct PanasonicDecoder;

impl Default for PanasonicDecoder {
    fn default() -> Self {
        PanasonicDecoder
    }
}

impl PanasonicDecoder {
    /// Create a new PanasonicDecoder
    pub fn new() -> Self {
        PanasonicDecoder
    }

    /// Parse timings into data
    fn parse_timings(&self, timings: &[u16]) -> Result<Vec<u8>> {
        // Header validation
        if timings.len() < 4
            || !Self::validate_timing(timings[0], PANASONIC_HDR_MARK)
            || !Self::validate_timing(timings[1], PANASONIC_HDR_SPACE)
        {
            return Err(Error::ProtocolError("Invalid header timing".to_string()));
        }

        let bit_count = (timings.len() - 4) / 2; // Subtract header and footer
        if bit_count % 8 != 0 {
            return Err(Error::ProtocolError(format!(
                "Bit count {} is not a multiple of 8",
                bit_count
            )));
        }

        let byte_count = bit_count / 8;
        let mut data = vec![0u8; byte_count];

        // Parse data
        for i in 0..bit_count {
            let timing_idx = 2 + i * 2; // Skip header
            if timing_idx + 1 >= timings.len() {
                break;
            }

            // Check mark
            if !Self::validate_timing(timings[timing_idx], PANASONIC_BIT_MARK) {
                continue; // Invalid mark, skip this bit
            }

            // Check space to determine bit value
            let bit_value = if Self::validate_timing(timings[timing_idx + 1], PANASONIC_ONE_SPACE) {
                1 // One bit
            } else {
                0 // Zero bit
            };

            // Set bit in data
            let byte_idx = i / 8;
            let bit_pos = 7 - (i % 8);
            if byte_idx < data.len() && bit_value == 1 {
                data[byte_idx] |= 1 << bit_pos;
            }
        }

        trace!("Decoded {} bytes from timings", data.len());
        Ok(data)
    }

    /// Validate if a timing value is within tolerance
    fn validate_timing(value: u16, expected: u16) -> bool {
        let tolerance = expected / 4; // 25% tolerance
        (value >= expected - tolerance) && (value <= expected + tolerance)
    }
}

impl Decoder for PanasonicDecoder {
    fn decode(&self, signal: &IrSignal) -> Result<Vec<u8>> {
        // If we have timings, parse them
        if let Some(timings) = &signal.timings {
            debug!("Decoding IR signal with {} timing points", timings.len());
            return self.parse_timings(timings);
        }

        // Otherwise, just return the data
        debug!("Using raw data from signal (no timings to decode)");
        Ok(signal.data.clone())
    }

    fn extract_command(&self, data: &[u8]) -> Result<u32> {
        if data.len() < 4 {
            return Err(Error::InvalidData(
                "Data too short to extract command".to_string(),
            ));
        }

        let mut command = 0u32;
        for &byte in data.iter().take(4) {
            command = (command << 8) | (byte as u32);
        }

        Ok(command)
    }
}
