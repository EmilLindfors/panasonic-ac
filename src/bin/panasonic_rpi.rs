//! Panasonic AC Controller for Raspberry Pi
//! 
//! This binary application provides a command-line interface for controlling
//! Panasonic air conditioners using a Raspberry Pi with an IR LED.
//!
//! Features:
//! - Control Panasonic AC units from the command line
//! - Set temperature, mode, fan speed, and more
//! - Save and load presets
//! - Decode IR signals received from original remotes
//!
//! Hardware requirements:
//! - Raspberry Pi (any model)
//! - IR LED connected to GPIO pin (default: GPIO 17)
//! - Optional IR receiver connected to GPIO pin (default: GPIO 27)

use std::io::{self, Write};
use std::path::Path;
use std::process;

use clap::{Parser, Subcommand};
use log::{debug, error, info, LevelFilter};

#[cfg(feature = "rpi")]
use panasonic_ac::{
    hardware::{rpi::{RpiHardware, RpiTransmitter}, Hardware, IrTransmitter},
    types::{AcMode, FanSpeed, SwingH, SwingV},
    AcDevice, Device, PanasonicAc, PanasonicAcModel, Result,
};

/// CLI Arguments
#[derive(Parser)]
#[command(name = "panasonic-rpi")]
#[command(about = "Control Panasonic AC units with Raspberry Pi", long_about = None)]
struct Cli {
    /// Increase verbosity (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// GPIO pin for IR transmitter
    #[arg(long, default_value_t = 17)]
    tx_pin: u8,

    /// GPIO pin for IR receiver
    #[arg(long, default_value_t = 27)]
    rx_pin: u8,

    /// AC model
    #[arg(short, long, default_value = "dke")]
    model: String,

    /// Command to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a command to the AC unit
    Send {
        /// Turn the AC on or off
        #[arg(long)]
        power: Option<bool>,

        /// Set the operation mode (auto, cool, heat, dry, fan)
        #[arg(short, long)]
        mode: Option<String>,

        /// Set the temperature (16-30°C)
        #[arg(short, long)]
        temp: Option<u8>,

        /// Set the fan speed (auto, low, medium, high, max)
        #[arg(short, long)]
        fan: Option<String>,

        /// Set the vertical swing mode (auto, highest, high, middle, low, lowest)
        #[arg(long)]
        swing_v: Option<String>,

        /// Set the horizontal swing mode (auto, left, middle, right)
        #[arg(long)]
        swing_h: Option<String>,

        /// Enable/disable powerful mode
        #[arg(long)]
        powerful: Option<bool>,

        /// Enable/disable quiet mode
        #[arg(long)]
        quiet: Option<bool>,
    },

    /// Receive and decode an IR signal
    Receive {
        /// Timeout in seconds
        #[arg(short, long, default_value_t = 10)]
        timeout: u32,
    },

    /// Save current settings as a preset
    SavePreset {
        /// Preset name
        #[arg(short, long)]
        name: String,
    },

    /// Load a preset
    LoadPreset {
        /// Preset name
        #[arg(short, long)]
        name: String,
    },
}

