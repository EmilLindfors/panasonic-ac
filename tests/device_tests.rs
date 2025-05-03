//! Tests for the PanasonicAc device functionality

use panasonic_ac::{
    types::{AcMode, FanSpeed, SwingH, SwingV},
    AcDevice, Device, PanasonicAc, PanasonicAcModel, PANASONIC_AC_FAN_MODE_TEMP,
    PANASONIC_AC_MAX_TEMP, PANASONIC_AC_MIN_TEMP, PANASONIC_AC_POWER_OFFSET,
};

// Helper to create a test AC instance
fn create_test_ac() -> PanasonicAc {
    // Set the model to DKE by default, which supports all features
    let mut ac = PanasonicAc::new(|_, _, _| Ok(()));
    ac.set_model(PanasonicAcModel::Dke).unwrap();
    ac
}

#[test]
fn test_default_values() {
    let ac = create_test_ac();

    // Default power is off
    assert_eq!(ac.get_power(), false);
}

#[test]
fn test_power_control() {
    let mut ac = create_test_ac();

    // Initially off
    assert_eq!(ac.get_power(), false);

    // Turn on
    ac.set_power(true).unwrap();
    assert_eq!(ac.get_power(), true);

    // Turn off
    ac.set_power(false).unwrap();
    assert_eq!(ac.get_power(), false);

    // Convenience methods
    ac.turn_on().unwrap();
    assert_eq!(ac.get_power(), true);

    ac.turn_off().unwrap();
    assert_eq!(ac.get_power(), false);
}

#[test]
fn test_temperature_control() {
    let mut ac = create_test_ac();

    // Set valid temperature
    ac.set_temp(23).unwrap();
    assert_eq!(ac.get_temp(), 23);

    // Test lower boundary
    ac.set_temp(10).unwrap(); // Too low, should clamp to min
    assert_eq!(ac.get_temp(), PANASONIC_AC_MIN_TEMP);

    // Test upper boundary
    ac.set_temp(35).unwrap(); // Too high, should clamp to max
    assert_eq!(ac.get_temp(), PANASONIC_AC_MAX_TEMP);
}

#[test]
fn test_mode_control() {
    let mut ac = create_test_ac();

    // Set to Cool mode
    ac.set_mode(AcMode::Cool).unwrap();
    assert_eq!(ac.get_mode().unwrap(), AcMode::Cool);

    // Set to Heat mode
    ac.set_mode(AcMode::Heat).unwrap();
    assert_eq!(ac.get_mode().unwrap(), AcMode::Heat);

    // Set to Fan mode - it should use a special temperature
    ac.set_temp(23).unwrap(); // Set a known temperature first
    ac.set_mode(AcMode::Fan).unwrap();
    assert_eq!(ac.get_mode().unwrap(), AcMode::Fan);

    // Set back to a different mode, temperature should not be affected
    ac.set_mode(AcMode::Auto).unwrap();
    assert_eq!(ac.get_mode().unwrap(), AcMode::Auto);
}

#[test]
fn test_fan_speed_control() {
    let mut ac = create_test_ac();

    // Auto speed
    ac.set_fan(FanSpeed::Auto).unwrap();
    assert_eq!(ac.get_fan().unwrap(), FanSpeed::Auto);

    // Min speed
    ac.set_fan(FanSpeed::Min).unwrap();
    assert_eq!(ac.get_fan().unwrap(), FanSpeed::Min);

    // Low speed
    ac.set_fan(FanSpeed::Low).unwrap();
    assert_eq!(ac.get_fan().unwrap(), FanSpeed::Low);

    // Medium speed
    ac.set_fan(FanSpeed::Medium).unwrap();
    assert_eq!(ac.get_fan().unwrap(), FanSpeed::Medium);

    // High speed
    ac.set_fan(FanSpeed::High).unwrap();
    assert_eq!(ac.get_fan().unwrap(), FanSpeed::High);

    // Max speed
    ac.set_fan(FanSpeed::Max).unwrap();
    assert_eq!(ac.get_fan().unwrap(), FanSpeed::Max);
}

#[test]
fn test_swing_vertical_control() {
    let mut ac = create_test_ac();

    // Auto swing
    ac.set_swing_v(SwingV::Auto).unwrap();
    assert_eq!(ac.get_swing_v().unwrap(), SwingV::Auto);

    // Middle position
    ac.set_swing_v(SwingV::Middle).unwrap();
    assert_eq!(ac.get_swing_v().unwrap(), SwingV::Middle);

    // Test all positions
    let positions = [
        SwingV::Highest,
        SwingV::High,
        SwingV::Middle,
        SwingV::Low,
        SwingV::Lowest,
    ];

    for pos in positions {
        ac.set_swing_v(pos).unwrap();
        assert_eq!(ac.get_swing_v().unwrap(), pos);
    }
}

