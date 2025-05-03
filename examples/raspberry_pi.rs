//! Raspberry Pi example for Panasonic AC control
//!
//! This example demonstrates how to control a Panasonic AC unit using a Raspberry Pi.
//!
//! Requirements:
//! - Raspberry Pi (any model)
//! - IR LED connected to a GPIO pin (default: GPIO17)
//! - Optional IR receiver connected to another GPIO pin
//!
//! Make sure to enable the "rpi" feature when building this example:
//! ```
//! cargo build --example raspberry_pi --features rpi
//! ```

#[cfg(feature = "rpi")]
use panasonic_ac::{
    hardware::rpi::RpiHardware,
    types::{AcMode, FanSpeed, SwingV},
    AcDevice, Device, Error, PanasonicAc, PanasonicAcModel, Result,
};

#[cfg(feature = "rpi")]
fn main() -> Result<()> {
    println!("Panasonic AC Raspberry Pi Control Example");
    println!("=========================================");

    // GPIO pin connected to the IR LED
    const IR_LED_PIN: u8 = 17;

    // Initialize Raspberry Pi hardware
    println!("Initializing hardware...");
    let hardware = match RpiHardware::new(IR_LED_PIN) {
        Ok(hw) => {
            println!("Hardware initialized successfully on GPIO{}", IR_LED_PIN);
            hw
        }
        Err(e) => {
            eprintln!("Failed to initialize hardware: {}", e);
            return Err(e);
        }
    };

    // Create a new AC instance with the hardware
    println!("Creating AC controller...");
    let mut ac = PanasonicAc::new(hardware.create_sender());

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
    println!("\nPress Enter to turn off the AC...");
    input.clear();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input");

    // Turn off the AC
    println!("4. Turning off the AC");
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
