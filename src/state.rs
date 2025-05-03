// state.rs
//! State management implementation for Panasonic AC devices

use std::cmp::{max, min};
use std::fmt;

use crate::constants::*;
use crate::device::PanasonicAcModel;
use crate::encoder::PanasonicEncoder;
use crate::error::{Error, Result};
use crate::protocol::BitOps;
use crate::types::{AcMode, FanSpeed, SwingH, SwingV};

/// The core state representation for a Panasonic AC
#[derive(Clone, Debug)]
pub struct PanasonicAcState {
    /// Raw state buffer
    buffer: [u8; PANASONIC_AC_STATE_LENGTH],
    /// Model configuration
    model: PanasonicAcModel,
}

impl Default for PanasonicAcState {
    fn default() -> Self {
        Self {
            buffer: PANASONIC_KNOWN_GOOD_STATE,
            model: PanasonicAcModel::Unknown,
        }
    }
}

impl PanasonicAcState {
    /// Create a new state object with known good defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the model of the AC
    pub fn set_model(&mut self, model: PanasonicAcModel) -> Result<()> {
        self.model = model;

        // Save the current power state before modifying the buffer
        let power_state = self.get_power();

        // Clear & set various bits and bytes according to the model, but preserve the power bit
        let power_mask = 1 << PANASONIC_AC_POWER_OFFSET;
        let power_bit = self.buffer[13] & power_mask; // Save the power bit

        self.buffer[13] &= 0xF0 | power_mask; // Clear bits except power and high nibble
        self.buffer[17] = 0x00;
        self.buffer[21] &= 0b1110_1111;
        self.buffer[23] = 0x81;
        self.buffer[25] = 0x00;

        match model {
            PanasonicAcModel::Lke => {
                self.buffer[13] |= 0x02;
                self.buffer[17] = 0x06;
            }
            PanasonicAcModel::Dke => {
                self.buffer[23] = 0x01;
                self.buffer[25] = 0x06;
                // Set swing_h to Middle as default for DKE models
                // Initialize the buffer with a valid value before using get_swing_h
                self.buffer.set_bits(
                    17,
                    PANASONIC_AC_SWINGH_OFFSET,
                    PANASONIC_AC_SWINGH_SIZE,
                    PANASONIC_AC_SWING_H_MIDDLE,
                );
            }
            PanasonicAcModel::Nke => {
                self.buffer[17] = 0x06;
            }
            PanasonicAcModel::Jke => {}
            PanasonicAcModel::Ckp => {
                self.buffer[21] |= 0x10;
                self.buffer[23] = 0x01;
            }
            PanasonicAcModel::Rkr => {
                self.buffer[13] |= 0x08 | power_bit; // Preserve power bit when setting model bits
                self.buffer[23] = 0x89;
                // Set swing_h to Middle as default for RKR models
                self.buffer.set_bits(
                    17,
                    PANASONIC_AC_SWINGH_OFFSET,
                    PANASONIC_AC_SWINGH_SIZE,
                    PANASONIC_AC_SWING_H_MIDDLE,
                );
            }
            PanasonicAcModel::Unknown => {
                return Err(Error::InvalidValue("Unknown model".to_string()));
            }
        }

        // Reset ion filter
        if self.model_supports_ion() {
            self.set_ion(self.get_ion())?;
        }

        // Explicitly restore the power state
        self.set_power(power_state)?;

        Ok(())
    }

    /// Get the model of the AC
    pub fn get_model(&self) -> PanasonicAcModel {
        self.model
    }

    /// Check if a model supports ion filter functionality
    pub fn model_supports_ion(&self) -> bool {
        matches!(self.model, PanasonicAcModel::Dke)
    }

    /// Check if a model supports horizontal swing functionality
    pub fn model_supports_horizontal_swing(&self) -> bool {
        matches!(self.model, PanasonicAcModel::Dke | PanasonicAcModel::Rkr)
    }

    /// Get the buffer for IR transmission
    pub fn get_buffer(&self) -> &[u8; PANASONIC_AC_STATE_LENGTH] {
        &self.buffer
    }

