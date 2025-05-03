//! Raspberry Pi example for Panasonic AC control
//!
//! This example demonstrates how to control a Panasonic AC unit using a Raspberry Pi.
//!
//! Requirements:
//! - Raspberry Pi (any model)
//! - IR LED connected to a GPIO pin (default: GPIO17)
//! - Optional IR receiver connected to another GPIO pin (default: GPIO27)
//!
//! Make sure to enable the "rpi" feature when building this example:
//! ```
//! cargo build --example raspberry_pi --features rpi
//! ```

#[cfg(feature = "rpi")]
use log;
#[cfg(feature = "rpi")]
use panasonic_ac::{
    hardware::{rpi::RpiHardware, Hardware},
    types::{AcMode, FanSpeed, SwingV},
    AcDevice, Device, PanasonicAc, PanasonicAcModel, Result,
};

#[cfg(feature = "rpi")]
fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    println!("Panasonic AC Raspberry Pi Control Example");
    println!("=========================================");

    // GPIO pin connected to the IR LED and receiver
    const IR_LED_PIN: u8 = 17;
    const IR_RECEIVER_PIN: u8 = 27;

    // Initialize Raspberry Pi hardware
    println!("Initializing hardware...");
    let hardware = match RpiHardware::new_with_receiver(IR_LED_PIN, IR_RECEIVER_PIN) {
        Ok(hw) => {
            println!("Hardware initialized successfully:");
            println!("- IR LED: GPIO{}", IR_LED_PIN);
            println!("- IR Receiver: GPIO{}", IR_RECEIVER_PIN);
            hw
        }
        Err(e) => {
            eprintln!("Failed to initialize hardware: {}", e);
            return Err(e);
        }
    };

    // Create a new AC instance with the hardware
    println!("Creating AC controller...");
    // Create a sender function by cloning the values we need
    let tx_pin = hardware.tx_pin;
    let transmitter = std::sync::Arc::new(std::sync::Mutex::new(None));
    let tx_clone = transmitter.clone();
    
    // Create a sender closure for the AC controller
    let sender = move |data: &[u8], freq: u32, repeat: u16| -> Result<()> {
        // Lazy initialization of the transmitter
        let mut tx_guard = tx_clone.lock().map_err(|e| {
            panasonic_ac::error::Error::HardwareError(format!("Failed to acquire transmitter lock: {}", e))
        })?;
        
        if tx_guard.is_none() {
            *tx_guard = Some(RpiHardware::new(tx_pin)?.create_transmitter()?);
        }
        
        let tx = tx_guard.as_mut().unwrap();
        tx.send(data, freq, repeat)
    };
    
    let mut ac = PanasonicAc::new(sender);

    // Configure the AC model
    println!("Setting up the AC...");
    ac.set_model(PanasonicAcModel::Dke)?;

    println!("AC ready. Starting control sequence...");

    // Example control sequence

    // Turn on with cooling mode
    println!("\n1. Turning on AC in cooling mode");
    ac.set_power(true)?;
    ac.set_mode(AcMode::Cool)?;
    ac.set_temp(24)?;
    ac.set_fan(FanSpeed::Auto)?;
    ac.set_swing_v(SwingV::Auto)?;

    println!("Sending command: {}", ac);
    ac.send()?;

    // Wait for user input before continuing
    println!("\nPress Enter to change temperature...");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input");

    // Change temperature
    println!("2. Changing temperature to 22°C");
    ac.set_temp(22)?;

    println!("Sending command: {}", ac);
    ac.send()?;

    // Wait for user input before continuing
    println!("\nPress Enter to turn on powerful mode...");
    input.clear();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input");

    // Turn on powerful mode
    println!("3. Activating powerful mode");
    ac.set_powerful(true)?;

    println!("Sending command: {}", ac);
    ac.send()?;

    // Wait for user input before continuing
    println!("\nPress Enter to try receiving an IR signal...");
    input.clear();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input");
        
    // Try to receive IR signal
    println!("4. Waiting 10 seconds for IR signal...");
    println!("Please press a button on your remote control");
    
    // Attempt to create a receiver - this might fail if hardware isn't properly configured
    if let Ok(mut receiver) = hardware.create_receiver() {
        match receiver.receive(10000) { // 10 second timeout
            Ok(timings) => {
                println!("Received IR signal with {} timing points!", timings.len());
                println!("First 10 timing points: {:?}", &timings[..10.min(timings.len())]);
                // You could decode this signal if needed
            },
            Err(e) => println!("Error receiving IR signal: {}", e),
        }
    } else {
        eprintln!("Could not initialize receiver");
        println!("\nSkipping receive test.");
    }

    // Wait for user input before continuing
    println!("\nPress Enter to turn off the AC...");
    input.clear();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input");

    // Turn off the AC
    println!("5. Turning off the AC");
    ac.set_power(false)?;

    println!("Sending command: {}", ac);
    ac.send()?;

    println!("\nAll commands completed successfully!");

    Ok(())
}

#[cfg(not(feature = "rpi"))]
fn main() {
    eprintln!("This example requires the 'rpi' feature to be enabled.");
    eprintln!("Please build with: cargo run --example raspberry_pi --features rpi");
}
