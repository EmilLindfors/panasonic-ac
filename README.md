# Panasonic AC Control Library

A modern Rust library for controlling Panasonic heat pumps and air conditioners via IR communication. Designed with safety, flexibility, and hardware support in mind.

## Features

- 📱 Control Panasonic AC units using infrared signals
- 🧩 Modular trait-based architecture for clean abstraction
- 🔄 Complete state management with builder pattern
- 🌡️ Comprehensive support for temperature, modes, fan speeds, and swing settings
- ⏱️ Timer functionality (on/off timers, clock setting)
- 🪄 Special modes (Quiet, Powerful, Ion)
- 🏷️ Support for multiple Panasonic AC models (DKE, JKE, LKE, NKE, CKP, RKR)
- 🖥️ Direct hardware support for Raspberry Pi GPIO
- 🧪 Type-safe API with proper error handling via thiserror
- 📦 Zero-cost abstractions with minimal dependencies
- 🧩 Feature-gated hardware support

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
panasonic-ac = "0.1.0"

# If you need Raspberry Pi hardware support:
panasonic-ac = { version = "0.1.0", features = ["rpi"] }
```

## Basic Usage

```rust
use panasonic_ac::prelude::*;

fn main() -> Result<()> {
    // Create a Panasonic AC instance with a custom IR sender function
    let mut ac = PanasonicAc::new(|data, freq, repeat| {
        // Your IR transmission implementation goes here
        println!("Sending {} bytes at {} Hz ({} repeats)", data.len(), freq, repeat);
        Ok(())
    });
    
    // Configure the AC
    ac.set_model(PanasonicAcModel::Dke)?;
    ac.set_power(true)?;
    ac.set_mode(AcMode::Cool)?;
    ac.set_temp(23)?;
    ac.set_fan(FanSpeed::Auto)?;
    ac.set_swing_v(SwingV::Auto)?;
    
    // Send the IR command
    ac.send()?;
    
    println!("AC Status: {}", ac);
    
    Ok(())
}
```

## Using the Builder Pattern

The library provides a convenient builder pattern for creating states:

```rust
use panasonic_ac::prelude::*;
use panasonic_ac::utils::standard::{OpMode, StateBuilder};

fn main() -> Result<()> {
    let mut ac = PanasonicAc::new(|data, freq, repeat| {
        // IR transmission implementation
        Ok(())
    });
    
    // Use the builder to create a state
    let cooling_state = StateBuilder::new()
        .power(true)
        .mode(OpMode::Cool)
        .temp(23)
        .fan(standard::FanLevel::Auto)
        .swing_v(Some(standard::SwingVPos::Auto))
        .quiet(true)
        .build();
    
    // Apply the state
    ac.set_state(&cooling_state)?;
    ac.send()?;
    
    Ok(())
}
```

## Raspberry Pi Hardware Support

The library provides direct support for controlling AC units via Raspberry Pi GPIO pins:

```rust
use panasonic_ac::prelude::*;
use panasonic_ac::hardware;

fn main() -> Result<()> {
    // Initialize hardware transmitter (default: GPIO pin 17)
    let mut transmitter = hardware::create_default_transmitter()?;
    
    // Create AC with hardware transmitter
    let mut ac = PanasonicAc::new(move |data, freq, repeat| {
        transmitter.send(data, freq, repeat)
    });
    
    // Control your AC
    ac.set_model(PanasonicAcModel::Dke)?;
    ac.set_power(true)?;
    ac.set_mode(AcMode::Cool)?;
    ac.set_temp(23)?;
    ac.send()?;
    
    // You can also receive IR signals (default: GPIO pin 27)
    let mut receiver = hardware::create_default_receiver()?;
    match receiver.receive(5000) { // 5 second timeout
        Ok(timings) => {
            println!("Received signal with {} timing points", timings.len());
            // Process the signal...
        },
        Err(e) => println!("Error receiving IR signal: {}", e),
    }
    
    Ok(())
}
```

### Hardware Setup

For Raspberry Pi, connect your hardware as follows:

**IR Transmitter:**
- Connect an IR LED to GPIO pin 17 (configurable)
- Use a transistor (e.g., 2N2222) to drive the LED for better range
- Add a current-limiting resistor (~100Ω)

**IR Receiver:**
- Connect an IR receiver module (e.g., TSOP38238) to GPIO pin 27 (configurable)
- Connect VCC to 3.3V, GND to ground

## Supported Models

The library supports multiple Panasonic models:

- DKE series
- JKE series
- LKE series
- NKE series
- CKP series
- RKR series

Each model has different feature support. The library will return appropriate errors when attempting to use unsupported features.

## Advanced Features

### Timer Control

```rust
// Set the clock
ac.set_clock(14, 30)?;

// Set On timer - AC will turn on at 17:00
ac.set_on_timer(17, 0, true)?;

// Set Off timer - AC will turn off at 22:30
ac.set_off_timer(22, 30, true)?;
```

### Special Modes

```rust
// Enable powerful mode for maximum cooling/heating
ac.set_powerful(true)?;

// Enable quiet mode for reduced noise
ac.set_quiet(true)?;

// Enable ion filtering (on supported models)
ac.set_ion(true)?;
```

### Raw State Access

```rust
// Get raw state bytes
let raw_state = ac.get_raw();

// Set from raw state
ac.set_raw(&raw_state)?;
```

### IR Signal Decoding

```rust
// Create a decoder
let decoder = PanasonicDecoder::new();

// Decode captured IR signal
let decoded_data = decoder.decode(&captured_signal)?;

// Print decoded state
println!("Decoded state: {:?}", decoded_data);
```

## Architecture Overview

The library is built with a modular architecture:

- **Device Traits**: Core traits defining device operations
- **Protocol Implementation**: Panasonic-specific implementations
- **Encoder/Decoder**: IR signal encoding and decoding
- **Standard Interface**: Common interface for device control
- **Type-safe Enums**: For modes, fan speeds, swing positions, etc.
- **State Machine Support**: Facilitates complex control flows
- **Hardware Abstraction**: Support for different hardware platforms

## Project Status and Roadmap

### Implemented Features
- ✅ Core IR protocol support for all major Panasonic AC models
- ✅ Complete state management with builder pattern
- ✅ Temperature, mode, fan speed, and swing controls
- ✅ Timer functionality and special modes
- ✅ Basic Raspberry Pi hardware support
- ✅ Comprehensive error handling
- ✅ IR signal encoding and decoding

### TODO List
- 🔄 Improve Raspberry Pi hardware support with better GPIO handling
- 🔄 Add better hardware abstraction layer for diverse platforms
- 🔄 Implement comprehensive error handling for hardware failures 
- 🔄 Add support for additional Panasonic AC models
- 🔄 Create more examples and documentation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This library is licensed under the MIT License - see the LICENSE file for details.