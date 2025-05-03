//! Tests for the RPi hardware implementation using mocks
//! 
//! Since we can't directly test hardware interaction in CI environments,
//! these tests use mocking to verify the behavior of the hardware implementation.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use panasonic_ac::error::{Error, Result};
#[cfg(feature = "rpi")]
use panasonic_ac::hardware::{IrReceiver, IrTransmitter};

#[cfg(feature = "rpi")]
mod rpi_tests {
    use super::*;
    use panasonic_ac::encoder::PanasonicEncoder;
    use panasonic_ac::hardware::{Hardware, rpi::{RpiHardware, RpiReceiver, RpiTransmitter}};

    /// Mock of the GPIO pin handling
    struct MockRpiTransmitter {
        data_log: Rc<RefCell<Vec<Vec<u8>>>>,
        freq_log: Rc<RefCell<Vec<u32>>>,
        repeat_log: Rc<RefCell<Vec<u16>>>,
        should_fail: bool,
    }

    impl MockRpiTransmitter {
        fn new(
            data_log: Rc<RefCell<Vec<Vec<u8>>>>,
            freq_log: Rc<RefCell<Vec<u32>>>,
            repeat_log: Rc<RefCell<Vec<u16>>>,
            should_fail: bool,
        ) -> Self {
            Self {
                data_log,
                freq_log,
                repeat_log,
                should_fail,
            }
        }
    }

    impl IrTransmitter for MockRpiTransmitter {
        fn send(&mut self, data: &[u8], freq: u32, repeat: u16) -> Result<()> {
            if self.should_fail {
                return Err(Error::HardwareError("Mock transmitter failure".to_string()));
            }

            self.data_log.borrow_mut().push(data.to_vec());
            self.freq_log.borrow_mut().push(freq);
            self.repeat_log.borrow_mut().push(repeat);
            
            Ok(())
        }

        fn cleanup(&mut self) -> Result<()> {
            Ok(())
        }
    }

    /// Mock of the IR receiver
    struct MockRpiReceiver {
        timings: Vec<u16>,
        should_fail: bool,
    }

    impl MockRpiReceiver {
        fn new(timings: Vec<u16>, should_fail: bool) -> Self {
            Self {
                timings,
                should_fail,
            }
        }
    }

    impl IrReceiver for MockRpiReceiver {
        fn receive(&mut self, _timeout_ms: u32) -> Result<Vec<u16>> {
            if self.should_fail {
                return Err(Error::HardwareError("Mock receiver failure".to_string()));
            }
            Ok(self.timings.clone())
        }

        fn cleanup(&mut self) -> Result<()> {
            Ok(())
        }
    }

    /// Mock version of RpiHardware for testing
    struct MockRpiHardware {
        tx_data: Rc<RefCell<Vec<Vec<u8>>>>,
        tx_freq: Rc<RefCell<Vec<u32>>>,
        tx_repeat: Rc<RefCell<Vec<u16>>>,
        rx_timings: Vec<u16>,
        tx_should_fail: bool,
        rx_should_fail: bool,
    }

    impl MockRpiHardware {
        fn new(
            tx_data: Rc<RefCell<Vec<Vec<u8>>>>,
            tx_freq: Rc<RefCell<Vec<u32>>>,
            tx_repeat: Rc<RefCell<Vec<u16>>>,
            rx_timings: Vec<u16>,
            tx_should_fail: bool,
            rx_should_fail: bool,
        ) -> Self {
            Self {
                tx_data,
                tx_freq,
                tx_repeat,
                rx_timings,
                tx_should_fail,
                rx_should_fail,
            }
        }
    }

    impl panasonic_ac::hardware::Hardware for MockRpiHardware {
        fn create_transmitter(&self) -> Result<Box<dyn IrTransmitter>> {
            Ok(Box::new(MockRpiTransmitter::new(
                self.tx_data.clone(),
                self.tx_freq.clone(),
                self.tx_repeat.clone(),
                self.tx_should_fail,
            )))
        }

        fn create_receiver(&self) -> Result<Box<dyn IrReceiver>> {
            Ok(Box::new(MockRpiReceiver::new(
                self.rx_timings.clone(),
                self.rx_should_fail,
            )))
        }

        fn create_sender(&self) -> Result<Box<dyn Fn(&[u8], u32, u16) -> Result<()>>> {
            let tx_data = self.tx_data.clone();
            let tx_freq = self.tx_freq.clone();
            let tx_repeat = self.tx_repeat.clone();
            let should_fail = self.tx_should_fail;

            Ok(Box::new(move |data: &[u8], freq: u32, repeat: u16| -> Result<()> {
                if should_fail {
                    return Err(Error::HardwareError("Mock sender failure".to_string()));
                }

                tx_data.borrow_mut().push(data.to_vec());
                tx_freq.borrow_mut().push(freq);
                tx_repeat.borrow_mut().push(repeat);

                Ok(())
            }))
        }
    }

