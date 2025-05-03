//! Tests for the encoder and decoder functionality

use panasonic_ac::{
    Decoder, Encoder, PanasonicDecoder, PanasonicEncoder, PANASONIC_AC_STATE_LENGTH,
    PANASONIC_BIT_MARK, PANASONIC_END_GAP, PANASONIC_HDR_MARK, PANASONIC_HDR_SPACE,
    PANASONIC_KNOWN_GOOD_STATE, PANASONIC_ONE_SPACE, PANASONIC_ZERO_SPACE,
};

#[test]
fn test_checksum_calculation() {
    let mut state = PANASONIC_KNOWN_GOOD_STATE.to_vec();

    // Update the last byte with the correct checksum
    let correct_checksum = 0x92; // Lower byte of 0x292
    state[PANASONIC_AC_STATE_LENGTH - 1] = correct_checksum;

    // Calculate checksum for known good state
    let checksum = PanasonicEncoder::calculate_checksum(&state, PANASONIC_AC_STATE_LENGTH);

    // Verify the checksum is correct
    assert_eq!(correct_checksum, checksum);

    // Modify state (excluding checksum)
    state[3] = 0xFF;

    // Recalculate checksum
    let new_checksum = PanasonicEncoder::calculate_checksum(&state, PANASONIC_AC_STATE_LENGTH);

    // Verify checksum has changed
    assert_ne!(checksum, new_checksum);
}

#[test]
fn test_checksum_verification() {
    let mut state = PANASONIC_KNOWN_GOOD_STATE.to_vec();

    // Update the last byte with the correct checksum
    let correct_checksum = 0x92; // Lower byte of 0x292
    state[PANASONIC_AC_STATE_LENGTH - 1] = correct_checksum;

    // Should verify with correct checksum
    assert!(PanasonicEncoder::verify_checksum(
        &state,
        PANASONIC_AC_STATE_LENGTH
    ));

    // Corrupt the checksum
    state[PANASONIC_AC_STATE_LENGTH - 1] = !correct_checksum;

    // Should fail verification with incorrect checksum
    assert!(!PanasonicEncoder::verify_checksum(
        &state,
        PANASONIC_AC_STATE_LENGTH
    ));
}

#[test]
fn test_encode_decode_roundtrip() {
    let encoder = PanasonicEncoder::new();
    let decoder = PanasonicDecoder::new();

    // Known good state
    let state = PANASONIC_KNOWN_GOOD_STATE;

    // Encode
    let signal = encoder.encode(&state).unwrap();

    // Decode - note: since we're not testing with real timings but just the data,
    // we'll directly extract data from signal for the round-trip test
    let decoded = decoder.decode(&signal).unwrap();

    // Verify roundtrip (decoded data should match original state)
    assert_eq!(state.to_vec(), decoded);
}

#[test]
fn test_encode_command() {
    let encoder = PanasonicEncoder::new();
    let decoder = PanasonicDecoder::new();

    // Test command
    let command: u32 = 0x12345678;

    // Encode command
    let signal = encoder.encode_command(command).unwrap();

    // Decode
    let decoded = decoder.decode(&signal).unwrap();

    // Extract command
    let extracted_command = decoder.extract_command(&decoded).unwrap();

    // Verify roundtrip (extracted command should match original command)
    assert_eq!(command, extracted_command);
}

#[test]
fn test_generate_timings() {
    let encoder = PanasonicEncoder::new();

    // Simple test data
    let data = [0xA5, 0x5A]; // 10100101 01011010

    // Encode with timings
    let signal = encoder.encode(&data).unwrap();

    // Verify timings were generated
    assert!(signal.timings.is_some());

    if let Some(timings) = signal.timings {
        // Header (mark + space)
        assert_eq!(timings[0], PANASONIC_HDR_MARK);
        assert_eq!(timings[1], PANASONIC_HDR_SPACE);

        // Should have 2 bytes * 8 bits * 2 timing entries each + header (2) + footer (2)
        // = 2 * 8 * 2 + 2 + 2 = 36
        assert_eq!(timings.len(), 36);

        // Footer (mark + space)
        assert_eq!(timings[34], PANASONIC_BIT_MARK);
        assert_eq!(timings[35], PANASONIC_END_GAP);

        // Spot check a few bit encodings (first byte 0xA5 = 10100101)
        // First bit is 1
        assert_eq!(timings[2], PANASONIC_BIT_MARK);
        assert_eq!(timings[3], PANASONIC_ONE_SPACE);

        // Second bit is 0
        assert_eq!(timings[4], PANASONIC_BIT_MARK);
        assert_eq!(timings[5], PANASONIC_ZERO_SPACE);
    } else {
        panic!("Timings not generated");
    }
}
