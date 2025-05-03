// mock.rs
//! Mock hardware implementation for testing Panasonic AC functionality

use crate::encoder::{Encoder, IrSignal, PanasonicEncoder};
use crate::error::{Error, Result};
use std::sync::{Arc, Mutex};

/// Trait for IR transmission hardware
pub trait IrTransmitter {
    /// Send IR data
    fn send(&mut self, data: &[u8], freq: u32, repeat: u16) -> Result<()>;

    /// Cleanup resources when done
    fn cleanup(&mut self) -> Result<()>;
}

/// Trait for IR reception hardware
pub trait IrReceiver {
    /// Receive IR timings with a timeout
    fn receive(&mut self, timeout_ms: u32) -> Result<Vec<u16>>;

    /// Cleanup resources when done
    fn cleanup(&mut self) -> Result<()>;
}

/// Mock IR transmitter for testing
#[derive(Debug, Clone)]
pub struct MockTransmitter {
    /// Transmitted data
    sent_data: Arc<Mutex<Vec<Vec<u8>>>>,
    /// Transmitted frequencies
    sent_freq: Arc<Mutex<Vec<u32>>>,
    /// Transmitted repeat counts
    sent_repeat: Arc<Mutex<Vec<u16>>>,
    /// Flag to simulate failure
    should_fail: Arc<Mutex<bool>>,
}

impl MockTransmitter {
    /// Create a new mock transmitter
    pub fn new() -> Self {
        Self {
            sent_data: Arc::new(Mutex::new(Vec::new())),
            sent_freq: Arc::new(Mutex::new(Vec::new())),
            sent_repeat: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Set whether the transmitter should fail
    pub fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().unwrap() = fail;
    }

    /// Get the transmitted data
    pub fn get_sent_data(&self) -> Vec<Vec<u8>> {
        self.sent_data.lock().unwrap().clone()
    }

    /// Get the transmitted frequencies
    pub fn get_sent_freq(&self) -> Vec<u32> {
        self.sent_freq.lock().unwrap().clone()
    }

    /// Get the transmitted repeat counts
    pub fn get_sent_repeat(&self) -> Vec<u16> {
        self.sent_repeat.lock().unwrap().clone()
    }

    /// Get the number of transmissions
    pub fn get_send_count(&self) -> usize {
        self.sent_data.lock().unwrap().len()
    }

    /// Clear the transmission history
    pub fn clear_history(&self) {
        self.sent_data.lock().unwrap().clear();
        self.sent_freq.lock().unwrap().clear();
        self.sent_repeat.lock().unwrap().clear();
    }
}

impl IrTransmitter for MockTransmitter {
    fn send(&mut self, data: &[u8], freq: u32, repeat: u16) -> Result<()> {
        // Check if we should simulate failure
        if *self.should_fail.lock().unwrap() {
            return Err(Error::HardwareError(
                "Simulated transmitter failure".to_string(),
            ));
        }

        // Record the transmission
        self.sent_data.lock().unwrap().push(data.to_vec());
        self.sent_freq.lock().unwrap().push(freq);
        self.sent_repeat.lock().unwrap().push(repeat);

        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        // Nothing to clean up in mock
        Ok(())
    }
}

impl Default for MockTransmitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock IR receiver for testing
#[derive(Debug, Clone)]
pub struct MockReceiver {
    /// Queue of signals to return
    signal_queue: Arc<Mutex<Vec<IrSignal>>>,
    /// Flag to simulate failure
    should_fail: Arc<Mutex<bool>>,
}

impl MockReceiver {
    /// Create a new mock receiver
    pub fn new() -> Self {
        Self {
            signal_queue: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Set whether the receiver should fail
    pub fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().unwrap() = fail;
    }

    /// Queue a signal to be returned by receive
    pub fn queue_signal(&self, signal: IrSignal) {
        self.signal_queue.lock().unwrap().push(signal);
    }

    /// Queue a raw data signal to be returned
    pub fn queue_data(&self, data: &[u8], _freq: u32) -> Result<()> {
        let encoder = PanasonicEncoder::new();
        let signal = encoder.encode(data)?;
        self.queue_signal(signal);
        Ok(())
    }

    /// Clear the signal queue
    pub fn clear_queue(&self) {
        self.signal_queue.lock().unwrap().clear();
    }
}

impl IrReceiver for MockReceiver {
    fn receive(&mut self, _timeout_ms: u32) -> Result<Vec<u16>> {
        // Check if we should simulate failure
        if *self.should_fail.lock().unwrap() {
            return Err(Error::HardwareError(
                "Simulated receiver failure".to_string(),
            ));
        }

        // Get the next signal from the queue
        let mut queue = self.signal_queue.lock().unwrap();
        if queue.is_empty() {
            return Err(Error::HardwareError("No more signals in queue".to_string()));
        }

        let signal = queue.remove(0);

        // Return the timings or generate them if needed
        if let Some(timings) = signal.timings {
            Ok(timings)
        } else {
            // Generate timings from data
            let encoder = PanasonicEncoder::new();
            let signal_with_timings = encoder.encode(&signal.data)?;
            if let Some(timings) = signal_with_timings.timings {
                Ok(timings)
            } else {
                Err(Error::HardwareError(
                    "Failed to generate timings".to_string(),
                ))
            }
        }
    }

    fn cleanup(&mut self) -> Result<()> {
        // Nothing to clean up in mock
        Ok(())
    }
}

impl Default for MockReceiver {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a function that sends IR signals to a mock transmitter
///
/// This is useful for testing without real hardware.
///
/// # Returns
/// * A tuple containing:
///   - A send function compatible with PanasonicAc::new()
///   - The mock transmitter for inspecting what was sent
pub fn create_mock_sender() -> (
    impl Fn(&[u8], u32, u16) -> Result<()> + 'static,
    MockTransmitter,
) {
    let transmitter = MockTransmitter::new();
    let tx_clone = transmitter.clone();

    // We need to capture by value to satisfy the Fn trait (not FnMut)
    let sender = move |data: &[u8], freq: u32, repeat: u16| {
        // Create a new clone for each call to avoid mutating tx_clone
        let mut tx = tx_clone.clone();
        tx.send(data, freq, repeat)
    };

    (sender, transmitter)
}