    /// Get a mutable reference to the buffer
    pub fn get_buffer_mut(&mut self) -> &mut [u8; PANASONIC_AC_STATE_LENGTH] {
        &mut self.buffer
    }

    /// Calculate and fix the checksum for the current state
    pub fn fix_checksum(&mut self) {
        self.buffer[PANASONIC_AC_STATE_LENGTH - 1] =
            PanasonicEncoder::calculate_checksum(&self.buffer, PANASONIC_AC_STATE_LENGTH);
    }

    /// Verify the checksum for the state
    pub fn verify_checksum(&self) -> bool {
        PanasonicEncoder::verify_checksum(&self.buffer, PANASONIC_AC_STATE_LENGTH)
    }

    /// Clone the state buffer
    pub fn to_vec(&self) -> Vec<u8> {
        let mut state = self.buffer.to_vec();
        state[PANASONIC_AC_STATE_LENGTH - 1] =
            PanasonicEncoder::calculate_checksum(&state, PANASONIC_AC_STATE_LENGTH);
        state
    }

    /// Set the state from raw data
    pub fn from_raw_data(&mut self, data: &[u8]) -> Result<()> {
        if data.len() != PANASONIC_AC_STATE_LENGTH {
            return Err(Error::InvalidData(format!(
                "Expected {} bytes, got {}",
                PANASONIC_AC_STATE_LENGTH,
                data.len()
            )));
        }

        // Verify checksum
        if !PanasonicEncoder::verify_checksum(data, PANASONIC_AC_STATE_LENGTH) {
            return Err(Error::InvalidChecksum);
        }

        // Extract power state from the incoming data before copying
        let power_bit = data[13] & (1 << PANASONIC_AC_POWER_OFFSET) != 0;

        // Copy the state
        self.buffer.copy_from_slice(data);

        // Detect the model
        let detected_model = self.detect_model();

        // If we have a known model, set it (which will initialize model-specific fields)
        if detected_model != PanasonicAcModel::Unknown {
            // Set the model, which might reset some fields
            self.set_model(detected_model)?;

            // Explicitly restore the power state from the incoming data
            self.set_power(power_bit)?;
        } else {
            self.model = detected_model;
        }

        Ok(())
    }

    /// Detect model based on state buffer contents
    fn detect_model(&self) -> PanasonicAcModel {
        if self.buffer[23] == 0x89 {
            return PanasonicAcModel::Rkr;
        }

        if self.buffer[17] == 0x00 {
            if (self.buffer[21] & 0x10 != 0) && (self.buffer[23] & 0x01 != 0) {
                return PanasonicAcModel::Ckp;
            }
            if self.buffer[23] & 0x80 != 0 {
                return PanasonicAcModel::Jke;
            }
        }

        if self.buffer[17] == 0x06 && (self.buffer[13] & 0x0F) == 0x02 {
            return PanasonicAcModel::Lke;
        }

        if self.buffer[23] == 0x01 {
            return PanasonicAcModel::Dke;
        }

        if self.buffer[17] == 0x06 {
            return PanasonicAcModel::Nke;
        }

        PanasonicAcModel::Unknown
    }

    /// Set the power state (on/off)
    pub fn set_power(&mut self, on: bool) -> Result<()> {
        // Directly set or clear the power bit
        if on {
            self.buffer[13] |= 1 << PANASONIC_AC_POWER_OFFSET;
        } else {
            self.buffer[13] &= !(1 << PANASONIC_AC_POWER_OFFSET);
        }
        Ok(())
    }

    /// Get the power state
    pub fn get_power(&self) -> bool {
        // Directly check the power bit
        (self.buffer[13] & (1 << PANASONIC_AC_POWER_OFFSET)) != 0
    }

    /// Set the operating mode
    pub fn set_mode(&mut self, mode: AcMode) -> Result<()> {
        let mode_value: u8 = mode.into();

        // If switching to Fan mode, set a special temperature
        if mode == AcMode::Fan {
            self.set_temp(PANASONIC_AC_FAN_MODE_TEMP)?;
        }

        // Clear the previous mode bits and set the new mode
        self.buffer[13] &= 0x0F;
        self.buffer.set_bits(
            13,
            PANASONIC_AC_MODE_OFFSET,
            PANASONIC_AC_MODE_SIZE,
            mode_value,
        );

        Ok(())
    }

