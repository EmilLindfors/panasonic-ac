// device.rs
//! Panasonic AC device implementation

use std::fmt;

use crate::constants::PANASONIC_FREQ;
use crate::error::{Error, Result};
use crate::protocol::{AcDevice, Device};
use crate::state::{PanasonicAc32State, PanasonicAcState};
use crate::types::{AcMode, FanSpeed, SwingH, SwingV};

/// Represents different Panasonic AC models
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanasonicAcModel {
    /// DKE model series
    Dke,
    /// JKE model series
    Jke,
    /// LKE model series
    Lke,
    /// NKE model series
    Nke,
    /// CKP model series
    Ckp,
    /// RKR model series
    Rkr,
    /// Unknown model
    Unknown,
}

impl fmt::Display for PanasonicAcModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dke => write!(f, "DKE"),
            Self::Jke => write!(f, "JKE"),
            Self::Lke => write!(f, "LKE"),
            Self::Nke => write!(f, "NKE"),
            Self::Ckp => write!(f, "CKP"),
            Self::Rkr => write!(f, "RKR"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Function signature for IR sender callback
pub type IrSendFn = Box<dyn Fn(&[u8], u32, u16) -> Result<()>>;

/// Represents a Panasonic AC device
pub struct PanasonicAc {
    /// Internal state
    state: PanasonicAcState,
    /// Function to send IR signals
    ir_send: IrSendFn,
}

impl PanasonicAc {
    /// Create a new `PanasonicAc` instance
    ///
    /// # Arguments
    /// * `ir_send_fn` - Function that will be called to send IR signals
    ///
    /// # Returns
    /// * A new `PanasonicAc` instance
    pub fn new<F>(ir_send_fn: F) -> Self
    where
        F: Fn(&[u8], u32, u16) -> Result<()> + 'static,
    {
        // Create a new instance with default state
        Self {
            state: PanasonicAcState::new(),
            ir_send: Box::new(ir_send_fn),
        }
    }

    /// Create a new builder for configuring a `PanasonicAc` instance
    ///
    /// # Returns
    /// * A builder for fluent configuration
    ///
    /// # Example
    /// ```
    /// use panasonic_ac::prelude::*;
    ///
    /// let ac = PanasonicAc::builder()
    ///     .model(PanasonicAcModel::Dke)
    ///     .power(true)
    ///     .mode(AcMode::Cool)
    ///     .temp(23)
    ///     .fan(FanSpeed::Auto)
    ///     .build(|data, freq, repeat| {
    ///         // Your IR transmission implementation
    ///         Ok(())
    ///     })
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn builder() -> PanasonicAcBuilder {
        PanasonicAcBuilder::new()
    }

    /// Reset the state to known good values
    pub fn reset(&mut self) {
        self.state = PanasonicAcState::default();
    }

    /// Apply changes to the AC and return whether any changes were made
    ///
    /// This method allows efficient updates by only changing the specified settings.
    /// It returns a boolean indicating whether any changes were actually made.
    ///
    /// # Arguments
    /// * `changes` - The changes to apply
    ///
    /// # Returns
    /// * `Ok(true)` if any changes were made, `Ok(false)` if no changes were needed
    ///
    /// # Errors
    /// * `Error::UnsupportedFeature` - If a feature is not supported by the current model
    /// * `Error::InvalidValue` - If a value is invalid for its setting (e.g., temperature out of range)
    ///
    /// # Example
    /// ```
    /// use panasonic_ac::prelude::*;
    ///
    /// let mut ac = PanasonicAc::new(|_, _, _| Ok(()));
    /// let changes = PanasonicAcChanges {
    ///     power: Some(true),
    ///     temp: Some(23),
    ///     ..Default::default()
    /// };
    ///
    /// // Apply only these changes, leaving other settings untouched
    /// let changed = ac.apply_changes(&changes).unwrap();
    /// if changed {
    ///     // Send the command if changes were made
    ///     ac.send().unwrap();
    /// }
    /// ```
    pub fn apply_changes(&mut self, changes: &PanasonicAcChanges) -> Result<bool> {
        // Save original state to compare later
        let original_state = self.state.clone();

        // Apply changes only if the corresponding field is Some
        if let Some(power) = changes.power {
            if self.get_power() != power {
                self.set_power(power)?;
            }
        }

        if let Some(mode) = changes.mode {
            if self.get_mode().map_or(true, |current| current != mode) {
                self.set_mode(mode)?;
            }
        }

        if let Some(temp) = changes.temp {
            if self.get_temp() != temp {
                self.set_temp(temp)?;
            }
        }

        if let Some(fan) = changes.fan {
            if self.get_fan().map_or(true, |current| current != fan) {
                self.set_fan(fan)?;
            }
        }

        if let Some(swing_v) = changes.swing_v {
            if self
                .get_swing_v()
                .map_or(true, |current| current != swing_v)
            {
                self.set_swing_v(swing_v)?;
            }
        }

        if let Some(swing_h) = changes.swing_h {
            if self
                .get_swing_h()
                .map_or(true, |current| current != swing_h)
            {
                self.set_swing_h(swing_h)?;
            }
        }

        if let Some(quiet) = changes.quiet {
            if self.get_quiet() != quiet {
                self.set_quiet(quiet)?;
            }
        }

        if let Some(powerful) = changes.powerful {
            if self.get_powerful() != powerful {
                self.set_powerful(powerful)?;
            }
        }

        if let Some(ion) = changes.ion {
            if self.get_ion() != ion {
                self.set_ion(ion)?;
            }
        }

        // Check if any changes were made by comparing buffer contents
        let changed = original_state.get_buffer() != self.state.get_buffer();

        Ok(changed)
    }

    /// Apply changes and send the IR command if any changes were made
    ///
    /// This is a convenience method that applies changes and automatically
    /// sends the IR command only if any settings were actually changed.
    ///
    /// # Arguments
    /// * `changes` - The changes to apply
    ///
    /// # Returns
    /// * `Ok(true)` if changes were made and the command was sent
    /// * `Ok(false)` if no changes were needed (command not sent)
    ///
    /// # Errors
    /// * `Error::UnsupportedFeature` - If a feature is not supported by the current model
    /// * `Error::InvalidValue` - If a value is invalid for its setting
    /// * `Error::SendError` - If there was an error sending the IR command
    pub fn apply_changes_and_send(&mut self, changes: &PanasonicAcChanges) -> Result<bool> {
        let changed = self.apply_changes(changes)?;

        if changed {
            self.send()?;
        }

        Ok(changed)
    }

    /// Set the model of the AC
    ///
    /// # Arguments
    /// * `model` - The model to set
    ///
    /// # Returns
    /// * Ok(()) if successful
    ///
    /// # Errors
    /// * `Error::InvalidValue` - If the model is Unknown
    pub fn set_model(&mut self, model: PanasonicAcModel) -> Result<()> {
        self.state.set_model(model)
    }

    /// Get the model of the AC
    ///
    /// # Returns
    /// * The detected model
    #[must_use]
    pub fn get_model(&self) -> PanasonicAcModel {
        self.state.get_model()
    }

    /// Check if the current model supports ion filter functionality
    #[must_use]
    pub fn supports_ion(&self) -> bool {
        self.state.model_supports_ion()
    }

    /// Check if the current model supports horizontal swing functionality
    #[must_use]
    pub fn supports_horizontal_swing(&self) -> bool {
        self.state.model_supports_horizontal_swing()
    }

    /// Set the clock time
    ///
    /// # Arguments
    /// * `hours` - Hours (0-23)
    /// * `minutes` - Minutes (0-59)
    ///
    /// # Returns
    /// * Ok(()) if successful, Err otherwise
    pub fn set_clock(&mut self, hours: u8, minutes: u8) -> Result<()> {
        self.state.set_clock(hours, minutes)
    }

    /// Get the clock time
    ///
    /// # Returns
    /// * (hours, minutes) tuple
    #[must_use]
    pub fn get_clock(&self) -> (u8, u8) {
        self.state.get_clock()
    }

    /// Set the On Timer
    ///
    /// # Arguments
    /// * `hours` - Hours (0-23)
    /// * `minutes` - Minutes (0-59)
    /// * `enable` - true to enable the timer, false to disable
    ///
    /// # Returns
    /// * Ok(()) if successful, Err otherwise
    pub fn set_on_timer(&mut self, hours: u8, minutes: u8, enable: bool) -> Result<()> {
        self.state.set_on_timer(hours, minutes, enable)
    }

    /// Cancel the On Timer
    ///
    /// # Returns
    /// * Ok(()) if successful, Err otherwise
    pub fn cancel_on_timer(&mut self) -> Result<()> {
        self.state.cancel_on_timer()
    }

    /// Check if the On Timer is enabled
    ///
    /// # Returns
    /// * true if enabled, false otherwise
    #[must_use]
    pub fn is_on_timer_enabled(&self) -> bool {
        self.state.is_on_timer_enabled()
    }

    /// Get the On Timer time
    ///
    /// # Returns
    /// * (hours, minutes) tuple
    #[must_use]
    pub fn get_on_timer(&self) -> (u8, u8) {
        self.state.get_on_timer()
    }

    /// Set the Off Timer
    ///
    /// # Arguments
    /// * `hours` - Hours (0-23)
    /// * `minutes` - Minutes (0-59)
    /// * `enable` - true to enable the timer, false to disable
    ///
    /// # Returns
    /// * Ok(()) if successful, Err otherwise
    pub fn set_off_timer(&mut self, hours: u8, minutes: u8, enable: bool) -> Result<()> {
        self.state.set_off_timer(hours, minutes, enable)
    }

    /// Cancel the Off Timer
    ///
    /// # Returns
    /// * Ok(()) if successful, Err otherwise
    pub fn cancel_off_timer(&mut self) -> Result<()> {
        self.state.cancel_off_timer()
    }

    /// Check if the Off Timer is enabled
    ///
    /// # Returns
    /// * true if enabled, false otherwise
    #[must_use]
    pub fn is_off_timer_enabled(&self) -> bool {
        self.state.is_off_timer_enabled()
    }

    /// Get the Off Timer time
    ///
    /// # Returns
    /// * (hours, minutes) tuple
    #[must_use]
    pub fn get_off_timer(&self) -> (u8, u8) {
        self.state.get_off_timer()
    }
}

impl Device for PanasonicAc {
    fn set_power(&mut self, on: bool) -> Result<()> {
        self.state.set_power(on)
    }

    fn get_power(&self) -> bool {
        self.state.get_power()
    }

    fn send(&self) -> Result<()> {
        // Get a copy of the state buffer with a fixed checksum
        let state = self.state.to_vec();

        // Send the IR signal
        (self.ir_send)(&state, PANASONIC_FREQ, 0)
    }

    fn get_raw(&self) -> Vec<u8> {
        self.state.to_vec()
    }

    fn set_raw(&mut self, data: &[u8]) -> Result<()> {
        self.state.from_raw_data(data)
    }
}

impl AcDevice for PanasonicAc {
    fn set_mode(&mut self, mode: AcMode) -> Result<()> {
        self.state.set_mode(mode)
    }

    fn get_mode(&self) -> Result<AcMode> {
        self.state.get_mode()
    }

    fn set_temp(&mut self, celsius: u8) -> Result<()> {
        self.state.set_temp(celsius)
    }

    fn get_temp(&self) -> u8 {
        self.state.get_temp()
    }

    fn set_fan(&mut self, speed: FanSpeed) -> Result<()> {
        self.state.set_fan(speed)
    }

    fn get_fan(&self) -> Result<FanSpeed> {
        self.state.get_fan()
    }

    fn set_swing_v(&mut self, position: SwingV) -> Result<()> {
        self.state.set_swing_v(position)
    }

    fn get_swing_v(&self) -> Result<SwingV> {
        self.state.get_swing_v()
    }

    fn set_swing_h(&mut self, position: SwingH) -> Result<()> {
        self.state.set_swing_h(position)
    }

    fn get_swing_h(&self) -> Result<SwingH> {
        self.state.get_swing_h()
    }

    fn set_quiet(&mut self, on: bool) -> Result<()> {
        self.state.set_quiet(on)
    }

    fn get_quiet(&self) -> bool {
        self.state.get_quiet()
    }

    fn set_powerful(&mut self, on: bool) -> Result<()> {
        self.state.set_powerful(on)
    }

    fn get_powerful(&self) -> bool {
        self.state.get_powerful()
    }

    fn set_ion(&mut self, on: bool) -> Result<()> {
        self.state.set_ion(on)
    }

    fn get_ion(&self) -> bool {
        self.state.get_ion()
    }
}

impl fmt::Display for PanasonicAc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Forward to the state's Display implementation
        write!(f, "{}", self.state)
    }
}

