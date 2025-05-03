use panasonic_ac::{
    AcDevice, Decoder, Device, Encoder, PanasonicAc, PanasonicAcModel, PanasonicDecoder,
    PanasonicEncoder, Result,
};

fn main() -> Result<()> {
    println!("Panasonic AC IR Decoding Example");
    println!("================================");

    // Create encoder and decoder instances
    let encoder = PanasonicEncoder::new();
    let decoder = PanasonicDecoder::new();

    // Create an AC instance and configure it with desired settings
    let mut ac = PanasonicAc::new(|_, _, _| Ok(()));
    ac.set_model(PanasonicAcModel::Dke)?;
    ac.set_power(true)?;
    ac.set_mode(panasonic_ac::types::AcMode::Cool)?;
    ac.set_temp(23)?;

    println!("Original AC state: {}", ac);

    // Get the raw state data that would be transmitted via IR
    let raw_state = ac.get_raw();

    println!("\nRaw state data (hex):");
    print_hex_data(&raw_state);

    // Generate an IR signal from the raw state (simulating transmission)
    let signal = encoder.encode(&raw_state)?;

    println!("\nGenerated IR signal:");
    println!("- Data length: {} bytes", signal.data.len());
    println!("- Carrier frequency: {} Hz", signal.carrier_frequency);
    println!(
        "- Timing points: {}",
        signal.timings.as_ref().map_or(0, |t| t.len())
    );

    // Decode the IR signal back to raw data (simulating reception)
    let decoded_data = decoder.decode(&signal)?;

    println!("\nDecoded data (hex):");
    print_hex_data(&decoded_data);

    // Verify that the decoded data matches the original raw state
    let matches = raw_state == decoded_data;
    println!("\nDecoded data matches original: {}", matches);

    // Create a new AC instance and set its state from the decoded data
    let mut received_ac = PanasonicAc::new(|_, _, _| Ok(()));
    received_ac.set_raw(&decoded_data)?;

    // The model may not be properly detected from raw data, so set it explicitly
    received_ac.set_model(PanasonicAcModel::Dke)?;

    println!("\nReceived AC state: {}", received_ac);

    // Demonstrate extracting individual settings from the decoded state
    println!("\nExtracted settings from decoded data:");
    println!("Power: {}", received_ac.get_power());
    println!("Mode: {:?}", received_ac.get_mode()?);
    println!("Temperature: {}°C", received_ac.get_temp());
    println!("Fan Speed: {:?}", received_ac.get_fan()?);
    println!("Vertical Swing: {:?}", received_ac.get_swing_v()?);
    println!("Horizontal Swing: {:?}", received_ac.get_swing_h()?);

    // Demonstrate a simple command (not a full AC state)
    println!("\n--- Simple Command Encoding/Decoding ---");

    // Custom command (could be a simple on/off toggle, etc.)
    let custom_command: u32 = 0x1234ABCD;

    // Encode the command
    let command_signal = encoder.encode_command(custom_command)?;

    // Decode the command
    let decoded_command_data = decoder.decode(&command_signal)?;
    let extracted_command = decoder.extract_command(&decoded_command_data)?;

    println!("Original command: 0x{:08X}", custom_command);
    println!("Extracted command: 0x{:08X}", extracted_command);
    println!("Commands match: {}", custom_command == extracted_command);

    Ok(())
}

// Helper function to print data in hex format
fn print_hex_data(data: &[u8]) {
    for (i, byte) in data.iter().enumerate() {
        print!("{:02X} ", byte);
        if (i + 1) % 8 == 0 {
            println!();
        }
    }
    if data.len() % 8 != 0 {
        println!();
    }
}