    /// Get the current operating mode
    pub fn get_mode(&self) -> Result<AcMode> {
        let mode_value = self
            .buffer
            .get_bits(13, PANASONIC_AC_MODE_OFFSET, PANASONIC_AC_MODE_SIZE);
        AcMode::try_from(mode_value)
    }

    /// Set the temperature in Celsius
    pub fn set_temp(&mut self, celsius: u8) -> Result<()> {
        // Constrain temperature to valid range
        let temp = max(celsius, PANASONIC_AC_MIN_TEMP);
        let temp = min(temp, PANASONIC_AC_MAX_TEMP);

        // Set the temperature in the state buffer
        self.buffer[14] = temp; // Directly set byte 14 to the temperature value

        Ok(())
    }

    /// Get the current temperature setting in Celsius
    pub fn get_temp(&self) -> u8 {
        self.buffer[14] // Directly get the temperature from byte 14
    }

    /// Set the fan speed
    pub fn set_fan(&mut self, speed: FanSpeed) -> Result<()> {
        let speed_value: u8 = speed.into();

        self.buffer.set_bits(
            16,
            PANASONIC_AC_FAN_OFFSET,
            PANASONIC_AC_FAN_SIZE,
            speed_value + PANASONIC_AC_FAN_DELTA,
        );
        Ok(())
    }

    /// Get the current fan speed
    pub fn get_fan(&self) -> Result<FanSpeed> {
        let raw_value = self
            .buffer
            .get_bits(16, PANASONIC_AC_FAN_OFFSET, PANASONIC_AC_FAN_SIZE);

        // Use wrapping_sub to avoid panic, then check if result is valid
        let fan_value = raw_value.wrapping_sub(PANASONIC_AC_FAN_DELTA);

        // If fan_value is too large (wrap around occurred), return an error
        if fan_value > 5 {
            // Assuming max valid value is 5
            return Err(Error::InvalidValue(format!(
                "Invalid fan value after adjustment: {} (original: {})",
                fan_value, raw_value
            )));
        }

        FanSpeed::try_from(fan_value)
    }

    /// Set the vertical swing position
    pub fn set_swing_v(&mut self, position: SwingV) -> Result<()> {
        let position_value: u8 = position.into();

        self.buffer.set_bits(
            16,
            PANASONIC_AC_SWINGV_OFFSET,
            PANASONIC_AC_SWINGV_SIZE,
            position_value,
        );
        Ok(())
    }

    /// Get the current vertical swing position
    pub fn get_swing_v(&self) -> Result<SwingV> {
        let value = self
            .buffer
            .get_bits(16, PANASONIC_AC_SWINGV_OFFSET, PANASONIC_AC_SWINGV_SIZE);
        SwingV::try_from(value)
    }

    /// Set the horizontal swing position
    pub fn set_swing_h(&mut self, position: SwingH) -> Result<()> {
        let position_value: u8 = position.into();

        // Different models support different horizontal swing settings
        let direction = match self.model {
            PanasonicAcModel::Dke | PanasonicAcModel::Rkr => position_value,
            PanasonicAcModel::Nke | PanasonicAcModel::Lke => {
                // These models only support middle position
                SwingH::Middle.into()
            }
            _ => {
                return Err(Error::UnsupportedFeature {
                    feature: "Horizontal swing".to_string(),
                    model: Some(self.model.to_string()),
                });
            }
        };

        self.buffer.set_bits(
            17,
            PANASONIC_AC_SWINGH_OFFSET,
            PANASONIC_AC_SWINGH_SIZE,
            direction,
        );

        Ok(())
    }

    /// Get the current horizontal swing position
    pub fn get_swing_h(&self) -> Result<SwingH> {
        let value = self
            .buffer
            .get_bits(17, PANASONIC_AC_SWINGH_OFFSET, PANASONIC_AC_SWINGH_SIZE);

        // For models that don't support horizontal swing, return Middle as default
        match self.model {
            PanasonicAcModel::Dke | PanasonicAcModel::Rkr => {
                // If the value is 0 (uninitialized) or otherwise invalid, return Middle as the default
                SwingH::try_from(value).or(Ok(SwingH::Middle))
            }
            _ => {
                // For models that don't support horizontal swing, always return Middle
                Ok(SwingH::Middle)
            }
        }
    }

