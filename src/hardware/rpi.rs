// hardware/rpi.rs
//! Raspberry Pi hardware implementation for IR control

#[cfg(feature = "rpi")]
use log::{debug, info, warn};
#[cfg(feature = "rpi")]
use rppal::gpio::{Gpio, InputPin, Level, OutputPin};
#[cfg(feature = "rpi")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "rpi")]
use std::thread;
#[cfg(feature = "rpi")]
use std::time::{Duration, Instant};

#[cfg(feature = "rpi")]
use crate::encoder::{Encoder, PanasonicEncoder};
#[cfg(feature = "rpi")]
use crate::error::{Error, Result};
#[cfg(feature = "rpi")]
use crate::hardware::{IrReceiver, IrTransmitter};


#[cfg(feature = "rpi")]
/// Raspberry Pi IR transmitter implementation
pub struct RpiTransmitter {
    pin: OutputPin,
    encoder: PanasonicEncoder,
}

#[cfg(feature = "rpi")]
impl RpiTransmitter {
    /// Create a new Raspberry Pi IR transmitter
    pub fn new(gpio_pin: u8) -> Result<Self> {
        let gpio = Gpio::new().map_err(|e| Error::GpioError(e.to_string()))?;
        let mut pin = gpio
            .get(gpio_pin)
            .map_err(|e| Error::GpioError(e.to_string()))?
            .into_output();

        // Initialize pin as low
        pin.set_low();

        info!("RpiTransmitter initialized on GPIO pin {}", gpio_pin);
        Ok(Self {
            pin,
            encoder: PanasonicEncoder::new(),
        })
    }

    /// Send raw timing sequence
    pub fn send_raw(&mut self, timings: &[u16], carrier_freq: u32) -> Result<()> {
        debug!(
            "Sending {} timing pairs at {}Hz",
            timings.len() / 2,
            carrier_freq
        );

        // Process the timings directly using the pin
        // Instead of using CarrierPulseGenerator which would require moving the pin
        for (i, &duration) in timings.iter().enumerate() {
            if duration == 0 {
                continue;
            }

            if i % 2 == 0 {
                // Mark - modulated carrier
                // Generate carrier by toggling pin
                let cycle_time_us = 1_000_000 / carrier_freq;
                let half_cycle_us = cycle_time_us / 2;
                
                let start = Instant::now();
                let duration = Duration::from_micros(duration as u64);
                
                while start.elapsed() < duration {
                    self.pin.set_high();
                    thread::sleep(Duration::from_micros(half_cycle_us as u64));
                    self.pin.set_low();
                    thread::sleep(Duration::from_micros(half_cycle_us as u64));
                }
            } else {
                // Space - no carrier
                self.pin.set_low();
                thread::sleep(Duration::from_micros(duration as u64));
            }
        }

        // Ensure pin is low when done
        self.pin.set_low();
        Ok(())
    }
}

#[cfg(feature = "rpi")]
impl IrTransmitter for RpiTransmitter {
    fn send(&mut self, data: &[u8], carrier_freq: u32, repeat: u16) -> Result<()> {
        // Use the encoder to generate proper timing sequences from data
        let signal = self.encoder.encode(data)?;
        
        if let Some(timings) = signal.timings {
            debug!("Sending encoded IR signal with {} timings", timings.len());
            
            // Send the signal the specified number of times
            for r in 0..=repeat {
                if r > 0 {
                    debug!("Sending repeat {} of {}", r, repeat);
                }
                
                self.send_raw(&timings, carrier_freq)?;
                
                // Add gap between repeats if needed
                if r < repeat {
                    thread::sleep(Duration::from_millis(45));
                }
            }
            
            Ok(())
        } else {
            warn!("No timings available in encoded signal");
            Err(Error::HardwareError("No timings available in encoded signal".to_string()))
        }
    }

    fn cleanup(&mut self) -> Result<()> {
        // Ensure pin is low when done
        self.pin.set_low();
        Ok(())
    }
}

#[cfg(feature = "rpi")]
/// Raspberry Pi IR receiver implementation
pub struct RpiReceiver {
    pin: InputPin,
}

#[cfg(feature = "rpi")]
impl RpiReceiver {
    /// Create a new Raspberry Pi IR receiver
    pub fn new(gpio_pin: u8) -> Result<Self> {
        let gpio = Gpio::new().map_err(|e| Error::GpioError(e.to_string()))?;
        let pin = gpio
            .get(gpio_pin)
            .map_err(|e| Error::GpioError(e.to_string()))?
            .into_input();

        info!("RpiReceiver initialized on GPIO pin {}", gpio_pin);
        Ok(Self { pin })
    }
}