/// For AC32 models (simplified implementation)
pub struct PanasonicAc32 {
    /// Internal state
    state: PanasonicAc32State,
    /// Function to send IR signals
    ir_send: IrSendFn,
}

impl PanasonicAc32 {
    /// Create a new `PanasonicAc32` instance
    pub fn new<F>(ir_send_fn: F) -> Self
    where
        F: Fn(&[u8], u32, u16) -> Result<()> + 'static,
    {
        Self {
            state: PanasonicAc32State::new(),
            ir_send: Box::new(ir_send_fn),
        }
    }

    /// Reset state to defaults
    pub fn reset(&mut self) {
        self.state = PanasonicAc32State::default();
    }

    /// Set power toggle state
    pub fn set_power_toggle(&mut self, on: bool) -> Result<()> {
        self.state.set_power_toggle(on)
    }

    /// Get power toggle state
    pub fn get_power_toggle(&self) -> bool {
        self.state.get_power_toggle()
    }
}

impl Device for PanasonicAc32 {
    fn set_power(&mut self, on: bool) -> Result<()> {
        // For AC32, this toggles the power state rather than setting it directly
        self.set_power_toggle(on)
    }

    fn get_power(&self) -> bool {
        self.get_power_toggle()
    }

    fn send(&self) -> Result<()> {
        // Convert u32 to bytes
        let bytes = self.state.get_bytes();

        // Send the IR signal
        (self.ir_send)(&bytes, PANASONIC_FREQ, 0)
    }

