// builder.rs
//! Builder pattern for creating and configuring Panasonic AC devices

use crate::device::{PanasonicAc, PanasonicAcModel};
use crate::error::Result;
use crate::protocol::{AcDevice, Device, BitOps};
use crate::types::{AcMode, FanSpeed, SwingH, SwingV};
/// Builder for creating and configuring a PanasonicAc instance
#[derive(Debug, Default)]
pub struct PanasonicAcBuilder {
    /// The model of the AC unit
    model: Option<PanasonicAcModel>,
    /// Power state (on/off)
    power: Option<bool>,
    /// Operating mode
    mode: Option<AcMode>,
    /// Temperature in Celsius
    temp: Option<u8>,
    /// Fan speed
    fan: Option<FanSpeed>,
    /// Vertical swing position
    swing_v: Option<SwingV>,
    /// Horizontal swing position
    swing_h: Option<SwingH>,
    /// Quiet mode
    quiet: Option<bool>,
    /// Powerful mode
    powerful: Option<bool>,
    /// Ion filter (only available on some models)
    ion: Option<bool>,
}

impl PanasonicAcBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the AC model
    pub fn model(mut self, model: PanasonicAcModel) -> Self {
        self.model = Some(model);
        self
    }
    
    /// Set the power state
    pub fn power(mut self, on: bool) -> Self {
        self.power = Some(on);
        self
    }
    
    /// Set the operating mode
    pub fn mode(mut self, mode: AcMode) -> Self {
        self.mode = Some(mode);
        self
    }
    
    /// Set the temperature in Celsius
    pub fn temp(mut self, celsius: u8) -> Self {
        self.temp = Some(celsius);
        self
    }
    
    /// Set the fan speed
    pub fn fan(mut self, speed: FanSpeed) -> Self {
        self.fan = Some(speed);
        self
    }
    
    /// Set the vertical swing position
    pub fn swing_v(mut self, position: SwingV) -> Self {
        self.swing_v = Some(position);
        self
    }
    
    /// Set the horizontal swing position
    pub fn swing_h(mut self, position: SwingH) -> Self {
        self.swing_h = Some(position);
        self
    }
    
    /// Set quiet mode
    pub fn quiet(mut self, on: bool) -> Self {
        self.quiet = Some(on);
        self
    }
    
    /// Set powerful mode
    pub fn powerful(mut self, on: bool) -> Self {
        self.powerful = Some(on);
        self
    }
    
    /// Set ion filter mode
    pub fn ion(mut self, on: bool) -> Self {
        self.ion = Some(on);
        self
    }
    
    /// Build the PanasonicAc instance with the configured settings
    pub fn build<F>(self, ir_send_fn: F) -> Result<PanasonicAc>
    where 
        F: Fn(&[u8], u32, u16) -> Result<()> + 'static
    {
        let mut ac = PanasonicAc::new(ir_send_fn);
        
        // Apply settings if provided
        if let Some(model) = self.model {
            ac.set_model(model)?;
        }
        
        if let Some(power) = self.power {
            ac.set_power(power)?;
        }
        
        if let Some(mode) = self.mode {
            ac.set_mode(mode)?;
        }
        
        if let Some(temp) = self.temp {
            // Workaround for setting temperature in builder pattern
            
            // 1. Make sure temp is within valid range
            use std::cmp::{min, max};
            use crate::constants::{PANASONIC_AC_MIN_TEMP, PANASONIC_AC_MAX_TEMP, 
                                  PANASONIC_AC_TEMP_OFFSET, PANASONIC_AC_TEMP_SIZE};
            
            let clamped_temp = max(temp, PANASONIC_AC_MIN_TEMP);
            let clamped_temp = min(clamped_temp, PANASONIC_AC_MAX_TEMP);
            
            // 2. Directly set both the field and the state buffer
            ac.temp = clamped_temp;
            ac.state.set_bits(14, PANASONIC_AC_TEMP_OFFSET, PANASONIC_AC_TEMP_SIZE, clamped_temp);
            
            // 3. Also call the method for possible side effects
            let _ = ac.set_temp(clamped_temp);
        }
        
        if let Some(fan) = self.fan {
            ac.set_fan(fan)?;
        }
        
        if let Some(swing_v) = self.swing_v {
            ac.set_swing_v(swing_v)?;
        }
        
        if let Some(swing_h) = self.swing_h {
            ac.set_swing_h(swing_h)?;
        }
        
        if let Some(quiet) = self.quiet {
            ac.set_quiet(quiet)?;
        }
        
        if let Some(powerful) = self.powerful {
            ac.set_powerful(powerful)?;
        }
        
        if let Some(ion) = self.ion {
            ac.set_ion(ion)?;
        }
        
        Ok(ac)
    }
    
    /// Build the PanasonicAc instance and turn it on with the configured settings
    pub fn build_and_turn_on<F>(self, ir_send_fn: F) -> Result<PanasonicAc>
    where 
        F: Fn(&[u8], u32, u16) -> Result<()> + 'static
    {
        let mut builder = self;
        builder.power = Some(true);
        builder.build(ir_send_fn)
    }
}