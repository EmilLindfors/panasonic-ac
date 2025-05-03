//! Tests for the state diffing functionality

#[cfg(feature = "diff")]
mod diff_tests {
    use panasonic_ac::{
        types::{AcMode, FanSpeed, SwingH, SwingV},
        AcDevice, Device, PanasonicAc, PanasonicAcChanges, PanasonicAcChangesBuilder,
        PanasonicAcModel,
    };

    // Helper to create a test AC instance
    fn create_test_ac() -> PanasonicAc {
        let mut ac = PanasonicAc::new(|_, _, _| Ok(()));
        // Set to DKE model for full feature support
        let _ = ac.set_model(PanasonicAcModel::Dke);
        ac
    }

    #[test]
    fn test_changes_builder() {
        let changes = PanasonicAcChangesBuilder::new()
            .power(true)
            .mode(AcMode::Cool)
            .temp(23)
            .fan(FanSpeed::Auto)
            .build();

        assert_eq!(changes.power, Some(true));
        assert_eq!(changes.mode, Some(AcMode::Cool));
        assert_eq!(changes.temp, Some(23));
        assert_eq!(changes.fan, Some(FanSpeed::Auto));
        assert_eq!(changes.swing_v, None); // Not set
        assert_eq!(changes.swing_h, None); // Not set
    }

    #[test]
    fn test_has_changes() {
        // Empty changes
        let empty_changes = PanasonicAcChanges::new();
        assert_eq!(empty_changes.has_changes(), false);

        // With changes
        let changes = PanasonicAcChangesBuilder::new().power(true).build();

        assert_eq!(changes.has_changes(), true);
    }

    #[test]
    fn test_apply_changes() {
        let mut ac = create_test_ac();

        // Initial state
        ac.set_power(false).unwrap();
        ac.set_temp(25).unwrap();

        // Create changes
        let changes = PanasonicAcChangesBuilder::new()
            .power(true)
            .temp(23)
            .build();

        // Apply changes
        let changed = ac.apply_changes(&changes).unwrap();

        // Verify changes were applied
        assert_eq!(changed, true);
        assert_eq!(ac.get_power(), true);
        assert_eq!(ac.get_temp(), 23);
    }

    #[test]
    fn test_apply_no_effective_changes() {
        let mut ac = create_test_ac();

        // Initial state
        ac.set_power(true).unwrap();
        ac.set_temp(23).unwrap();

        // Create changes with same values
        let changes = PanasonicAcChangesBuilder::new()
            .power(true)
            .temp(23)
            .build();

        // Apply changes - should return false since no actual changes were made
        let changed = ac.apply_changes(&changes).unwrap();

        // Verify no changes were needed
        assert_eq!(changed, false);
    }

    #[test]
    fn test_partial_changes() {
        let mut ac = create_test_ac();

        // Initial state
        ac.set_power(false).unwrap();
        ac.set_mode(AcMode::Auto).unwrap();
        ac.set_temp(25).unwrap();
        ac.set_fan(FanSpeed::Auto).unwrap();

        // Create partial changes (only power and temp)
        let changes = PanasonicAcChangesBuilder::new()
            .power(true)
            .temp(23)
            .build();

        // Apply changes
        ac.apply_changes(&changes).unwrap();

        // Verify only specified changes were applied
        assert_eq!(ac.get_power(), true);
        assert_eq!(ac.get_temp(), 23);
        assert_eq!(ac.get_mode().unwrap(), AcMode::Auto); // Unchanged
        assert_eq!(ac.get_fan().unwrap(), FanSpeed::Auto); // Unchanged
    }

    #[test]
    fn test_send_only_when_needed() {
        // Create a value that can be shared with the closure
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
        let counter_clone = counter.clone();

        let mut ac = PanasonicAc::new(move |_, _, _| {
            // Use a mutex to safely modify the counter
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            Ok(())
        });

        ac.set_model(PanasonicAcModel::Dke).unwrap();

        // Initial state
        ac.set_power(false).unwrap();
        ac.set_temp(25).unwrap();

        // Reset counter
        *counter.lock().unwrap() = 0;

        // Case 1: Changes that actually modify the state
        let changes1 = PanasonicAcChangesBuilder::new().power(true).build();

        let result1 = ac.apply_changes_and_send(&changes1).unwrap();
        assert_eq!(result1, true); // Changes were made
        assert_eq!(*counter.lock().unwrap(), 1); // Command was sent

        // Case 2: Changes that don't modify anything
        let changes2 = PanasonicAcChangesBuilder::new()
            .power(true) // Already true from previous change
            .build();

        let result2 = ac.apply_changes_and_send(&changes2).unwrap();
        assert_eq!(result2, false); // No changes were needed
        assert_eq!(*counter.lock().unwrap(), 1); // Counter should still be 1 (no additional send)
    }
}