#[cfg(feature = "rpi")]
impl IrReceiver for RpiReceiver {
    fn receive(&mut self, timeout_ms: u32) -> Result<Vec<u16>> {
        let mut timings = Vec::new();
        let timeout = Duration::from_millis(timeout_ms as u64);
        let start_time = Instant::now();

        // Wait for the start of a signal (wait for a LOW)
        debug!("Waiting for IR signal...");
        while self.pin.read() == Level::High {
            if start_time.elapsed() > timeout {
                return Err(Error::HardwareError(
                    "Timeout waiting for signal".to_string(),
                ));
            }
            thread::sleep(Duration::from_micros(5));
        }

        debug!("Signal detected, starting capture");

        // Capture the timing of changes
        let mut last_change = Instant::now();
        let mut last_level = self.pin.read();

        loop {
            let level = self.pin.read();

            // If the level changed, record the timing
            if level != last_level {
                let elapsed = last_change.elapsed();
                timings.push(elapsed.as_micros() as u16);
                last_change = Instant::now();
                last_level = level;
            }

            // Check for timeout or too long gap (indicating end of signal)
            let current_gap = last_change.elapsed();
            if current_gap > Duration::from_millis(30) || start_time.elapsed() > timeout {
                break;
            }

            // Avoid busy-waiting
            thread::sleep(Duration::from_micros(5));
        }

        debug!("Captured {} timing transitions", timings.len());

        Ok(timings)
    }

    fn cleanup(&mut self) -> Result<()> {
        // Nothing to clean up for input pin
        Ok(())
    }
}

/// Comprehensive Raspberry Pi hardware handler
#[cfg(feature = "rpi")]
pub struct RpiHardware {
    pub tx_pin: u8,
    pub rx_pin: Option<u8>,
}

#[cfg(feature = "rpi")]
impl RpiHardware {
    /// Create a new RpiHardware instance
    pub fn new(tx_pin: u8) -> Result<Self> {
        // Initialize GPIO once
        let _lock = GPIO_INIT.lock().map_err(|_| {
            Error::HardwareError("Failed to acquire GPIO initialization lock".to_string())
        })?;

        // Return hardware manager
        Ok(Self {
            tx_pin,
            rx_pin: None,
        })
    }

    /// Create a new RpiHardware instance with both transmitter and receiver
    pub fn new_with_receiver(tx_pin: u8, rx_pin: u8) -> Result<Self> {
        // Initialize GPIO once
        let _lock = GPIO_INIT.lock().map_err(|_| {
            Error::HardwareError("Failed to acquire GPIO initialization lock".to_string())
        })?;

        // Return hardware manager
        Ok(Self {
            tx_pin,
            rx_pin: Some(rx_pin),
        })
    }

    /// Create an IR transmitter closure
    pub fn create_sender(&self) -> impl Fn(&[u8], u32, u16) -> Result<()> + '_ {
        // Move to heap to ensure it doesn't go out of scope
        let transmitter = Arc::new(Mutex::new(None));
        let tx_pin = self.tx_pin;
        
        let tx_clone = transmitter.clone();
        
        move |data: &[u8], carrier_freq: u32, repeat: u16| -> Result<()> {
            // Lazy initialization of the transmitter
            let mut tx_guard = tx_clone.lock().map_err(|e| {
                Error::HardwareError(format!("Failed to acquire transmitter lock: {}", e))
            })?;
            
            if tx_guard.is_none() {
                *tx_guard = Some(RpiTransmitter::new(tx_pin)?);
            }
            
            let tx = tx_guard.as_mut().unwrap();
            tx.send(data, carrier_freq, repeat)
        }
    }

    /// Create an IR receiver if configured
    pub fn create_receiver_impl(&self) -> Result<Box<dyn IrReceiver>> {
        if let Some(rx_pin) = self.rx_pin {
            Ok(Box::new(RpiReceiver::new(rx_pin)?))
        } else {
            Err(Error::HardwareError("Receiver not configured".to_string()))
        }
    }
}

#[cfg(feature = "rpi")]
impl crate::hardware::Hardware for RpiHardware {
    fn create_transmitter(&self) -> Result<Box<dyn IrTransmitter>> {
        Ok(Box::new(RpiTransmitter::new(self.tx_pin)?))
    }
    
    fn create_receiver(&self) -> Result<Box<dyn IrReceiver>> {
        self.create_receiver_impl()
    }
    
    fn create_sender(&self) -> Result<Box<dyn Fn(&[u8], u32, u16) -> Result<()> + 'static>> {
        // Create a sender that doesn't depend on the closure created by self.create_sender()
        let tx_pin = self.tx_pin;
        
        // Create a captured transmitter on the heap
        let transmitter = Arc::new(Mutex::new(None));
        let transmitter_clone = transmitter.clone();
        
        Ok(Box::new(move |data: &[u8], freq: u32, repeat: u16| -> Result<()> {
            // Lazy initialization of the transmitter
            let mut tx_guard = transmitter_clone.lock().map_err(|e| {
                Error::HardwareError(format!("Failed to acquire transmitter lock: {}", e))
            })?;
            
            if tx_guard.is_none() {
                *tx_guard = Some(RpiTransmitter::new(tx_pin)?);
            }
            
            let tx = tx_guard.as_mut().unwrap();
            tx.send(data, freq, repeat)
        }))
    }
}

#[cfg(feature = "rpi")]
lazy_static::lazy_static! {
    // Once-cell initialization for GPIO
    static ref GPIO_INIT: Mutex<()> = Mutex::new(());
}