    fn get_raw(&self) -> Vec<u8> {
        self.state.get_bytes().to_vec()
    }

    fn set_raw(&mut self, data: &[u8]) -> Result<()> {
        self.state.set_from_bytes(data)
    }
}

impl AcDevice for PanasonicAc32 {
    fn set_mode(&mut self, mode: AcMode) -> Result<()> {
        self.state.set_mode(mode)
    }

    fn get_mode(&self) -> Result<AcMode> {
        self.state.get_mode()
    }

    fn set_temp(&mut self, celsius: u8) -> Result<()> {
        self.state.set_temp(celsius)
    }

    fn get_temp(&self) -> u8 {
        self.state.get_temp()
    }

    // Default implementations for unsupported functions

    fn set_fan(&mut self, _speed: FanSpeed) -> Result<()> {
        Err(Error::UnsupportedFeature {
            feature: "Fan speed control".to_string(),
            model: Some("AC32".to_string()),
        })
    }

    fn get_fan(&self) -> Result<FanSpeed> {
        Err(Error::UnsupportedFeature {
            feature: "Fan speed control".to_string(),
            model: Some("AC32".to_string()),
        })
    }

    fn set_swing_v(&mut self, _position: SwingV) -> Result<()> {
        Err(Error::UnsupportedFeature {
            feature: "Vertical swing".to_string(),
            model: Some("AC32".to_string()),
        })
    }

