// utils/standard.rs
//! Standard device interface and utilities

use std::fmt;

use crate::device::PanasonicAcModel;
use crate::error::Result;
use crate::protocol::{AcDevice, Device};
use crate::types::{AcMode, FanSpeed, SwingH, SwingV};
use crate::PanasonicAc;

/// Standard operation mode types that map to common AC control systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpMode {
    /// Automatic mode
    Auto,
    /// Cooling mode
    Cool,
    /// Heating mode
    Heat,
    /// Dehumidification (dry) mode
    Dry,
    /// Fan only mode
    Fan,
}

impl From<AcMode> for OpMode {
    fn from(mode: AcMode) -> Self {
        match mode {
            AcMode::Auto => OpMode::Auto,
            AcMode::Cool => OpMode::Cool,
            AcMode::Heat => OpMode::Heat,
            AcMode::Dry => OpMode::Dry,
            AcMode::Fan => OpMode::Fan,
        }
    }
}

impl From<OpMode> for AcMode {
    fn from(mode: OpMode) -> Self {
        match mode {
            OpMode::Auto => AcMode::Auto,
            OpMode::Cool => AcMode::Cool,
            OpMode::Heat => AcMode::Heat,
            OpMode::Dry => AcMode::Dry,
            OpMode::Fan => AcMode::Fan,
        }
    }
}

/// Fan speed levels that map to common AC control systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanLevel {
    /// Automatic fan speed
    Auto,
    /// Minimum fan speed
    Min,
    /// Low fan speed
    Low,
    /// Medium fan speed
    Medium,
    /// High fan speed
    High,
    /// Maximum fan speed
    Max,
}

impl From<FanSpeed> for FanLevel {
    fn from(speed: FanSpeed) -> Self {
        match speed {
            FanSpeed::Auto => FanLevel::Auto,
            FanSpeed::Min => FanLevel::Min,
            FanSpeed::Low => FanLevel::Low,
            FanSpeed::Medium => FanLevel::Medium,
            FanSpeed::High => FanLevel::High,
            FanSpeed::Max => FanLevel::Max,
        }
    }
}

impl From<FanLevel> for FanSpeed {
    fn from(level: FanLevel) -> Self {
        match level {
            FanLevel::Auto => FanSpeed::Auto,
            FanLevel::Min => FanSpeed::Min,
            FanLevel::Low => FanSpeed::Low,
            FanLevel::Medium => FanSpeed::Medium,
            FanLevel::High => FanSpeed::High,
            FanLevel::Max => FanSpeed::Max,
        }
    }
}

/// Vertical swing positions that map to common AC control systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwingVPos {
    /// Automatic swing
    Auto,
    /// Highest position
    Highest,
    /// High position
    High,
    /// Middle position
    Middle,
    /// Low position
    Low,
    /// Lowest position
    Lowest,
}

impl From<SwingV> for SwingVPos {
    fn from(swing: SwingV) -> Self {
        match swing {
            SwingV::Auto => SwingVPos::Auto,
            SwingV::Highest => SwingVPos::Highest,
            SwingV::High => SwingVPos::High,
            SwingV::Middle => SwingVPos::Middle,
            SwingV::Low => SwingVPos::Low,
            SwingV::Lowest => SwingVPos::Lowest,
        }
    }
}

impl From<SwingVPos> for SwingV {
    fn from(pos: SwingVPos) -> Self {
        match pos {
            SwingVPos::Auto => SwingV::Auto,
            SwingVPos::Highest => SwingV::Highest,
            SwingVPos::High => SwingV::High,
            SwingVPos::Middle => SwingV::Middle,
            SwingVPos::Low => SwingV::Low,
            SwingVPos::Lowest => SwingV::Lowest,
        }
    }
}

/// Horizontal swing positions that map to common AC control systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwingHPos {
    /// Automatic swing
    Auto,
    /// Far left position
    LeftMax,
    /// Left position
    Left,
    /// Middle position
    Middle,
    /// Right position
    Right,
    /// Far right position
    RightMax,
}

impl From<SwingH> for SwingHPos {
    fn from(swing: SwingH) -> Self {
        match swing {
            SwingH::Auto => SwingHPos::Auto,
            SwingH::FullLeft => SwingHPos::LeftMax,
            SwingH::Left => SwingHPos::Left,
            SwingH::Middle => SwingHPos::Middle,
            SwingH::Right => SwingHPos::Right,
            SwingH::FullRight => SwingHPos::RightMax,
        }
    }
}