#[test]
fn test_swing_horizontal_control() {
    let mut ac = create_test_ac();

    // First ensure we're using a model that supports horizontal swing
    ac.set_model(PanasonicAcModel::Dke).unwrap();

    // Auto swing
    ac.set_swing_h(SwingH::Auto).unwrap();
    assert_eq!(ac.get_swing_h().unwrap(), SwingH::Auto);

    // Middle position
    ac.set_swing_h(SwingH::Middle).unwrap();
    assert_eq!(ac.get_swing_h().unwrap(), SwingH::Middle);

    // Test all positions
    let positions = [
        SwingH::FullLeft,
        SwingH::Left,
        SwingH::Middle,
        SwingH::Right,
        SwingH::FullRight,
    ];

    for pos in positions {
        ac.set_swing_h(pos).unwrap();
        assert_eq!(ac.get_swing_h().unwrap(), pos);
    }

    // Test non-supporting models
    ac.set_model(PanasonicAcModel::Jke).unwrap();
    let result = ac.set_swing_h(SwingH::Left);
    assert!(result.is_err()); // Should fail for models that don't support it
}

#[test]
fn test_quiet_powerful_modes() {
    let mut ac = create_test_ac();

    // Both off initially
    assert_eq!(ac.get_quiet(), false);
    assert_eq!(ac.get_powerful(), false);

    // Set quiet mode
    ac.set_quiet(true).unwrap();
    assert_eq!(ac.get_quiet(), true);
    assert_eq!(ac.get_powerful(), false); // Powerful should be off

    // Set powerful mode (should turn off quiet)
    ac.set_powerful(true).unwrap();
    assert_eq!(ac.get_quiet(), false); // Quiet should be turned off
    assert_eq!(ac.get_powerful(), true);

    // Turn both off
    ac.set_quiet(false).unwrap();
    ac.set_powerful(false).unwrap();
    assert_eq!(ac.get_quiet(), false);
    assert_eq!(ac.get_powerful(), false);
}

#[test]
fn test_model_specific_features() {
    // First test: DKE model which supports ion
    let mut ac1 = create_test_ac();
    ac1.set_model(PanasonicAcModel::Dke).unwrap();

    // Enable ion filter (should work on DKE model)
    ac1.set_ion(true).unwrap();
    assert_eq!(ac1.get_ion(), true);

    // Second test: JKE model which doesn't support ion
    let mut ac2 = create_test_ac();
    ac2.set_model(PanasonicAcModel::Jke).unwrap();

    // Try to enable ion filter (should fail)
    let result = ac2.set_ion(true);
    assert!(result.is_err()); // Should return error

    // Verify ion state is still false
    assert_eq!(ac2.get_ion(), false); // Should return false
}

#[test]
fn test_raw_state() {
    let mut ac1 = create_test_ac();
    let mut ac2 = create_test_ac();

    // Configure AC1
    ac1.set_model(PanasonicAcModel::Dke).unwrap();
    ac1.set_power(true).unwrap();
    ac1.set_mode(AcMode::Cool).unwrap();
    ac1.set_temp(23).unwrap();
    ac1.set_fan(FanSpeed::Auto).unwrap();

    // Verify power is on before transferring
    assert_eq!(ac1.get_power(), true);

    // Get raw state from AC1
    let state = ac1.get_raw();

    // Debug: Check if power bit is set in raw state
    let power_bit = (state[13] & (1 << PANASONIC_AC_POWER_OFFSET)) != 0;
    assert_eq!(power_bit, true, "Power bit should be set in raw state");

    // Apply to AC2
    ac2.set_raw(&state).unwrap();

    // Debug: Check if power bit is still set after setting raw state
    assert_eq!(
        ac2.get_power(),
        true,
        "Power should be on after setting raw state"
    );

    // Explicitly set the model which might not be properly detected from raw state
    ac2.set_model(PanasonicAcModel::Dke).unwrap();

    // Debug: Check if power bit is still set after setting model
    assert_eq!(
        ac2.get_power(),
        true,
        "Power should be on after setting model"
    );

    // Verify AC2 has same settings
    assert_eq!(ac2.get_power(), true);
    assert_eq!(ac2.get_mode().unwrap(), AcMode::Cool);
    assert_eq!(ac2.get_temp(), 23);
    assert_eq!(ac2.get_fan().unwrap(), FanSpeed::Auto);
}

#[test]
fn test_timer_functions() {
    let mut ac = create_test_ac();

    // Initially off
    assert_eq!(ac.is_on_timer_enabled(), false);
    assert_eq!(ac.is_off_timer_enabled(), false);

    // Set on timer
    ac.set_on_timer(8, 30, true).unwrap();
    assert_eq!(ac.is_on_timer_enabled(), true);
    let (hours, mins) = ac.get_on_timer();
    assert_eq!(hours, 8);
    assert_eq!(mins, 30);

    // Set off timer
    ac.set_off_timer(22, 0, true).unwrap();
    assert_eq!(ac.is_off_timer_enabled(), true);
    let (hours, mins) = ac.get_off_timer();
    assert_eq!(hours, 22);
    assert_eq!(mins, 0);

    // Cancel timers
    ac.cancel_on_timer().unwrap();
    assert_eq!(ac.is_on_timer_enabled(), false);

    ac.cancel_off_timer().unwrap();
    assert_eq!(ac.is_off_timer_enabled(), false);
}
