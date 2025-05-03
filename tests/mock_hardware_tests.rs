//! Tests for the mock hardware functionality

#[cfg(feature = "mock")]
mod mock_tests {
    use panasonic_ac::{
        mock::{create_mock_sender, IrReceiver, IrTransmitter, MockReceiver, MockTransmitter},
        types::{AcMode, FanSpeed, SwingV},
        AcDevice, Decoder, Device, IrSignal, PanasonicAc, PanasonicAcModel, PanasonicDecoder,
    };

    #[test]
    fn test_mock_transmitter() {
        let mut transmitter = MockTransmitter::new();

        // Send some test data
        let test_data = vec![0x01, 0x02, 0x03];
        transmitter.send(&test_data, 38000, 2).unwrap();

        // Verify it was recorded
        let sent_data = transmitter.get_sent_data();
        assert_eq!(sent_data.len(), 1);
        assert_eq!(sent_data[0], test_data);

        // Verify frequency was recorded
        let sent_freq = transmitter.get_sent_freq();
        assert_eq!(sent_freq.len(), 1);
        assert_eq!(sent_freq[0], 38000);

        // Verify repeat was recorded
        let sent_repeat = transmitter.get_sent_repeat();
        assert_eq!(sent_repeat.len(), 1);
        assert_eq!(sent_repeat[0], 2);
    }

    #[test]
    fn test_mock_transmitter_fail() {
        let mut transmitter = MockTransmitter::new();

        // Set to fail
        transmitter.set_should_fail(true);

        // Send should fail
        let result = transmitter.send(&[0x01], 38000, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_receiver() {
        let mut receiver = MockReceiver::new();

        // Queue a signal
        let signal = IrSignal {
            data: vec![0x01, 0x02, 0x03],
            carrier_frequency: 38000,
            timings: Some(vec![100, 200, 300]),
        };
        receiver.queue_signal(signal);

        // Receive should return the queued timings
        let received = receiver.receive(1000).unwrap();
        assert_eq!(received, vec![100, 200, 300]);

        // Queue should now be empty
        let result = receiver.receive(1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_receiver_with_data() {
        let mut receiver = MockReceiver::new();

        // Queue raw data
        let data = vec![0x01, 0x02, 0x03];
        receiver.queue_data(&data, 38000).unwrap();

        // Receive should return generated timings
        let received = receiver.receive(1000).unwrap();
        assert!(!received.is_empty()); // Should have some timings
    }

    #[test]
    fn test_create_mock_sender() {
        // Create a mock sender
        let (sender, transmitter) = create_mock_sender();

        // Create an AC with the mock sender
        let mut ac = PanasonicAc::new(sender);

        // Configure the AC
        ac.set_model(PanasonicAcModel::Dke).unwrap();
        ac.set_power(true).unwrap();
        ac.set_mode(AcMode::Cool).unwrap();
        ac.set_temp(23).unwrap();

        // Send the command
        ac.send().unwrap();

        // Verify the mock transmitter recorded it
        assert_eq!(transmitter.get_send_count(), 1);

        // Get the first sent data
        let sent_data = transmitter.get_sent_data()[0].clone();

        // Verify we can decode it
        let mut decoded_ac = PanasonicAc::new(|_, _, _| Ok(()));
        decoded_ac.set_raw(&sent_data).unwrap();

        assert_eq!(decoded_ac.get_power(), true);
        assert_eq!(decoded_ac.get_mode().unwrap(), AcMode::Cool);
        assert_eq!(decoded_ac.get_temp(), 23);
    }

    #[test]
    fn test_full_transmit_receive_cycle() {
        // Create a mock sender and receiver
        let (sender, transmitter) = create_mock_sender();
        let mut receiver = MockReceiver::new();

        // Create an AC with the mock sender
        let mut ac = PanasonicAc::new(sender);

        // Configure the AC
        ac.set_model(PanasonicAcModel::Dke).unwrap();
        ac.set_power(true).unwrap();
        ac.set_mode(AcMode::Heat).unwrap();
        ac.set_temp(24).unwrap();
        ac.set_fan(FanSpeed::Auto).unwrap();
        ac.set_swing_v(SwingV::Auto).unwrap();

        // Send the command
        ac.send().unwrap();

        // Get the sent data from the transmitter
        let sent_data = transmitter.get_sent_data()[0].clone();

        // Queue this data in the receiver
        receiver.queue_data(&sent_data, 38000).unwrap();

        // Receive the signal
        let timings = receiver.receive(1000).unwrap();

        // Create a signal from the timings
        let signal = IrSignal {
            data: vec![],
            carrier_frequency: 38000,
            timings: Some(timings),
        };

        // Decode the signal
        let decoder = panasonic_ac::PanasonicDecoder::new();
        let decoded = decoder.decode(&signal).unwrap();

        // Create a new AC from the decoded data
        let mut decoded_ac = PanasonicAc::new(|_, _, _| Ok(()));
        decoded_ac.set_raw(&decoded).unwrap();

        // Verify the decoded AC has the same settings
        assert_eq!(decoded_ac.get_power(), true);
        assert_eq!(decoded_ac.get_mode().unwrap(), AcMode::Heat);
        assert_eq!(decoded_ac.get_temp(), 24);
        assert_eq!(decoded_ac.get_fan().unwrap(), FanSpeed::Auto);
        assert_eq!(decoded_ac.get_swing_v().unwrap(), SwingV::Auto);
    }
}