    fn get_swing_v(&self) -> Result<SwingV> {
        Err(Error::UnsupportedFeature {
            feature: "Vertical swing".to_string(),
            model: Some("AC32".to_string()),
        })
    }

    fn set_swing_h(&mut self, _position: SwingH) -> Result<()> {
        Err(Error::UnsupportedFeature {
            feature: "Horizontal swing".to_string(),
            model: Some("AC32".to_string()),
        })
    }

    fn get_swing_h(&self) -> Result<SwingH> {
        Err(Error::UnsupportedFeature {
            feature: "Horizontal swing".to_string(),
            model: Some("AC32".to_string()),
        })
    }

    fn set_quiet(&mut self, _on: bool) -> Result<()> {
        Err(Error::UnsupportedFeature {
            feature: "Quiet mode".to_string(),
            model: Some("AC32".to_string()),
        })
    }

    fn get_quiet(&self) -> bool {
        false
    }

    fn set_powerful(&mut self, _on: bool) -> Result<()> {
        Err(Error::UnsupportedFeature {
            feature: "Powerful mode".to_string(),
            model: Some("AC32".to_string()),
        })
    }

    fn get_powerful(&self) -> bool {
        false
    }

    fn set_ion(&mut self, _on: bool) -> Result<()> {
        Err(Error::UnsupportedFeature {
            feature: "Ion filter".to_string(),
            model: Some("AC32".to_string()),
        })
    }

    fn get_ion(&self) -> bool {
        false
    }
}