    /// Set quiet mode
    pub fn set_quiet(&mut self, on: bool) -> Result<()> {
        // Quiet and Powerful are mutually exclusive
        if on {
            self.set_powerful(false)?;
        }

        let offset = match self.model {
            PanasonicAcModel::Rkr | PanasonicAcModel::Ckp => PANASONIC_AC_QUIET_CKP_OFFSET,
            _ => PANASONIC_AC_QUIET_OFFSET,
        };

        self.buffer.set_bit(21, offset, on);

        Ok(())
    }

    /// Get quiet mode status
    pub fn get_quiet(&self) -> bool {
        let offset = match self.model {
            PanasonicAcModel::Rkr | PanasonicAcModel::Ckp => PANASONIC_AC_QUIET_CKP_OFFSET,
            _ => PANASONIC_AC_QUIET_OFFSET,
        };

        self.buffer.get_bit(21, offset)
    }

    /// Set powerful mode
    pub fn set_powerful(&mut self, on: bool) -> Result<()> {
        // Quiet and Powerful are mutually exclusive
        if on {
            self.set_quiet(false)?;
        }

        let offset = match self.model {
            PanasonicAcModel::Rkr | PanasonicAcModel::Ckp => PANASONIC_AC_POWERFUL_CKP_OFFSET,
            _ => PANASONIC_AC_POWERFUL_OFFSET,
        };

        self.buffer.set_bit(21, offset, on);

        Ok(())
    }

    /// Get powerful mode status
    pub fn get_powerful(&self) -> bool {
        let offset = match self.model {
            PanasonicAcModel::Rkr | PanasonicAcModel::Ckp => PANASONIC_AC_POWERFUL_CKP_OFFSET,
            _ => PANASONIC_AC_POWERFUL_OFFSET,
        };

        self.buffer.get_bit(21, offset)
    }

    /// Set ion filter mode (if supported by the model)
    pub fn set_ion(&mut self, on: bool) -> Result<()> {
        if !self.model_supports_ion() {
            return Err(Error::UnsupportedFeature {
                feature: "Ion filter".to_string(),
                model: Some(self.model.to_string()),
            });
        }

        // Only DKE model supports ion
        self.buffer.set_bit(
            PANASONIC_AC_ION_FILTER_BYTE,
            PANASONIC_AC_ION_FILTER_OFFSET,
            on,
        );
        Ok(())
    }

    /// Get ion filter status
    pub fn get_ion(&self) -> bool {
        if !self.model_supports_ion() {
            return false;
        }

        self.buffer
            .get_bit(PANASONIC_AC_ION_FILTER_BYTE, PANASONIC_AC_ION_FILTER_OFFSET)
    }

    /// Set the clock time
    pub fn set_clock(&mut self, hours: u8, minutes: u8) -> Result<()> {
        let hours = min(hours, 23);
        let minutes = min(minutes, 59);
        let time = hours as u16 * 60 + minutes as u16;

        self.buffer[24] = time as u8;
        self.buffer.set_bits(25, 0, 3, (time >> 8) as u8);

        Ok(())
    }

    /// Get the clock time
    pub fn get_clock(&self) -> (u8, u8) {
        let time = self.buffer[24] as u16 | ((self.buffer[25] & 0x07) as u16) << 8;

        if time == PANASONIC_AC_TIME_SPECIAL {
            return (0, 0);
        }

        let hours = (time / 60) as u8;
        let minutes = (time % 60) as u8;

        (hours, minutes)
    }

    /// Set the On Timer
    pub fn set_on_timer(&mut self, hours: u8, minutes: u8, enable: bool) -> Result<()> {
        let hours = min(hours, 23);
        let minutes = min(minutes, 59);
        // Round down to nearest 10 minutes
        let minutes = minutes - (minutes % 10);
        let time = hours as u16 * 60 + minutes as u16;

        // Set the timer flag
        self.buffer
            .set_bit(13, PANASONIC_AC_ON_TIMER_OFFSET, enable);

        // Store the time
        self.buffer[18] = time as u8;
        self.buffer.set_bits(19, 0, 3, (time >> 8) as u8);

        Ok(())
    }