/// Main function that wraps the actual implementation
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// The actual implementation with error handling
#[cfg(feature = "rpi")]
fn run() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Set up logging
    let log_level = match cli.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] {} - {}",
                record.level(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.args()
            )
        })
        .init();

    info!("Panasonic AC Controller for Raspberry Pi starting");
    debug!("Log level set to: {}", log_level);

    // Initialize hardware
    info!("Initializing hardware: TX pin {}, RX pin {}", cli.tx_pin, cli.rx_pin);
    let hardware = match RpiHardware::new_with_receiver(cli.tx_pin, cli.rx_pin) {
        Ok(hw) => hw,
        Err(e) => {
            error!("Failed to initialize hardware: {}", e);
            return Err(e);
        }
    };

    // Parse AC model
    let model = parse_model(&cli.model)?;
    
    // Create AC controller
    info!("Creating AC controller for model: {:?}", model);
    let tx_pin = hardware.tx_pin;
    
    // Create a sender using a pattern similar to the one in the examples
    let sender = move |data: &[u8], freq: u32, repeat: u16| -> Result<()> {
        // Create a new transmitter for each send operation
        let mut transmitter = RpiTransmitter::new(tx_pin)?;
        transmitter.send(data, freq, repeat)
    };
    
    let mut ac = PanasonicAc::new(sender);
    ac.set_model(model)?;

    // Process command
    match &cli.command {
        Commands::Send {
            power,
            mode,
            temp,
            fan,
            swing_v,
            swing_h,
            powerful,
            quiet,
        } => {
            send_command(
                &mut ac, *power, mode, *temp, fan, swing_v, swing_h, *powerful, *quiet,
            )?;
        }
        Commands::Receive { timeout } => {
            receive_ir(&hardware, *timeout)?;
        }
        Commands::SavePreset { name } => {
            save_preset(&ac, name)?;
        }
        Commands::LoadPreset { name } => {
            load_preset(&mut ac, name)?;
        }
    }

    info!("Command executed successfully");
    Ok(())
}

#[cfg(not(feature = "rpi"))]
fn run() -> Result<()> {
    eprintln!("This application requires the 'rpi' feature to be enabled.");
    eprintln!("Please build with: cargo build --bin panasonic_rpi --features rpi");
    process::exit(1);
}

/// Parse AC model from string
#[cfg(feature = "rpi")]
fn parse_model(model_str: &str) -> Result<PanasonicAcModel> {
    match model_str.to_lowercase().as_str() {
        "dke" => Ok(PanasonicAcModel::Dke),
        "jke" => Ok(PanasonicAcModel::Jke),
        "lke" => Ok(PanasonicAcModel::Lke),
        "nke" => Ok(PanasonicAcModel::Nke),
        "ckp" => Ok(PanasonicAcModel::Ckp),
        "rkr" => Ok(PanasonicAcModel::Rkr),
        _ => {
            error!("Invalid AC model: {}", model_str);
            Err(panasonic_ac::error::Error::InvalidValue(format!(
                "Invalid AC model: {}. Valid models are: dke, jke, lke, nke, ckp, rkr",
                model_str
            )))
        }
    }
}

/// Send command to AC unit
#[cfg(feature = "rpi")]
fn send_command(
    ac: &mut PanasonicAc,
    power: Option<bool>,
    mode: &Option<String>,
    temp: Option<u8>,
    fan: &Option<String>,
    swing_v: &Option<String>,
    swing_h: &Option<String>,
    powerful: Option<bool>,
    quiet: Option<bool>,
) -> Result<()> {
    info!("Preparing to send command to AC unit");

    // Set power
    if let Some(p) = power {
        info!("Setting power: {}", p);
        ac.set_power(p)?;
    }

    // Set mode
    if let Some(m) = mode {
        let ac_mode = parse_mode(m)?;
        info!("Setting mode: {:?}", ac_mode);
        ac.set_mode(ac_mode)?;
    }

    // Set temperature
    if let Some(t) = temp {
        info!("Setting temperature: {}°C", t);
        ac.set_temp(t)?;
    }

    // Set fan speed
    if let Some(f) = fan {
        let fan_speed = parse_fan_speed(f)?;
        info!("Setting fan speed: {:?}", fan_speed);
        ac.set_fan(fan_speed)?;
    }

    // Set vertical swing
    if let Some(s) = swing_v {
        let swing = parse_swing_v(s)?;
        info!("Setting vertical swing: {:?}", swing);
        ac.set_swing_v(swing)?;
    }

    // Set horizontal swing
    if let Some(s) = swing_h {
        let swing = parse_swing_h(s)?;
        info!("Setting horizontal swing: {:?}", swing);
        ac.set_swing_h(swing)?;
    }

    // Set powerful mode
    if let Some(p) = powerful {
        info!("Setting powerful mode: {}", p);
        ac.set_powerful(p)?;
    }

    // Set quiet mode
    if let Some(q) = quiet {
        info!("Setting quiet mode: {}", q);
        ac.set_quiet(q)?;
    }

    // Print current state before sending
    info!("Current AC state: {}", ac);

    // Send the command
    info!("Sending IR command...");
    ac.send()?;
    info!("Command sent successfully");

    Ok(())
}

