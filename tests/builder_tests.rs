//! Tests for the builder pattern implementation

#[cfg(feature = "builder")]
mod builder_tests {
    use panasonic_ac::{
        types::{AcMode, FanSpeed, SwingH, SwingV},
        AcDevice, Device, PanasonicAc, PanasonicAcBuilder, PanasonicAcModel,
    };

    #[test]
    fn test_builder_fluent_api() {
        let ac = PanasonicAc::builder()
            .model(PanasonicAcModel::Dke)
            .power(true)
            .mode(AcMode::Cool)
            .temp(23)
            .fan(FanSpeed::Auto)
            .swing_v(SwingV::Auto)
            .swing_h(SwingH::Auto)
            .build(|_, _, _| Ok(()))
            .unwrap();

        // Verify the settings were applied
        assert_eq!(ac.get_model(), PanasonicAcModel::Dke);
        assert_eq!(ac.get_power(), true);
        assert_eq!(ac.get_mode().unwrap(), AcMode::Cool);
        assert_eq!(ac.get_temp(), 23);
        assert_eq!(ac.get_fan().unwrap(), FanSpeed::Auto);
        assert_eq!(ac.get_swing_v().unwrap(), SwingV::Auto);
        assert_eq!(ac.get_swing_h().unwrap(), SwingH::Auto);
    }

    #[test]
    fn test_builder_partial_configuration() {
        // Only setting some properties
        let ac = PanasonicAc::builder()
            .model(PanasonicAcModel::Dke)
            .temp(22)
            .build(|_, _, _| Ok(()))
            .unwrap();

        // Verify the set properties
        assert_eq!(ac.get_model(), PanasonicAcModel::Dke);
        assert_eq!(ac.get_temp(), 22);

        // Other properties should have defaults
        assert_eq!(ac.get_power(), false); // Default is off
    }

    #[test]
    fn test_builder_turn_on() {
        // Using the convenience build_and_turn_on method
        let ac = PanasonicAc::builder()
            .model(PanasonicAcModel::Dke)
            .mode(AcMode::Heat)
            .temp(24)
            .build_and_turn_on(|_, _, _| Ok(()))
            .unwrap();

        // Power should be on regardless of what was set in builder
        assert_eq!(ac.get_power(), true);

        // Other settings should be applied
        assert_eq!(ac.get_model(), PanasonicAcModel::Dke);
        assert_eq!(ac.get_mode().unwrap(), AcMode::Heat);
        assert_eq!(ac.get_temp(), 24);
    }
}