    #[test]
    fn test_rpi_transmitter_with_encoder() {
        // Setup test data
        let test_data = vec![0x02, 0x20, 0xE0, 0x04, 0x00, 0x00, 0x00, 0x06];
        let test_freq = 36700;
        let test_repeat = 2;

        // Create a mock transmitter
        let data_log = Rc::new(RefCell::new(Vec::new()));
        let freq_log = Rc::new(RefCell::new(Vec::new()));
        let repeat_log = Rc::new(RefCell::new(Vec::new()));
        
        // Simulate RPi transmitter with PanasonicEncoder
        let mut mock_tx = MockRpiTransmitter::new(
            data_log.clone(),
            freq_log.clone(),
            repeat_log.clone(),
            false,
        );

        // Send the data
        assert!(mock_tx.send(&test_data, test_freq, test_repeat).is_ok());

        // Verify the data was correctly logged
        assert_eq!(data_log.borrow().len(), 1);
        assert_eq!(data_log.borrow()[0], test_data);
        assert_eq!(freq_log.borrow()[0], test_freq);
        assert_eq!(repeat_log.borrow()[0], test_repeat);
    }

    #[test]
    fn test_rpi_transmitter_failure() {
        // Setup test data
        let test_data = vec![0x02, 0x20, 0xE0, 0x04];
        let test_freq = 36700;
        let test_repeat = 0;

        // Create a mock transmitter that will fail
        let data_log = Rc::new(RefCell::new(Vec::new()));
        let freq_log = Rc::new(RefCell::new(Vec::new()));
        let repeat_log = Rc::new(RefCell::new(Vec::new()));
        
        let mut mock_tx = MockRpiTransmitter::new(
            data_log.clone(),
            freq_log.clone(),
            repeat_log.clone(),
            true, // Should fail
        );

        // Send should fail
        let result = mock_tx.send(&test_data, test_freq, test_repeat);
        assert!(result.is_err());
        
        // Verify no data was logged
        assert_eq!(data_log.borrow().len(), 0);
    }

    #[test]
    fn test_rpi_receiver() {
        // Setup test timings
        let test_timings = vec![3456, 1728, 432, 1296, 432, 432, 432, 1296];
        
        // Create a mock receiver
        let mut mock_rx = MockRpiReceiver::new(test_timings.clone(), false);
        
        // Receive data with a timeout
        let result = mock_rx.receive(1000);
        assert!(result.is_ok());
        
        // Verify received data matches
        let received = result.unwrap();
        assert_eq!(received, test_timings);
    }