/// Receive and decode IR signal
#[cfg(feature = "rpi")]
fn receive_ir(hardware: &RpiHardware, timeout_seconds: u32) -> Result<()> {
    info!("Initializing IR receiver...");
    let mut receiver = hardware.create_receiver()?;
    
    info!("Waiting for IR signal (timeout: {} seconds)...", timeout_seconds);
    println!("Please press a button on your remote control...");
    
    match receiver.receive(timeout_seconds * 1000) {
        Ok(timings) => {
            info!("Received IR signal with {} timing points", timings.len());
            println!("Received IR signal!");
            println!("Number of timing points: {}", timings.len());
            println!("First 20 timing points: {:?}", &timings[..20.min(timings.len())]);
            
            // TODO: Implement decoding of the IR signal using decoder
            // For now, just save the raw timings
            println!("Saving raw timings to 'captured_signal.txt'...");
            let mut file = std::fs::File::create("captured_signal.txt")?;
            for timing in timings {
                writeln!(file, "{}", timing)?;
            }
            println!("Raw timings saved to 'captured_signal.txt'");
            
            Ok(())
        }
        Err(e) => {
            error!("Error receiving IR signal: {}", e);
            Err(e)
        }
    }
}

/// Save current AC state as a preset
#[cfg(feature = "rpi")]
fn save_preset(ac: &PanasonicAc, name: &str) -> Result<()> {
    info!("Saving preset: {}", name);
    
    // Create presets directory if it doesn't exist
    let presets_dir = Path::new("presets");
    if !presets_dir.exists() {
        info!("Creating presets directory");
        std::fs::create_dir_all(presets_dir)?;
    }
    
    // Get raw state
    let raw_state = ac.get_raw();
    
    // Save to file
    let preset_path = presets_dir.join(format!("{}.preset", name));
    info!("Saving preset to: {:?}", preset_path);
    
    let mut file = std::fs::File::create(&preset_path)?;
    
    // Write model first
    writeln!(file, "{}", ac.get_model().to_string())?;
    
    // Write raw state bytes
    for byte in raw_state {
        write!(file, "{:02X}", byte)?;
    }
    
    info!("Preset saved successfully");
    println!("Preset '{}' saved successfully!", name);
    
    Ok(())
}

/// Load a preset
#[cfg(feature = "rpi")]
fn load_preset(
    ac: &mut PanasonicAc,
    name: &str,
) -> Result<()> {
    info!("Loading preset: {}", name);
    
    // Check if preset exists
    let preset_path = Path::new("presets").join(format!("{}.preset", name));
    if !preset_path.exists() {
        error!("Preset not found: {}", name);
        return Err(panasonic_ac::error::Error::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Preset not found: {}", name),
        )));
    }
    
    info!("Reading preset from: {:?}", preset_path);
    let content = std::fs::read_to_string(&preset_path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.len() < 2 {
        error!("Invalid preset file format");
        return Err(panasonic_ac::error::Error::InvalidData(
            "Invalid preset file format".to_string(),
        ));
    }
    
    // First line is the model
    let model = parse_model(lines[0])?;
    ac.set_model(model)?;
    
    // Second line is the raw state
    let hex_state = lines[1];
    let raw_state = parse_hex_string(hex_state)?;
    
    // Set the raw state
    info!("Setting raw state ({} bytes)", raw_state.len());
    ac.set_raw(&raw_state)?;
    
    info!("Loaded preset successfully: {}", ac);
    println!("Preset '{}' loaded successfully!", name);
    println!("Current AC state: {}", ac);
    
    // Ask if user wants to send command
    print!("Send this command? [y/N]: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() == "y" {
        info!("Sending command...");
        ac.send()?;
        info!("Command sent successfully");
        println!("Command sent!");
    } else {
        info!("Command not sent (user declined)");
        println!("Command not sent.");
    }
    
    Ok(())
}

