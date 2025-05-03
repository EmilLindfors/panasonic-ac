// lib.rs
//! Panasonic heat pump controller library
//!
//! This library provides functionality for controlling Panasonic heat pumps/air conditioners
//! via IR communication, with optional hardware support for Raspberry Pi.

pub mod constants;
mod device;
pub mod encoder;
pub mod error;
#[cfg(feature = "rpi")]
pub mod hardware;
#[cfg(feature = "mock")]
pub mod mock;
pub mod protocol;
mod state;
pub mod types;
pub mod utils;

pub use constants::*;
pub use device::{
    PanasonicAc, PanasonicAc32, PanasonicAcBuilder, PanasonicAcChanges, PanasonicAcModel,
};
pub use encoder::{Decoder, Encoder, IrSignal, PanasonicDecoder, PanasonicEncoder};
pub use error::{Error, Result};
pub use protocol::{AcDevice, Device};
pub use state::{PanasonicAc32State, PanasonicAcState};
pub use types::{AcMode, FanSpeed, SwingH, SwingV};
pub use utils::standard;

/// Re-export of commonly used items for easier access
pub mod prelude {
    pub use crate::device::{
        PanasonicAc, PanasonicAc32, PanasonicAcBuilder, PanasonicAcChanges, PanasonicAcModel,
    };
    pub use crate::encoder::{Decoder, Encoder, IrSignal, PanasonicDecoder, PanasonicEncoder};
    pub use crate::protocol::{AcDevice, Device};
    pub use crate::state::{PanasonicAc32State, PanasonicAcState};
    pub use crate::types::{AcMode, FanSpeed, SwingH, SwingV};
    pub use crate::utils::standard;
    pub use crate::{Error, Result};

    // Hardware support if enabled
    #[cfg(feature = "rpi")]
    pub use crate::hardware::{IrReceiver, IrTransmitter};
}

/// Configure the library with default settings
pub fn init() -> Result<()> {
    #[cfg(feature = "rpi")]
    {
        // Initialize hardware if using Raspberry Pi
        info!("Initializing Panasonic AC library with Raspberry Pi support");
    }

    #[cfg(not(any(feature = "rpi")))]
    {
        // Generic initialization
        log::info!("Initializing Panasonic AC library");
    }

    Ok(())
}

// Internal logger macro
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}
