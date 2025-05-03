// protocol.rs
//! Core traits for device operations

use crate::error::Result;
use crate::types::{AcMode, FanSpeed, SwingH, SwingV};
use bitvec::prelude::*;

/// Trait for common device operations
pub trait Device {
    /// Set the device power state
    ///
    /// # Arguments
    /// * `on` - true to turn the device on, false to turn it off
    fn set_power(&mut self, on: bool) -> Result<()>;

    /// Get the current power state
    ///
    /// # Returns
    /// * `true` if the device is on, `false` if it's off
    fn get_power(&self) -> bool;

    /// Turn the device on
    fn turn_on(&mut self) -> Result<()> {
        self.set_power(true)
    }

    /// Turn the device off
    fn turn_off(&mut self) -> Result<()> {
        self.set_power(false)
    }

    /// Send the current state to the device
    fn send(&self) -> Result<()>;

    /// Get the raw state data
    fn get_raw(&self) -> Vec<u8>;

    /// Set the state from raw data
    ///
    /// # Arguments
    /// * `data` - The raw data to set the state from
    fn set_raw(&mut self, data: &[u8]) -> Result<()>;
}

/// Trait for AC device specific operations
pub trait AcDevice: Device {
    /// Set the operation mode
    fn set_mode(&mut self, mode: AcMode) -> Result<()>;

    /// Get the current operation mode
    fn get_mode(&self) -> Result<AcMode>;

    /// Set the temperature
    fn set_temp(&mut self, celsius: u8) -> Result<()>;

    /// Get the current temperature setting
    fn get_temp(&self) -> u8;

    /// Set the fan speed
    fn set_fan(&mut self, speed: FanSpeed) -> Result<()>;

    /// Get the current fan speed
    fn get_fan(&self) -> Result<FanSpeed>;

    /// Set the vertical swing position
    fn set_swing_v(&mut self, position: SwingV) -> Result<()>;

    /// Get the current vertical swing position
    fn get_swing_v(&self) -> Result<SwingV>;

    /// Set the horizontal swing position
    fn set_swing_h(&mut self, position: SwingH) -> Result<()>;

    /// Get the current horizontal swing position
    fn get_swing_h(&self) -> Result<SwingH>;

    /// Set quiet mode
    fn set_quiet(&mut self, on: bool) -> Result<()>;

    /// Get quiet mode status
    fn get_quiet(&self) -> bool;

    /// Set powerful mode
    fn set_powerful(&mut self, on: bool) -> Result<()>;

    /// Get powerful mode status
    fn get_powerful(&self) -> bool;

    /// Set the ion filter (if supported by the model)
    fn set_ion(&mut self, on: bool) -> Result<()>;

    /// Get ion filter status
    fn get_ion(&self) -> bool;
}

/// Bit manipulation operations trait for internal use
pub(crate) trait BitOps {
    /// Set a specific bit in a byte
    fn set_bit(&mut self, index: usize, bit_offset: u8, value: bool);

    /// Get a specific bit from a byte
    fn get_bit(&self, index: usize, bit_offset: u8) -> bool;

    /// Set multiple bits in a byte
    fn set_bits(&mut self, index: usize, offset: u8, size: u8, value: u8);

    /// Get multiple bits from a byte
    fn get_bits(&self, index: usize, offset: u8, size: u8) -> u8;
}

// Fixed BitOps implementation to prevent shift overflows
impl BitOps for [u8] {
    fn set_bit(&mut self, index: usize, bit_offset: u8, value: bool) {
        if index >= self.len() || bit_offset >= 8 {
            return;
        }

        let bits = self[index].view_bits_mut::<Lsb0>();
        bits.set(bit_offset as usize, value);
    }

    fn get_bit(&self, index: usize, bit_offset: u8) -> bool {
        if index >= self.len() || bit_offset >= 8 {
            return false;
        }

        let bits = self[index].view_bits::<Lsb0>();
        bits[bit_offset as usize]
    }

    fn set_bits(&mut self, index: usize, offset: u8, size: u8, value: u8) {
        if index >= self.len() || offset >= 8 || size > 8 || offset + size > 8 {
            return;
        }

        // Create a mask that only has bits set in the range we want to modify
        let mask = if size == 0 {
            0u8
        } else {
            ((1u8.wrapping_shl(size as u32)) - 1) << offset
        };

        // Clear the bits we want to set, then set them with the new value
        self[index] = (self[index] & !mask) | ((value << offset) & mask);
    }

    fn get_bits(&self, index: usize, offset: u8, size: u8) -> u8 {
        if index >= self.len() || offset >= 8 || size > 8 || offset + size > 8 {
            return 0;
        }

        let mask = if size == 0 {
            0u8
        } else if size == 8 {
            0xFF
        } else {
            (1u8 << size) - 1
        };

        (self[index] >> offset) & mask
    }
}