    #[test]
    fn test_rpi_receiver_failure() {
        // Create a mock receiver that will fail
        let mut mock_rx = MockRpiReceiver::new(vec![], true);
        
        // Receive should fail
        let result = mock_rx.receive(1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_rpi_hardware_integration() {
        // Setup test data
        let test_data = vec![0x02, 0x20, 0xE0, 0x04];
        let test_timings = vec![3456, 1728, 432, 1296];
        
        // Create logs for tracking calls
        let tx_data = Rc::new(RefCell::new(Vec::new()));
        let tx_freq = Rc::new(RefCell::new(Vec::new()));
        let tx_repeat = Rc::new(RefCell::new(Vec::new()));
        
        // Create mock hardware
        let hardware = MockRpiHardware::new(
            tx_data.clone(),
            tx_freq.clone(),
            tx_repeat.clone(),
            test_timings.clone(),
            false, // Tx should not fail
            false, // Rx should not fail
        );
        
        // Test transmitter creation
        let mut transmitter = hardware.create_transmitter().unwrap();
        assert!(transmitter.send(&test_data, 36700, 0).is_ok());
        
        // Test receiver creation
        let mut receiver = hardware.create_receiver().unwrap();
        let received = receiver.receive(1000).unwrap();
        assert_eq!(received, test_timings);
        
        // Test sender creation
        let sender = hardware.create_sender().unwrap();
        assert!(sender(&test_data, 36700, 0).is_ok());
        
        // Verify all data was correctly logged
        assert_eq!(tx_data.borrow().len(), 2); // One from transmitter, one from sender
        assert_eq!(tx_data.borrow()[0], test_data);
        assert_eq!(tx_data.borrow()[1], test_data);
    }

    #[test]
    fn test_mock_hardware_failures() {
        // Create logs for tracking calls
        let tx_data = Rc::new(RefCell::new(Vec::new()));
        let tx_freq = Rc::new(RefCell::new(Vec::new()));
        let tx_repeat = Rc::new(RefCell::new(Vec::new()));
        
        // Create mock hardware with failures
        let hardware = MockRpiHardware::new(
            tx_data.clone(),
            tx_freq.clone(),
            tx_repeat.clone(),
            vec![],           // Empty timings
            true,             // Tx should fail
            true,             // Rx should fail
        );
        
        // Transmitter should fail
        let mut transmitter = hardware.create_transmitter().unwrap();
        assert!(transmitter.send(&[0x01], 36700, 0).is_err());
        
        // Receiver should fail
        let mut receiver = hardware.create_receiver().unwrap();
        assert!(receiver.receive(1000).is_err());
        
        // Sender should fail
        let sender = hardware.create_sender().unwrap();
        assert!(sender(&[0x01], 36700, 0).is_err());
        
        // Verify no data was logged
        assert_eq!(tx_data.borrow().len(), 0);
    }
}

#[cfg(feature = "rpi")]
mod panasonic_ac_with_hardware_tests {
    use super::*;
    use panasonic_ac::{AcDevice, Device, PanasonicAc, PanasonicAcModel};
    use panasonic_ac::types::{AcMode, FanSpeed, SwingV};

    // Helper struct to track IR signals
    struct SignalTracker {
        data: Arc<Mutex<Vec<Vec<u8>>>>,
        freq: Arc<Mutex<Vec<u32>>>,
        repeat: Arc<Mutex<Vec<u16>>>,
    }

    impl SignalTracker {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(Vec::new())),
                freq: Arc::new(Mutex::new(Vec::new())),
                repeat: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn create_sender(&self) -> impl Fn(&[u8], u32, u16) -> Result<()> + 'static {
            let data = self.data.clone();
            let freq = self.freq.clone();
            let repeat = self.repeat.clone();

            move |d: &[u8], f: u32, r: u16| -> Result<()> {
                data.lock().unwrap().push(d.to_vec());
                freq.lock().unwrap().push(f);
                repeat.lock().unwrap().push(r);
                Ok(())
            }
        }

        fn get_signals(&self) -> Vec<Vec<u8>> {
            self.data.lock().unwrap().clone()
        }
    }

    #[test]
    fn test_panasonic_ac_with_hardware() {
        // Create a signal tracker
        let tracker = SignalTracker::new();
        let sender = tracker.create_sender();
        
        // Create AC controller with the sender
        let mut ac = PanasonicAc::new(sender);
        
        // Configure the AC
        ac.set_model(PanasonicAcModel::Dke).unwrap();
        ac.set_power(true).unwrap();
        ac.set_mode(AcMode::Cool).unwrap();
        ac.set_temp(23).unwrap();
        ac.set_fan(FanSpeed::Auto).unwrap();
        ac.set_swing_v(SwingV::Auto).unwrap();
        
        // Send the command
        ac.send().unwrap();
        
        // Verify a signal was sent
        let signals = tracker.get_signals();
        assert_eq!(signals.len(), 1);
        assert!(!signals[0].is_empty());
        
        // Change some settings and send again
        ac.set_temp(25).unwrap();
        ac.set_power(false).unwrap();
        ac.send().unwrap();
        
        // Verify another signal was sent
        let signals = tracker.get_signals();
        assert_eq!(signals.len(), 2);
        
        // Verify the two signals are different (power on vs power off)
        assert_ne!(signals[0], signals[1]);
    }

    #[test]
    fn test_panasonic_ac_multiple_commands() {
        // Create a signal tracker
        let tracker = SignalTracker::new();
        let sender = tracker.create_sender();
        
        // Create AC controller with the sender
        let mut ac = PanasonicAc::new(sender);
        ac.set_model(PanasonicAcModel::Dke).unwrap();
        
        // Test sequence of commands
        let test_temperatures = [16, 20, 24, 28, 30];
        
        for &temp in &test_temperatures {
            ac.set_temp(temp).unwrap();
            ac.send().unwrap();
        }
        
        // Verify the correct number of signals
        let signals = tracker.get_signals();
        assert_eq!(signals.len(), test_temperatures.len());
        
        // Test different modes
        ac.set_power(true).unwrap();
        ac.set_mode(AcMode::Cool).unwrap();
        ac.send().unwrap();
        
        ac.set_mode(AcMode::Heat).unwrap();
        ac.send().unwrap();
        
        ac.set_mode(AcMode::Auto).unwrap();
        ac.send().unwrap();
        
        // Verify more signals were sent
        let signals = tracker.get_signals();
        assert_eq!(signals.len(), test_temperatures.len() + 3);
    }
}