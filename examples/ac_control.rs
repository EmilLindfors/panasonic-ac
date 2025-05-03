use panasonic_ac::{
    types::{AcMode, FanSpeed, SwingH, SwingV},
    AcDevice, Device, PanasonicAc, PanasonicAcModel, Result,
};
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<()> {
    println!("Panasonic AC Control Example with State-Based Design");

    // Create a Panasonic AC instance with a custom IR sender
    // In a real application, this would connect to actual hardware
    let mut ac = PanasonicAc::new(|data, freq, _repeat| {
        println!("Sending IR signal: {} bytes at {} Hz", data.len(), freq);
        // In a real application, this would send the IR signal using hardware
        // For example, using a GPIO pin connected to an IR LED
        Ok(())
    });

    // Set the model - necessary for enabling model-specific features
    ac.set_model(PanasonicAcModel::Dke)?;
    println!("AC Model: {}", ac.get_model());

    // =====================================================
    // Scenario 1: Basic Control
    // =====================================================
    println!("\n=== Basic Control ===");

    // Turn on with cooling mode
    configure_cooling(&mut ac)?;

    // Display the current state
    println!("Current state: {}", ac);

    // Send the command to the AC
    ac.send()?;

    // =====================================================
    // Scenario 2: Creating Different Presets
    // =====================================================
    println!("\n=== Preset: Night Mode ===");
    sleep(Duration::from_secs(1));

    // Configure night mode preset
    configure_night_mode(&mut ac)?;
    println!("Night mode state: {}", ac);
    ac.send()?;

    // =====================================================
    // Scenario 3: Incremental Changes
    // =====================================================
    println!("\n=== Incremental Changes ===");
    sleep(Duration::from_secs(1));

    // Adjust temperature
    ac.set_temp(24)?;
    println!("After temperature change: {}", ac);
    ac.send()?;

    sleep(Duration::from_secs(1));

    // Adjust fan speed
    ac.set_fan(FanSpeed::High)?;
    println!("After fan speed change: {}", ac);
    ac.send()?;

    // =====================================================
    // Scenario 4: Using the Builder Pattern
    // =====================================================
    println!("\n=== Using Builder Pattern ===");
    sleep(Duration::from_secs(1));

    // Create a new AC instance using the builder pattern
    let builder_ac = PanasonicAc::builder()
        .model(PanasonicAcModel::Dke)
        .power(true)
        .mode(AcMode::Heat)
        .temp(25)
        .fan(FanSpeed::Auto)
        .swing_v(SwingV::Middle)
        .swing_h(SwingH::Auto)
        .quiet(true)
        .build(|data, freq, _repeat| {
            println!("Builder AC sending: {} bytes at {} Hz", data.len(), freq);
            Ok(())
        })?;

    println!("Builder created state: {}", builder_ac);
    builder_ac.send()?;

    // =====================================================
    // Scenario 5: State Diffing for Efficient Updates
    // =====================================================
    println!("\n=== State Diffing ===");
    sleep(Duration::from_secs(1));

    // Create changes to apply
    use panasonic_ac::PanasonicAcChanges;

    let changes = PanasonicAcChanges {
        power: Some(true),
        mode: Some(AcMode::Cool),
        temp: Some(22),
        fan: None,     // Keep existing fan speed
        swing_v: None, // Keep existing vertical swing
        swing_h: None, // Keep existing horizontal swing
        quiet: Some(false),
        powerful: Some(true), // Enable powerful mode
        ion: None,            // Keep existing ion setting
    };

    // Apply changes and send only if something changed
    let changed = ac.apply_changes_and_send(&changes)?;
    println!("Changes applied and sent: {}", changed);
    println!("Current state after changes: {}", ac);

    // =====================================================
    // Scenario 6: Turning Off
    // =====================================================
    println!("\n=== Turning Off ===");
    sleep(Duration::from_secs(1));

    ac.turn_off()?;
    println!("AC turned off: {}", ac);
    ac.send()?;

    println!("\nExample completed successfully!");

    Ok(())
}

// Function to configure cooling mode
fn configure_cooling(ac: &mut PanasonicAc) -> Result<()> {
    ac.set_power(true)?;
    ac.set_mode(AcMode::Cool)?;
    ac.set_temp(22)?;
    ac.set_fan(FanSpeed::Auto)?;
    ac.set_swing_v(SwingV::Auto)?;
    ac.set_swing_h(SwingH::Middle)?;
    ac.set_quiet(false)?;
    ac.set_powerful(false)?;
    ac.set_ion(true)?;
    Ok(())
}

// Function to configure night mode
fn configure_night_mode(ac: &mut PanasonicAc) -> Result<()> {
    ac.set_power(true)?;
    ac.set_mode(AcMode::Cool)?;
    ac.set_temp(26)?;
    ac.set_fan(FanSpeed::Low)?;
    ac.set_swing_v(SwingV::High)?;
    ac.set_swing_h(SwingH::Middle)?;
    ac.set_quiet(true)?;
    ac.set_powerful(false)?;
    ac.set_ion(false)?;
    Ok(())
}
