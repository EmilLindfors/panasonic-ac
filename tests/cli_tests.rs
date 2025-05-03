//! Tests for the CLI binary
//! 
//! These tests focus on testing the command-line interface functionality
//! without requiring actual hardware.

#[cfg(all(test, feature = "cli", feature = "rpi"))]
mod cli_tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use panasonic_ac::error::Result;
    use panasonic_ac::types::{AcMode, FanSpeed, SwingV, SwingH};
    
    // Mock function to store commands for testing
    fn create_test_sender() -> (impl Fn(&[u8], u32, u16) -> Result<()>, Arc<Mutex<Vec<Vec<u8>>>>) {
        let commands = Arc::new(Mutex::new(Vec::new()));
        let commands_clone = commands.clone();
        
        let sender = move |data: &[u8], _freq: u32, _repeat: u16| -> Result<()> {
            commands.lock().unwrap().push(data.to_vec());
            Ok(())
        };
        
        (sender, commands_clone)
    }
    
    // Helper function to parse AcMode from string
    fn parse_mode(mode_str: &str) -> Result<AcMode> {
        match mode_str.to_lowercase().as_str() {
            "auto" => Ok(AcMode::Auto),
            "cool" => Ok(AcMode::Cool),
            "heat" => Ok(AcMode::Heat),
            "dry" => Ok(AcMode::Dry),
            "fan" => Ok(AcMode::Fan),
            _ => Err(panasonic_ac::error::Error::InvalidValue(
                format!("Invalid mode: {}", mode_str)
            )),
        }
    }
    
    // Helper function to parse fan speed from string
    fn parse_fan_speed(fan_str: &str) -> Result<FanSpeed> {
        match fan_str.to_lowercase().as_str() {
            "auto" => Ok(FanSpeed::Auto),
            "min" | "lowest" => Ok(FanSpeed::Min),
            "low" => Ok(FanSpeed::Low),
            "med" | "medium" => Ok(FanSpeed::Med),
            "high" => Ok(FanSpeed::High),
            "max" | "highest" => Ok(FanSpeed::Max),
            _ => Err(panasonic_ac::error::Error::InvalidValue(
                format!("Invalid fan speed: {}", fan_str)
            )),
        }
    }
    
    // Helper function to parse vertical swing from string
    fn parse_swing_v(swing_str: &str) -> Result<SwingV> {
        match swing_str.to_lowercase().as_str() {
            "auto" => Ok(SwingV::Auto),
            "highest" => Ok(SwingV::Highest),
            "high" => Ok(SwingV::High),
            "middle" => Ok(SwingV::Middle),
            "low" => Ok(SwingV::Low),
            "lowest" => Ok(SwingV::Lowest),
            _ => Err(panasonic_ac::error::Error::InvalidValue(
                format!("Invalid vertical swing: {}", swing_str)
            )),
        }
    }
    
    // Helper function to parse horizontal swing from string
    fn parse_swing_h(swing_str: &str) -> Result<SwingH> {
        match swing_str.to_lowercase().as_str() {
            "auto" => Ok(SwingH::Auto),
            "left" => Ok(SwingH::Left),
            "left_mid" => Ok(SwingH::LeftMid),
            "middle" => Ok(SwingH::Middle),
            "right_mid" => Ok(SwingH::RightMid),
            "right" => Ok(SwingH::Right),
            _ => Err(panasonic_ac::error::Error::InvalidValue(
                format!("Invalid horizontal swing: {}", swing_str)
            )),
        }
    }
    
    #[test]
    fn test_parse_mode() {
        assert_eq!(parse_mode("auto").unwrap(), AcMode::Auto);
        assert_eq!(parse_mode("cool").unwrap(), AcMode::Cool);
        assert_eq!(parse_mode("heat").unwrap(), AcMode::Heat);
        assert_eq!(parse_mode("dry").unwrap(), AcMode::Dry);
        assert_eq!(parse_mode("fan").unwrap(), AcMode::Fan);
        
        // Test case insensitivity
        assert_eq!(parse_mode("AUTO").unwrap(), AcMode::Auto);
        assert_eq!(parse_mode("Cool").unwrap(), AcMode::Cool);
        
        // Test invalid modes
        assert!(parse_mode("invalid").is_err());
    }
    
    #[test]
    fn test_parse_fan_speed() {
        assert_eq!(parse_fan_speed("auto").unwrap(), FanSpeed::Auto);
        assert_eq!(parse_fan_speed("min").unwrap(), FanSpeed::Min);
        assert_eq!(parse_fan_speed("lowest").unwrap(), FanSpeed::Min);
        assert_eq!(parse_fan_speed("low").unwrap(), FanSpeed::Low);
        assert_eq!(parse_fan_speed("med").unwrap(), FanSpeed::Med);
        assert_eq!(parse_fan_speed("medium").unwrap(), FanSpeed::Med);
        assert_eq!(parse_fan_speed("high").unwrap(), FanSpeed::High);
        assert_eq!(parse_fan_speed("max").unwrap(), FanSpeed::Max);
        assert_eq!(parse_fan_speed("highest").unwrap(), FanSpeed::Max);
        
        // Test invalid fan speeds
        assert!(parse_fan_speed("invalid").is_err());
    }
    
    #[test]
    fn test_parse_swing_v() {
        assert_eq!(parse_swing_v("auto").unwrap(), SwingV::Auto);
        assert_eq!(parse_swing_v("highest").unwrap(), SwingV::Highest);
        assert_eq!(parse_swing_v("high").unwrap(), SwingV::High);
        assert_eq!(parse_swing_v("middle").unwrap(), SwingV::Middle);
        assert_eq!(parse_swing_v("low").unwrap(), SwingV::Low);
        assert_eq!(parse_swing_v("lowest").unwrap(), SwingV::Lowest);
        
        // Test invalid swing settings
        assert!(parse_swing_v("invalid").is_err());
    }
    
    #[test]
    fn test_parse_swing_h() {
        assert_eq!(parse_swing_h("auto").unwrap(), SwingH::Auto);
        assert_eq!(parse_swing_h("left").unwrap(), SwingH::Left);
        assert_eq!(parse_swing_h("left_mid").unwrap(), SwingH::LeftMid);
        assert_eq!(parse_swing_h("middle").unwrap(), SwingH::Middle);
        assert_eq!(parse_swing_h("right_mid").unwrap(), SwingH::RightMid);
        assert_eq!(parse_swing_h("right").unwrap(), SwingH::Right);
        
        // Test invalid swing settings
        assert!(parse_swing_h("invalid").is_err());
    }
    
    #[test]
    fn test_preset_path() {
        // Test that preset path format is as expected
        let name = "cooling";
        let presets_dir = PathBuf::from("presets");
        let preset_path = presets_dir.join(format!("{}.preset", name));
        
        assert_eq!(preset_path.to_str().unwrap(), "presets/cooling.preset");
    }
    
    #[test]
    fn test_create_test_sender() {
        // Test our test helper
        let (sender, commands) = create_test_sender();
        
        // Send some test data
        sender(&[0x01, 0x02, 0x03], 36700, 0).unwrap();
        sender(&[0x04, 0x05, 0x06], 36700, 0).unwrap();
        
        // Verify data was captured
        let commands = commands.lock().unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], vec![0x01, 0x02, 0x03]);
        assert_eq!(commands[1], vec![0x04, 0x05, 0x06]);
    }
}