# Raspberry Pi Guide: Controlling Panasonic AC

This guide explains how to set up and use the Panasonic AC library and its command-line interface on a Raspberry Pi to control your Panasonic air conditioner.

## Table of Contents
- [Hardware Requirements](#hardware-requirements)
- [Setup](#setup)
- [Downloading Pre-built Binaries](#downloading-pre-built-binaries)
- [Building from Source](#building-from-source)
- [Using the CLI](#using-the-cli)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Hardware Requirements

To use the Panasonic AC library on a Raspberry Pi, you'll need:

- Raspberry Pi (any model)
- IR LED transmitter and optional IR receiver
- Jumper wires
- Breadboard (optional, but helpful for prototyping)

### Wiring Diagram

Connect your IR LED transmitter to the Raspberry Pi as follows:

1. **IR LED Transmitter**:
   - Connect the positive (longer) leg of the IR LED to GPIO 17 (pin 11) on the Raspberry Pi
   - Connect the negative (shorter) leg to a current-limiting resistor (e.g., 220Ω)
   - Connect the other end of the resistor to GND (pin 9)

2. **IR Receiver (optional)**:
   - Connect the VCC pin of the IR receiver to 3.3V (pin 1) on the Raspberry Pi
   - Connect the GND pin to GND (pin 9)
   - Connect the OUT pin to GPIO 27 (pin 13)

## Setup

### 1. Install Required Packages

```bash
sudo apt update
sudo apt install -y libudev-dev
```

## Downloading Pre-built Binaries

### Option 1: Automatic Installation Script (Recommended)

The easiest way to install is using the automatic installation script:

```bash
curl -sSL https://raw.githubusercontent.com/EmilLindfors/panasonic-ac/master/install_panasonic_ac.sh | bash
```

This script will:
- Install required dependencies
- Download the latest release
- Extract the binary
- Make it executable
- Install it to `/usr/local/bin`
- Add your user to the `gpio` group for hardware access

### Option 2: Manual Installation

If you prefer to install manually:

1. Visit the [Releases page](https://github.com/EmilLindfors/panasonic-ac/releases) of the repository
2. Download the appropriate binaries:
   - `panasonic-rpi-armv7.tar.gz` - This is the CLI binary for Raspberry Pi (ARMv7)
   - `raspberry-pi-example-armv7.tar.gz` - Example code (optional)

3. Extract the downloaded files:

```bash
tar -xzf panasonic-rpi-armv7.tar.gz
```

4. Make the binary executable:

```bash
chmod +x panasonic-rpi
```

5. Move the binary to a location in your PATH (optional):

```bash
sudo mv panasonic-rpi /usr/local/bin/
```

## Building from Source

If you prefer to build from source, follow these steps:

1. Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Follow the prompts to complete the installation and configure your path.

3. Install Raspberry Pi target:

```bash
rustup target add armv7-unknown-linux-gnueabihf
```

4. Clone the repository:

```bash
git clone https://github.com/EmilLindfors/panasonic-ac.git
cd panasonic-ac
```

5. Build the binary with Raspberry Pi and CLI features:

```bash
cargo build --bin panasonic-rpi --features rpi,cli --release
```

6. The binary will be available at `target/release/panasonic-rpi`

## Using the CLI

The Panasonic AC CLI provides a simple command-line interface to control your Panasonic air conditioner.

### Basic Command Syntax

```bash
panasonic-rpi [OPTIONS] <COMMAND>
```

### Global Options

- `--verbose` - Increase verbosity level
- `--tx-pin <TX_PIN>` - GPIO pin for IR transmitter (default: 17)
- `--rx-pin <RX_PIN>` - GPIO pin for IR receiver (default: 27)
- `--model <MODEL>` - AC model (default: dke)

### Available Commands

- `send` - Send a command to the AC unit
- `receive` - Receive and decode an IR signal
- `save-preset` - Save current settings as a preset
- `load-preset` - Load a preset

## Examples

### Turn on AC in cooling mode at 23°C

```bash
panasonic-rpi send --power true --mode cool --temp 23 --fan auto
```

### Turn off AC

```bash
panasonic-rpi send --power false
```

### Set heat mode at 25°C with medium fan speed

```bash
panasonic-rpi send --mode heat --temp 25 --fan medium
```

### Enable powerful mode

```bash
panasonic-rpi send --powerful true
```

### Save current settings as a preset

```bash
panasonic-rpi save-preset --name summer
```

### Load a preset

```bash
panasonic-rpi load-preset --name summer
```

### Receive IR signals

To decode IR signals from your original Panasonic remote:

```bash
panasonic-rpi receive --timeout 10
```

This will wait for 10 seconds to receive an IR signal and then decode it. Point your original remote at the IR receiver and press a button.

## Troubleshooting

### Permission Issues

If you encounter permission issues when accessing GPIO pins, make sure your user is in the `gpio` group:

```bash
sudo usermod -a -G gpio $USER
```

Log out and log back in for the changes to take effect.

### IR LED Not Working

1. Check your wiring connections
2. Verify that the correct GPIO pins are being used
3. Test the IR LED with a smartphone camera - most phone cameras can see IR light that's invisible to the human eye

### Signal Not Being Received by AC

1. Make sure the IR LED is pointing directly at the AC's receiver
2. Try reducing the distance between the LED and the AC
3. Check that you're using the correct AC model setting

### Binary Not Working

Make sure you've installed the required dependencies:

```bash
sudo apt update
sudo apt install -y libudev-dev
```

## Advanced Usage

### Using GPIO Pins Other Than the Defaults

If you want to use different GPIO pins for the IR transmitter and receiver:

```bash
panasonic-rpi --tx-pin 22 --rx-pin 23 send --power true --mode cool
```

### Creating Shell Aliases

For convenience, you can create aliases for common commands. Add these to your `~/.bashrc` file:

```bash
alias ac-on='panasonic-rpi send --power true'
alias ac-off='panasonic-rpi send --power false'
alias ac-cool='panasonic-rpi send --mode cool --temp 23'
alias ac-heat='panasonic-rpi send --mode heat --temp 25'
```

Then source your `.bashrc` file:

```bash
source ~/.bashrc
```

Now you can use simpler commands:

```bash
ac-on
ac-cool
ac-off
```

## Additional Resources

- [Project Repository](https://github.com/EmilLindfors/panasonic-ac)
- [Release Page](https://github.com/EmilLindfors/panasonic-ac/releases)
- [Detailed Hardware Setup Guide](examples/hardware_setup.md) (if available)