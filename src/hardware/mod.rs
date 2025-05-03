// hardware/mod.rs
//! Hardware implementations for IR control

#[cfg(feature = "rpi")]
pub mod rpi;

use crate::error::Result;

/// Trait for IR transmitter implementations
pub trait IrTransmitter {
    /// Send IR signal
    ///
    /// # Arguments
    /// * `data` - Data to send
    /// * `carrier_freq` - Carrier frequency in Hz
    /// * `repeat` - Number of times to repeat the transmission
    fn send(&mut self, data: &[u8], carrier_freq: u32, repeat: u16) -> Result<()>;

    /// Clean up resources when done
    fn cleanup(&mut self) -> Result<()>;
}

/// Trait for IR receiver implementations
pub trait IrReceiver {
    /// Receive IR signal
    ///
    /// # Arguments
    /// * `timeout_ms` - Timeout in milliseconds
    ///
    /// # Returns
    /// * `Vec<u16>` - Raw timings in microseconds
    fn receive(&mut self, timeout_ms: u32) -> Result<Vec<u16>>;

    /// Clean up resources when done
    fn cleanup(&mut self) -> Result<()>;
}

/// Create a default IR sender based on available hardware
pub fn create_default_transmitter() -> Result<Box<dyn IrTransmitter>> {
    #[cfg(feature = "rpi")]
    {
        // Default GPIO pin for IR LED on Raspberry Pi
        let default_pin = 17;
        return Ok(Box::new(rpi::RpiTransmitter::new(default_pin)?));
    }

    #[cfg(not(feature = "rpi"))]
    {
        return Err(crate::error::Error::UnsupportedFeature {
            feature: "Hardware IR transmitter".to_string(),
            model: None,
        });
    }
}

/// Create a default IR receiver based on available hardware
pub fn create_default_receiver() -> Result<Box<dyn IrReceiver>> {
    #[cfg(feature = "rpi")]
    {
        // Default GPIO pin for IR receiver on Raspberry Pi
        let default_pin = 27;
        return Ok(Box::new(rpi::RpiReceiver::new(default_pin)?));
    }

    #[cfg(not(feature = "rpi"))]
    {
        return Err(crate::error::Error::UnsupportedFeature {
            feature: "Hardware IR receiver".to_string(),
            model: None,
        });
    }
}