impl From<SwingHPos> for SwingH {
    fn from(pos: SwingHPos) -> Self {
        match pos {
            SwingHPos::Auto => SwingH::Auto,
            SwingHPos::LeftMax => SwingH::FullLeft,
            SwingHPos::Left => SwingH::Left,
            SwingHPos::Middle => SwingH::Middle,
            SwingHPos::Right => SwingH::Right,
            SwingHPos::RightMax => SwingH::FullRight,
        }
    }
}

/// Standard state representation for any AC device
#[derive(Debug, Clone)]
pub struct State {
    /// Power status (on/off)
    pub power: bool,
    /// Operation mode
    pub mode: OpMode,
    /// Temperature in Celsius
    pub temp: u8,
    /// Fan speed
    pub fan: FanLevel,
    /// Vertical swing position
    pub swing_v: Option<SwingVPos>,
    /// Horizontal swing position
    pub swing_h: Option<SwingHPos>,
    /// Quiet mode
    pub quiet: bool,
    /// Powerful/turbo mode
    pub powerful: bool,
    /// Ion/filter mode
    pub ion: bool,
    /// Model type (if known)
    pub model: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            power: false,
            mode: OpMode::Auto,
            temp: 25,
            fan: FanLevel::Auto,
            swing_v: Some(SwingVPos::Auto),
            swing_h: Some(SwingHPos::Auto),
            quiet: false,
            powerful: false,
            ion: false,
            model: None,
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let power = if self.power { "On" } else { "Off" };
        let mode = match self.mode {
            OpMode::Auto => "Auto",
            OpMode::Cool => "Cool",
            OpMode::Heat => "Heat",
            OpMode::Dry => "Dry",
            OpMode::Fan => "Fan",
        };

        let fan = match self.fan {
            FanLevel::Auto => "Auto",
            FanLevel::Min => "Minimum",
            FanLevel::Low => "Low",
            FanLevel::Medium => "Medium",
            FanLevel::High => "High",
            FanLevel::Max => "Maximum",
        };

        let swing_v = if let Some(pos) = self.swing_v {
            match pos {
                SwingVPos::Auto => "Auto",
                SwingVPos::Highest => "Highest",
                SwingVPos::High => "High",
                SwingVPos::Middle => "Middle",
                SwingVPos::Low => "Low",
                SwingVPos::Lowest => "Lowest",
            }
        } else {
            "N/A"
        };

        let swing_h = if let Some(pos) = self.swing_h {
            match pos {
                SwingHPos::Auto => "Auto",
                SwingHPos::LeftMax => "Far Left",
                SwingHPos::Left => "Left",
                SwingHPos::Middle => "Middle",
                SwingHPos::Right => "Right",
                SwingHPos::RightMax => "Far Right",
            }
        } else {
            "N/A"
        };

        let quiet = if self.quiet { "On" } else { "Off" };
        let powerful = if self.powerful { "On" } else { "Off" };
        let ion = if self.ion { "On" } else { "Off" };
        let model = self.model.as_deref().unwrap_or("Unknown");

        write!(
            f,
            "Power: {}, Mode: {}, Temp: {}°C, Fan: {}, Swing V: {}, Swing H: {}, \
             Quiet: {}, Powerful: {}, Ion: {}, Model: {}",
            power, mode, self.temp, fan, swing_v, swing_h, quiet, powerful, ion, model
        )
    }
}

/// Trait representing a standardized AC device interface
pub trait StandardAc {
    /// Get the current state of the device
    fn get_state(&self) -> Result<State>;

    /// Set the state of the device
    fn set_state(&mut self, state: &State) -> Result<()>;

    /// Apply changes to the current state
    ///
    /// This method allows partial state updates, only modifying the fields
    /// that are specified in the provided closure.
    ///
    /// # Arguments
    /// * `updater` - A closure that takes a mutable reference to the current state
    ///   and modifies it as needed
    ///
    /// # Returns
    /// * `Ok(())` if the update was successful, `Err` otherwise
    fn update_state<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut State);

    /// Send the current state to the device
    fn send(&self) -> Result<()>;
}