    /// Cancel the On Timer
    pub fn cancel_on_timer(&mut self) -> Result<()> {
        self.set_on_timer(0, 0, false)
    }

    /// Check if the On Timer is enabled
    pub fn is_on_timer_enabled(&self) -> bool {
        self.buffer.get_bit(13, PANASONIC_AC_ON_TIMER_OFFSET)
    }

    /// Get the On Timer time
    pub fn get_on_timer(&self) -> (u8, u8) {
        let time = self.buffer[18] as u16 | ((self.buffer[19] & 0x07) as u16) << 8;

        if time == PANASONIC_AC_TIME_SPECIAL {
            return (0, 0);
        }

        let hours = (time / 60) as u8;
        let minutes = (time % 60) as u8;

        (hours, minutes)
    }

    /// Set the Off Timer
    pub fn set_off_timer(&mut self, hours: u8, minutes: u8, enable: bool) -> Result<()> {
        let hours = min(hours, 23);
        let minutes = min(minutes, 59);
        // Round down to nearest 10 minutes
        let minutes = minutes - (minutes % 10);
        let time = hours as u16 * 60 + minutes as u16;

        // Set the timer flag
        self.buffer
            .set_bit(13, PANASONIC_AC_OFF_TIMER_OFFSET, enable);

        // Store the time (different format than On Timer)
        self.buffer.set_bits(19, 4, 4, (time & 0x0F) as u8);
        self.buffer.set_bits(20, 0, 7, (time >> 4) as u8);

        Ok(())
    }

    /// Cancel the Off Timer
    pub fn cancel_off_timer(&mut self) -> Result<()> {
        self.set_off_timer(0, 0, false)
    }

    /// Check if the Off Timer is enabled
    pub fn is_off_timer_enabled(&self) -> bool {
        self.buffer.get_bit(13, PANASONIC_AC_OFF_TIMER_OFFSET)
    }

    /// Get the Off Timer time
    pub fn get_off_timer(&self) -> (u8, u8) {
        let time = ((self.buffer[19] >> 4) as u16) | ((self.buffer[20] & 0x7F) as u16) << 4;

        if time == PANASONIC_AC_TIME_SPECIAL {
            return (0, 0);
        }

        let hours = (time / 60) as u8;
        let minutes = (time % 60) as u8;

        (hours, minutes)
    }
}

impl fmt::Display for PanasonicAcState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let model = self.get_model();
        let power = if self.get_power() { "On" } else { "Off" };

        let mode = match self.get_mode() {
            Ok(mode) => format!("{}", mode),
            Err(_) => "Unknown".to_string(),
        };

        let temp = self.get_temp();

        let fan = match self.get_fan() {
            Ok(fan) => format!("{}", fan),
            Err(_) => "Unknown".to_string(),
        };

        let swing_v = match self.get_swing_v() {
            Ok(swing) => format!("{}", swing),
            Err(_) => "Unknown".to_string(),
        };

        let swing_h = match self.get_swing_h() {
            Ok(swing) => format!("{}", swing),
            Err(_) => "Unknown".to_string(),
        };

        let quiet = if self.get_quiet() { "On" } else { "Off" };
        let powerful = if self.get_powerful() { "On" } else { "Off" };
        let ion = if self.get_ion() { "On" } else { "Off" };

        let (clock_h, clock_m) = self.get_clock();
        let clock = format!("{:02}:{:02}", clock_h, clock_m);

        let on_timer_status = if self.is_on_timer_enabled() {
            let (h, m) = self.get_on_timer();
            format!("{:02}:{:02}", h, m)
        } else {
            "Off".to_string()
        };

        let off_timer_status = if self.is_off_timer_enabled() {
            let (h, m) = self.get_off_timer();
            format!("{:02}:{:02}", h, m)
        } else {
            "Off".to_string()
        };

        write!(
            f,
            "Model: {}, Power: {}, Mode: {}, Temp: {}°C, Fan: {}, Swing V: {}, Swing H: {}, \
             Quiet: {}, Powerful: {}, Ion: {}, Clock: {}, On Timer: {}, Off Timer: {}",
            model,
            power,
            mode,
            temp,
            fan,
            swing_v,
            swing_h,
            quiet,
            powerful,
            ion,
            clock,
            on_timer_status,
            off_timer_status
        )
    }
}