/// Builder for creating and configuring a PanasonicAc instance
#[derive(Debug, Default)]
pub struct PanasonicAcBuilder {
    /// The model of the AC unit
    model: Option<PanasonicAcModel>,
    /// Power state (on/off)
    power: Option<bool>,
    /// Operating mode
    mode: Option<AcMode>,
    /// Temperature in Celsius
    temp: Option<u8>,
    /// Fan speed
    fan: Option<FanSpeed>,
    /// Vertical swing position
    swing_v: Option<SwingV>,
    /// Horizontal swing position
    swing_h: Option<SwingH>,
    /// Quiet mode
    quiet: Option<bool>,
    /// Powerful mode
    powerful: Option<bool>,
    /// Ion filter (only available on some models)
    ion: Option<bool>,
}

impl PanasonicAcBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the AC model
    pub fn model(mut self, model: PanasonicAcModel) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the power state
    pub fn power(mut self, on: bool) -> Self {
        self.power = Some(on);
        self
    }

    /// Set the operating mode
    pub fn mode(mut self, mode: AcMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Set the temperature in Celsius
    pub fn temp(mut self, celsius: u8) -> Self {
        self.temp = Some(celsius);
        self
    }

    /// Set the fan speed
    pub fn fan(mut self, speed: FanSpeed) -> Self {
        self.fan = Some(speed);
        self
    }

    /// Set the vertical swing position
    pub fn swing_v(mut self, position: SwingV) -> Self {
        self.swing_v = Some(position);
        self
    }

    /// Set the horizontal swing position
    pub fn swing_h(mut self, position: SwingH) -> Self {
        self.swing_h = Some(position);
        self
    }

    /// Set quiet mode
    pub fn quiet(mut self, on: bool) -> Self {
        self.quiet = Some(on);
        self
    }

    /// Set powerful mode
    pub fn powerful(mut self, on: bool) -> Self {
        self.powerful = Some(on);
        self
    }

    /// Set ion filter mode
    pub fn ion(mut self, on: bool) -> Self {
        self.ion = Some(on);
        self
    }

    /// Build the PanasonicAc instance with the configured settings
    pub fn build<F>(self, ir_send_fn: F) -> Result<PanasonicAc>
    where
        F: Fn(&[u8], u32, u16) -> Result<()> + 'static,
    {
        let mut ac = PanasonicAc::new(ir_send_fn);

        // Apply settings if provided
        if let Some(model) = self.model {
            ac.set_model(model)?;
        }

        if let Some(power) = self.power {
            ac.set_power(power)?;
        }

        if let Some(mode) = self.mode {
            ac.set_mode(mode)?;
        }

        if let Some(temp) = self.temp {
            ac.set_temp(temp)?;
        }

        if let Some(fan) = self.fan {
            ac.set_fan(fan)?;
        }

        if let Some(swing_v) = self.swing_v {
            ac.set_swing_v(swing_v)?;
        }

        if let Some(swing_h) = self.swing_h {
            ac.set_swing_h(swing_h)?;
        }

        if let Some(quiet) = self.quiet {
            ac.set_quiet(quiet)?;
        }

        if let Some(powerful) = self.powerful {
            ac.set_powerful(powerful)?;
        }

        if let Some(ion) = self.ion {
            ac.set_ion(ion)?;
        }

        Ok(ac)
    }

    /// Build the PanasonicAc instance and turn it on with the configured settings
    pub fn build_and_turn_on<F>(self, ir_send_fn: F) -> Result<PanasonicAc>
    where
        F: Fn(&[u8], u32, u16) -> Result<()> + 'static,
    {
        let mut builder = self;
        builder.power = Some(true);
        builder.build(ir_send_fn)
    }
}

/// Changes to apply to a PanasonicAc instance
#[derive(Debug, Default)]
pub struct PanasonicAcChanges {
    /// Power state change (on/off)
    pub power: Option<bool>,
    /// Operating mode change
    pub mode: Option<AcMode>,
    /// Temperature change in Celsius
    pub temp: Option<u8>,
    /// Fan speed change
    pub fan: Option<FanSpeed>,
    /// Vertical swing position change
    pub swing_v: Option<SwingV>,
    /// Horizontal swing position change
    pub swing_h: Option<SwingH>,
    /// Quiet mode change
    pub quiet: Option<bool>,
    /// Powerful mode change
    pub powerful: Option<bool>,
    /// Ion filter change
    pub ion: Option<bool>,
}