/// Implementation of StandardAc for any device implementing AcDevice
impl<T: AcDevice + 'static> StandardAc for T {
    fn get_state(&self) -> Result<State> {
        let mut state = State {
            power: self.get_power(),
            ..State::default()
        };

        // Handle potential errors for each property
        if let Ok(mode) = self.get_mode() {
            state.mode = mode.into();
        }

        state.temp = self.get_temp();

        if let Ok(fan) = self.get_fan() {
            state.fan = fan.into();
        }

        if let Ok(swing_v) = self.get_swing_v() {
            state.swing_v = Some(swing_v.into());
        } else {
            state.swing_v = None;
        }

        if let Ok(swing_h) = self.get_swing_h() {
            state.swing_h = Some(swing_h.into());
        } else {
            state.swing_h = None;
        }

        state.quiet = self.get_quiet();
        state.powerful = self.get_powerful();
        state.ion = self.get_ion();

        // Include model information if available
        if let Some(ac) = (self as &dyn std::any::Any).downcast_ref::<PanasonicAc>() {
            state.model = Some(crate::utils::standard::model_to_string(ac.get_model()));
        }

        Ok(state)
    }

    fn set_state(&mut self, state: &State) -> Result<()> {
        // Start with power off to avoid sending unnecessary commands
        self.set_power(false)?;

        // Configure all settings
        self.set_mode(state.mode.into())?;

        // Ensure temperature is set correctly
        // Important: set_temp after set_mode since set_mode might reset temp
        self.set_temp(state.temp)?;

        if let Ok(fan) = FanSpeed::try_from(state.fan as u8) {
            self.set_fan(fan)?;
        }

        if let Some(swing_v) = state.swing_v {
            self.set_swing_v(swing_v.into())?;
        }

        if let Some(swing_h) = state.swing_h {
            self.set_swing_h(swing_h.into())?;
        }

        self.set_quiet(state.quiet)?;
        self.set_powerful(state.powerful)?;
        self.set_ion(state.ion)?;

        // Finally, set the power state
        self.set_power(state.power)?;

        Ok(())
    }

    fn update_state<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut State),
    {
        let mut current = self.get_state()?;
        updater(&mut current);
        self.set_state(&current)
    }

    fn send(&self) -> Result<()> {
        <T as Device>::send(self)
    }
}

/// Convert a Panasonic model to a string
pub fn model_to_string(model: PanasonicAcModel) -> String {
    match model {
        PanasonicAcModel::Dke => "Panasonic DKE".to_string(),
        PanasonicAcModel::Jke => "Panasonic JKE".to_string(),
        PanasonicAcModel::Lke => "Panasonic LKE".to_string(),
        PanasonicAcModel::Nke => "Panasonic NKE".to_string(),
        PanasonicAcModel::Ckp => "Panasonic CKP".to_string(),
        PanasonicAcModel::Rkr => "Panasonic RKR".to_string(),
        PanasonicAcModel::Unknown => "Panasonic Unknown".to_string(),
    }
}

/// Builder for creating state objects
#[derive(Debug, Default)]
pub struct StateBuilder {
    state: State,
}

impl StateBuilder {
    /// Create a new StateBuilder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the power state
    pub fn power(mut self, on: bool) -> Self {
        self.state.power = on;
        self
    }

    /// Set the operation mode
    pub fn mode(mut self, mode: OpMode) -> Self {
        self.state.mode = mode;
        self
    }

    /// Set the temperature
    #[must_use]
    pub const fn temp(mut self, celsius: u8) -> Self {
        self.state.temp = celsius;
        self
    }

    /// Set the fan speed
    #[must_use]
    pub const fn fan(mut self, fan: FanLevel) -> Self {
        self.state.fan = fan;
        self
    }

    /// Set the vertical swing position
    #[must_use]
    pub const fn swing_v(mut self, pos: Option<SwingVPos>) -> Self {
        self.state.swing_v = pos;
        self
    }

    /// Set the horizontal swing position
    #[must_use]
    pub const fn swing_h(mut self, pos: Option<SwingHPos>) -> Self {
        self.state.swing_h = pos;
        self
    }

    /// Set quiet mode
    #[must_use]
    pub const fn quiet(mut self, on: bool) -> Self {
        self.state.quiet = on;
        self
    }

    /// Set powerful mode
    #[must_use]
    pub const fn powerful(mut self, on: bool) -> Self {
        self.state.powerful = on;
        self
    }

    /// Set ion filter mode
    #[must_use]
    pub const fn ion(mut self, on: bool) -> Self {
        self.state.ion = on;
        self
    }

    /// Build the state
    #[must_use]
    pub fn build(self) -> State {
        self.state
    }
}