/// Parse AC mode from string
#[cfg(feature = "rpi")]
fn parse_mode(mode_str: &str) -> Result<AcMode> {
    match mode_str.to_lowercase().as_str() {
        "auto" => Ok(AcMode::Auto),
        "cool" => Ok(AcMode::Cool),
        "heat" => Ok(AcMode::Heat),
        "dry" => Ok(AcMode::Dry),
        "fan" => Ok(AcMode::Fan),
        _ => {
            error!("Invalid mode: {}", mode_str);
            Err(panasonic_ac::error::Error::InvalidValue(format!(
                "Invalid mode: {}. Valid modes are: auto, cool, heat, dry, fan",
                mode_str
            )))
        }
    }
}

/// Parse fan speed from string
#[cfg(feature = "rpi")]
fn parse_fan_speed(fan_str: &str) -> Result<FanSpeed> {
    match fan_str.to_lowercase().as_str() {
        "auto" => Ok(FanSpeed::Auto),
        "min" | "lowest" => Ok(FanSpeed::Min),
        "low" => Ok(FanSpeed::Low),
        "med" | "medium" => Ok(FanSpeed::Medium),
        "high" => Ok(FanSpeed::High),
        "max" | "highest" => Ok(FanSpeed::Max),
        _ => {
            error!("Invalid fan speed: {}", fan_str);
            Err(panasonic_ac::error::Error::InvalidValue(format!(
                "Invalid fan speed: {}. Valid speeds are: auto, min, low, med, high, max",
                fan_str
            )))
        }
    }
}

/// Parse vertical swing from string
#[cfg(feature = "rpi")]
fn parse_swing_v(swing_str: &str) -> Result<SwingV> {
    match swing_str.to_lowercase().as_str() {
        "auto" => Ok(SwingV::Auto),
        "highest" => Ok(SwingV::Highest),
        "high" => Ok(SwingV::High),
        "middle" => Ok(SwingV::Middle),
        "low" => Ok(SwingV::Low),
        "lowest" => Ok(SwingV::Lowest),
        _ => {
            error!("Invalid vertical swing: {}", swing_str);
            Err(panasonic_ac::error::Error::InvalidValue(format!(
                "Invalid vertical swing: {}. Valid values are: auto, highest, high, middle, low, lowest",
                swing_str
            )))
        }
    }
}

/// Parse horizontal swing from string
#[cfg(feature = "rpi")]
fn parse_swing_h(swing_str: &str) -> Result<SwingH> {
    match swing_str.to_lowercase().as_str() {
        "auto" => Ok(SwingH::Auto),
        "left" => Ok(SwingH::Left),
        "left_mid" => Ok(SwingH::Middle), // Using Middle as replacement
        "middle" => Ok(SwingH::Middle),
        "right_mid" => Ok(SwingH::Right), // Using Right as replacement
        "right" => Ok(SwingH::Right),
        _ => {
            error!("Invalid horizontal swing: {}", swing_str);
            Err(panasonic_ac::error::Error::InvalidValue(format!(
                "Invalid horizontal swing: {}. Valid values are: auto, left, left_mid, middle, right_mid, right",
                swing_str
            )))
        }
    }
}

/// Parse hex string to u8 vector
#[cfg(feature = "rpi")]
fn parse_hex_string(s: &str) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    
    for i in (0..s.len()).step_by(2) {
        if i + 2 <= s.len() {
            match u8::from_str_radix(&s[i..i + 2], 16) {
                Ok(byte) => result.push(byte),
                Err(_) => {
                    return Err(panasonic_ac::error::Error::InvalidData(
                        format!("Invalid hex string: {}", s)
                    ));
                }
            }
        }
    }
    
    Ok(result)
}