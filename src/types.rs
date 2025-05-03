// types.rs
//! Common types used throughout the Panasonic AC library

use crate::constants::*;
use crate::error::{Error, Result};
use std::fmt;

/// Represents operation modes for AC units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcMode {
    /// Automatic mode selection based on conditions
    Auto,
    /// Cooling mode
    Cool,
    /// Heating mode
    Heat,
    /// Dehumidification mode
    Dry,
    /// Fan only mode (no temperature control)
    Fan,
}

impl From<AcMode> for u8 {
    fn from(mode: AcMode) -> Self {
        match mode {
            AcMode::Auto => PANASONIC_AC_AUTO,
            AcMode::Cool => PANASONIC_AC_COOL,
            AcMode::Heat => PANASONIC_AC_HEAT,
            AcMode::Dry => PANASONIC_AC_DRY,
            AcMode::Fan => PANASONIC_AC_FAN,
        }
    }
}

impl TryFrom<u8> for AcMode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            PANASONIC_AC_AUTO => Ok(AcMode::Auto),
            PANASONIC_AC_COOL => Ok(AcMode::Cool),
            PANASONIC_AC_HEAT => Ok(AcMode::Heat),
            PANASONIC_AC_DRY => Ok(AcMode::Dry),
            PANASONIC_AC_FAN => Ok(AcMode::Fan),
            _ => Err(Error::InvalidValue(format!("Invalid mode: {}", value))),
        }
    }
}

impl fmt::Display for AcMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AcMode::Auto => write!(f, "Auto"),
            AcMode::Cool => write!(f, "Cool"),
            AcMode::Heat => write!(f, "Heat"),
            AcMode::Dry => write!(f, "Dry"),
            AcMode::Fan => write!(f, "Fan"),
        }
    }
}

/// Represents fan speed settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanSpeed {
    /// Automatic fan speed selection
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

impl From<FanSpeed> for u8 {
    fn from(speed: FanSpeed) -> Self {
        match speed {
            FanSpeed::Auto => PANASONIC_AC_FAN_AUTO,
            FanSpeed::Min => PANASONIC_AC_FAN_MIN,
            FanSpeed::Low => PANASONIC_AC_FAN_LOW,
            FanSpeed::Medium => PANASONIC_AC_FAN_MED,
            FanSpeed::High => PANASONIC_AC_FAN_HIGH,
            FanSpeed::Max => PANASONIC_AC_FAN_MAX,
        }
    }
}

impl TryFrom<u8> for FanSpeed {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            PANASONIC_AC_FAN_AUTO => Ok(FanSpeed::Auto),
            PANASONIC_AC_FAN_MIN => Ok(FanSpeed::Min),
            PANASONIC_AC_FAN_LOW => Ok(FanSpeed::Low),
            PANASONIC_AC_FAN_MED => Ok(FanSpeed::Medium),
            PANASONIC_AC_FAN_HIGH => Ok(FanSpeed::High),
            PANASONIC_AC_FAN_MAX => Ok(FanSpeed::Max),
            _ => Err(Error::InvalidValue(format!("Invalid fan speed: {}", value))),
        }
    }
}

impl fmt::Display for FanSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FanSpeed::Auto => write!(f, "Auto"),
            FanSpeed::Min => write!(f, "Minimum"),
            FanSpeed::Low => write!(f, "Low"),
            FanSpeed::Medium => write!(f, "Medium"),
            FanSpeed::High => write!(f, "High"),
            FanSpeed::Max => write!(f, "Maximum"),
        }
    }
}

/// Represents vertical swing positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwingV {
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

impl From<SwingV> for u8 {
    fn from(swing: SwingV) -> Self {
        match swing {
            SwingV::Auto => PANASONIC_AC_SWING_V_AUTO,
            SwingV::Highest => PANASONIC_AC_SWING_V_HIGHEST,
            SwingV::High => PANASONIC_AC_SWING_V_HIGH,
            SwingV::Middle => PANASONIC_AC_SWING_V_MIDDLE,
            SwingV::Low => PANASONIC_AC_SWING_V_LOW,
            SwingV::Lowest => PANASONIC_AC_SWING_V_LOWEST,
        }
    }
}

impl TryFrom<u8> for SwingV {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            PANASONIC_AC_SWING_V_AUTO => Ok(SwingV::Auto),
            PANASONIC_AC_SWING_V_HIGHEST => Ok(SwingV::Highest),
            PANASONIC_AC_SWING_V_HIGH => Ok(SwingV::High),
            PANASONIC_AC_SWING_V_MIDDLE => Ok(SwingV::Middle),
            PANASONIC_AC_SWING_V_LOW => Ok(SwingV::Low),
            PANASONIC_AC_SWING_V_LOWEST => Ok(SwingV::Lowest),
            _ => Err(Error::InvalidValue(format!(
                "Invalid vertical swing: {}",
                value
            ))),
        }
    }
}

impl fmt::Display for SwingV {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwingV::Auto => write!(f, "Auto"),
            SwingV::Highest => write!(f, "Highest"),
            SwingV::High => write!(f, "High"),
            SwingV::Middle => write!(f, "Middle"),
            SwingV::Low => write!(f, "Low"),
            SwingV::Lowest => write!(f, "Lowest"),
        }
    }
}

/// Represents horizontal swing positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwingH {
    /// Automatic swing
    Auto,
    /// Middle position
    Middle,
    /// Full left position
    FullLeft,
    /// Left position
    Left,
    /// Right position
    Right,
    /// Full right position
    FullRight,
}

impl From<SwingH> for u8 {
    fn from(swing: SwingH) -> Self {
        match swing {
            SwingH::Auto => PANASONIC_AC_SWING_H_AUTO,
            SwingH::Middle => PANASONIC_AC_SWING_H_MIDDLE,
            SwingH::FullLeft => PANASONIC_AC_SWING_H_FULL_LEFT,
            SwingH::Left => PANASONIC_AC_SWING_H_LEFT,
            SwingH::Right => PANASONIC_AC_SWING_H_RIGHT,
            SwingH::FullRight => PANASONIC_AC_SWING_H_FULL_RIGHT,
        }
    }
}

impl TryFrom<u8> for SwingH {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            PANASONIC_AC_SWING_H_AUTO => Ok(SwingH::Auto),
            PANASONIC_AC_SWING_H_MIDDLE => Ok(SwingH::Middle),
            PANASONIC_AC_SWING_H_FULL_LEFT => Ok(SwingH::FullLeft),
            PANASONIC_AC_SWING_H_LEFT => Ok(SwingH::Left),
            PANASONIC_AC_SWING_H_RIGHT => Ok(SwingH::Right),
            PANASONIC_AC_SWING_H_FULL_RIGHT => Ok(SwingH::FullRight),
            _ => Err(Error::InvalidValue(format!(
                "Invalid horizontal swing: {}",
                value
            ))),
        }
    }
}

impl fmt::Display for SwingH {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwingH::Auto => write!(f, "Auto"),
            SwingH::Middle => write!(f, "Middle"),
            SwingH::FullLeft => write!(f, "Full Left"),
            SwingH::Left => write!(f, "Left"),
            SwingH::Right => write!(f, "Right"),
            SwingH::FullRight => write!(f, "Full Right"),
        }
    }
}
