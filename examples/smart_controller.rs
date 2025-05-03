use panasonic_ac::{
    types::{AcMode, FanSpeed, SwingH, SwingV},
    AcDevice, Device, PanasonicAc, PanasonicAcModel, Result,
};
use std::thread::sleep;
use std::time::Duration;

// Define states for our AC controller state machine
enum AcState {
    Off,
    Cooling,
    Heating,
    Dehumidifying,
    Fan,
    NightMode,
    EcoMode,
}

// Smart controller that manages state transitions
struct SmartController {
    ac: PanasonicAc,
    current_state: AcState,
    room_temp: f32,
    target_temp: f32,
    humidity: f32,
}

impl SmartController {
    fn new() -> Result<Self> {
        // Create a new AC with a mock sender
        let ac = PanasonicAc::new(|data, freq, _repeat| {
            println!("IR Signal: {} bytes at {} Hz", data.len(), freq);
            Ok(())
        });

        Ok(Self {
            ac,
            current_state: AcState::Off,
            room_temp: 25.0,
            target_temp: 23.0,
            humidity: 60.0,
        })
    }

    fn initialize(&mut self) -> Result<()> {
        // Set the model to enable all features
        self.ac.set_model(PanasonicAcModel::Dke)?;
        println!(
            "SmartController initialized with model: {}",
            self.ac.get_model()
        );
        Ok(())
    }

    fn set_target_temp(&mut self, temp: f32) -> Result<()> {
        self.target_temp = temp;
        println!("Target temperature set to {:.1}°C", self.target_temp);

        // Automatically transition to appropriate state based on new target
        self.update_state_for_conditions()?;

        Ok(())
    }

    fn update_room_conditions(&mut self, temp: f32, humidity: f32) -> Result<()> {
        self.room_temp = temp;
        self.humidity = humidity;
        println!(
            "Room conditions updated: {:.1}°C, {:.1}% humidity",
            self.room_temp, self.humidity
        );

        // Check if we need to change state based on new conditions
        self.update_state_for_conditions()?;

        Ok(())
    }

    fn update_state_for_conditions(&mut self) -> Result<()> {
        // Only update if the AC is on
        if matches!(self.current_state, AcState::Off) {
            return Ok(());
        }

        // Determine appropriate state based on current conditions
        let new_state = if self.humidity > 70.0 {
            // High humidity - prioritize dehumidification
            AcState::Dehumidifying
        } else if (self.room_temp - self.target_temp).abs() < 0.5 {
            // Near target temperature - use fan mode to maintain
            AcState::Fan
        } else if self.room_temp > self.target_temp + 0.5 {
            // Room too warm - cooling needed
            AcState::Cooling
        } else if self.room_temp < self.target_temp - 0.5 {
            // Room too cool - heating needed
            AcState::Heating
        } else {
            // No change needed
            return Ok(());
        };

        // Transition to the new state if different from current
        if std::mem::discriminant(&self.current_state) != std::mem::discriminant(&new_state) {
            self.transition_to(new_state)?;
        }

        Ok(())
    }