/// State for AC32 models (simplified implementation)
#[derive(Clone, Debug)]
pub struct PanasonicAc32State {
    /// Raw state as a 32-bit value
    buffer: u32,
}

impl Default for PanasonicAc32State {
    fn default() -> Self {
        Self {
            buffer: PANASONIC_AC32_KNOWN_GOOD,
        }
    }
}

impl PanasonicAc32State {
    /// Create a new AC32 state with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the raw buffer value
    pub fn get_buffer(&self) -> u32 {
        self.buffer
    }

    /// Get the buffer as a byte array
    pub fn get_bytes(&self) -> [u8; 4] {
        self.buffer.to_be_bytes()
    }

    /// Set the buffer from a byte array
    pub fn set_from_bytes(&mut self, data: &[u8]) -> Result<()> {
        if data.len() != 4 {
            return Err(Error::InvalidData(format!(
                "Expected 4 bytes, got {}",
                data.len()
            )));
        }

        // Convert bytes to u32
        let mut buffer = 0u32;
        for (i, &byte) in data.iter().enumerate() {
            buffer |= (byte as u32) << ((3 - i) * 8);
        }

        self.buffer = buffer;

        Ok(())
    }

    /// Set power toggle state
    pub fn set_power_toggle(&mut self, on: bool) -> Result<()> {
        // Power toggle bit in AC32 models
        if on {
            self.buffer &= !(1 << 24);
        } else {
            self.buffer |= 1 << 24;
        }
        Ok(())
    }

    /// Get power toggle state
    pub fn get_power_toggle(&self) -> bool {
        (self.buffer & (1 << 24)) == 0
    }

    /// Set the operating mode
    pub fn set_mode(&mut self, mode: AcMode) -> Result<()> {
        // AC32 uses different mode values
        let mode_value = match mode {
            AcMode::Auto => PANASONIC_AC32_AUTO,
            AcMode::Cool => PANASONIC_AC32_COOL,
            AcMode::Heat => PANASONIC_AC32_HEAT,
            AcMode::Dry => PANASONIC_AC32_DRY,
            AcMode::Fan => PANASONIC_AC32_FAN,
        };

        // Clear bits 8-10 and set new mode
        self.buffer &= !(0x7 << 8);
        self.buffer |= (mode_value as u32) << 8;

        Ok(())
    }

    /// Get the current operating mode
    pub fn get_mode(&self) -> Result<AcMode> {
        let mode = ((self.buffer >> 8) & 0x7) as u8;

        match mode {
            PANASONIC_AC32_AUTO => Ok(AcMode::Auto),
            PANASONIC_AC32_COOL => Ok(AcMode::Cool),
            PANASONIC_AC32_HEAT => Ok(AcMode::Heat),
            PANASONIC_AC32_DRY => Ok(AcMode::Dry),
            PANASONIC_AC32_FAN => Ok(AcMode::Fan),
            _ => Err(Error::InvalidValue(format!("Invalid mode: {}", mode))),
        }
    }

    /// Set the temperature in Celsius
    pub fn set_temp(&mut self, celsius: u8) -> Result<()> {
        let temp = max(celsius, PANASONIC_AC32_MIN_TEMP);
        let temp = min(temp, PANASONIC_AC32_MAX_TEMP);

        // Clear bits 16-20 and set new temperature
        self.buffer &= !(0x1F << 16);
        self.buffer |= ((temp - (PANASONIC_AC32_MIN_TEMP - 1)) as u32) << 16;

        Ok(())
    }

    /// Get the current temperature setting in Celsius
    pub fn get_temp(&self) -> u8 {
        ((self.buffer >> 16) & 0x1F) as u8 + (PANASONIC_AC32_MIN_TEMP - 1)
    }
}