    fn transition_to(&mut self, state: AcState) -> Result<()> {
        println!("Transitioning to new state...");

        match state {
            AcState::Off => {
                self.ac.set_power(false)?;
            }
            AcState::Cooling => {
                self.ac.set_power(true)?;
                self.ac.set_mode(AcMode::Cool)?;
                self.ac.set_temp(self.target_temp.round() as u8)?;
                self.ac.set_fan(FanSpeed::Auto)?;
                self.ac.set_swing_v(SwingV::Auto)?;
                self.ac.set_swing_h(SwingH::Middle)?;
                self.ac.set_quiet(false)?;
                self.ac
                    .set_powerful(self.room_temp - self.target_temp > 3.0)?;
            }
            AcState::Heating => {
                self.ac.set_power(true)?;
                self.ac.set_mode(AcMode::Heat)?;
                self.ac.set_temp(self.target_temp.round() as u8)?;
                self.ac.set_fan(FanSpeed::Auto)?;
                self.ac.set_swing_v(SwingV::Middle)?;
                self.ac.set_swing_h(SwingH::Middle)?;
                self.ac.set_quiet(false)?;
                self.ac
                    .set_powerful(self.target_temp - self.room_temp > 3.0)?;
            }
            AcState::Dehumidifying => {
                self.ac.set_power(true)?;
                self.ac.set_mode(AcMode::Dry)?;
                self.ac.set_temp(24)?; // Standard temp for dehumidification
                self.ac.set_fan(FanSpeed::Low)?;
                self.ac.set_swing_v(SwingV::Auto)?;
                self.ac.set_swing_h(SwingH::Middle)?;
                self.ac.set_quiet(true)?;
                self.ac.set_powerful(false)?;
            }
            AcState::Fan => {
                self.ac.set_power(true)?;
                self.ac.set_mode(AcMode::Fan)?;
                self.ac.set_fan(FanSpeed::Low)?;
                self.ac.set_swing_v(SwingV::Auto)?;
                self.ac.set_swing_h(SwingH::Middle)?;
                self.ac.set_quiet(true)?;
                self.ac.set_powerful(false)?;
            }
            AcState::NightMode => {
                self.ac.set_power(true)?;
                // Use cooling or heating depending on target vs room temp
                if self.room_temp > self.target_temp {
                    self.ac.set_mode(AcMode::Cool)?;
                    // Set night temperature slightly higher for comfort and energy saving
                    self.ac.set_temp((self.target_temp + 1.0).round() as u8)?;
                } else {
                    self.ac.set_mode(AcMode::Heat)?;
                    // Set night temperature slightly lower for comfort and energy saving
                    self.ac.set_temp((self.target_temp - 1.0).round() as u8)?;
                }
                self.ac.set_fan(FanSpeed::Low)?;
                self.ac.set_swing_v(SwingV::Middle)?;
                self.ac.set_swing_h(SwingH::Middle)?;
                self.ac.set_quiet(true)?;
                self.ac.set_powerful(false)?;
            }
            AcState::EcoMode => {
                self.ac.set_power(true)?;
                self.ac.set_mode(AcMode::Auto)?;
                self.ac.set_temp(25)?; // Eco-friendly temperature
                self.ac.set_fan(FanSpeed::Low)?;
                self.ac.set_swing_v(SwingV::Auto)?;
                self.ac.set_swing_h(SwingH::Middle)?;
                self.ac.set_quiet(true)?;
                self.ac.set_powerful(false)?;
            }
        }

        // Send the command
        self.ac.send()?;

        // Update current state
        self.current_state = state;

        // Display the new configuration
        println!("New state applied: {}", self.ac);

        Ok(())
    }

    fn power_on(&mut self) -> Result<()> {
        // Determine the best mode based on current conditions
        let initial_state = if self.humidity > 70.0 {
            AcState::Dehumidifying
        } else if self.room_temp > self.target_temp + 0.5 {
            AcState::Cooling
        } else if self.room_temp < self.target_temp - 0.5 {
            AcState::Heating
        } else {
            AcState::Fan
        };

        self.transition_to(initial_state)
    }

    fn power_off(&mut self) -> Result<()> {
        self.transition_to(AcState::Off)
    }

    fn night_mode(&mut self) -> Result<()> {
        self.transition_to(AcState::NightMode)
    }

    fn eco_mode(&mut self) -> Result<()> {
        self.transition_to(AcState::EcoMode)
    }
}

fn main() -> Result<()> {
    println!("Panasonic AC Smart Controller Example");
    println!("=====================================");

    // Create and initialize the controller
    let mut controller = SmartController::new()?;
    controller.initialize()?;

    // Simulate a day of temperature variations

    // Morning: Starting at 25°C, target 23°C
    println!("\n🌄 6:00 AM - Morning");
    controller.set_target_temp(23.0)?;
    controller.power_on()?;

    // Simulate cooling down
    sleep(Duration::from_secs(1));
    controller.update_room_conditions(24.0, 60.0)?;

    sleep(Duration::from_secs(1));
    controller.update_room_conditions(23.2, 58.0)?;

    sleep(Duration::from_secs(1));
    controller.update_room_conditions(23.0, 55.0)?;

    // Midday: Temperature rises
    println!("\n☀️ 12:00 PM - Midday");
    controller.update_room_conditions(26.0, 65.0)?;

    sleep(Duration::from_secs(1));
    // Set a new target temperature
    controller.set_target_temp(24.0)?;

    // Evening: Switch to eco mode
    println!("\n🌆 6:00 PM - Evening");
    controller.eco_mode()?;

    sleep(Duration::from_secs(1));
    controller.update_room_conditions(24.5, 62.0)?;

    // Night: Bedtime
    println!("\n🌙 10:00 PM - Night");
    controller.night_mode()?;

    sleep(Duration::from_secs(1));
    controller.update_room_conditions(24.0, 60.0)?;

    sleep(Duration::from_secs(1));
    // A bit chilly in the middle of the night
    controller.update_room_conditions(22.0, 58.0)?;

    // Next morning: Turn off
    println!("\n🌄 Next morning - Shutting down");
    controller.power_off()?;

    println!("\nExample completed successfully!");

    Ok(())
}
