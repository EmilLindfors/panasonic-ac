// diff.rs
//! State diffing implementation for efficient updates

use crate::types::{AcMode, FanSpeed, SwingH, SwingV};

/// Changes that can be applied to a Panasonic AC device
#[derive(Debug, Default, Clone)]
pub struct PanasonicAcChanges {
    /// Power state change
    pub power: Option<bool>,
    /// Mode change
    pub mode: Option<AcMode>,
    /// Temperature change
    pub temp: Option<u8>,
    /// Fan speed change
    pub fan: Option<FanSpeed>,
    /// Vertical swing change
    pub swing_v: Option<SwingV>,
    /// Horizontal swing change
    pub swing_h: Option<SwingH>,
    /// Quiet mode change
    pub quiet: Option<bool>,
    /// Powerful mode change
    pub powerful: Option<bool>,
    /// Ion filter change
    pub ion: Option<bool>,
}

impl PanasonicAcChanges {
    /// Create a new empty changes set
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a builder for changes
    pub fn builder() -> PanasonicAcChangesBuilder {
        PanasonicAcChangesBuilder::new()
    }
    
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.power.is_some() || 
        self.mode.is_some() || 
        self.temp.is_some() || 
        self.fan.is_some() || 
        self.swing_v.is_some() || 
        self.swing_h.is_some() || 
        self.quiet.is_some() || 
        self.powerful.is_some() || 
        self.ion.is_some()
    }
}

/// Builder for creating PanasonicAcChanges
#[derive(Debug, Default)]
pub struct PanasonicAcChangesBuilder {
    changes: PanasonicAcChanges,
}

impl PanasonicAcChangesBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set power state change
    pub fn power(mut self, on: bool) -> Self {
        self.changes.power = Some(on);
        self
    }
    
    /// Set mode change
    pub fn mode(mut self, mode: AcMode) -> Self {
        self.changes.mode = Some(mode);
        self
    }
    
    /// Set temperature change
    pub fn temp(mut self, temp: u8) -> Self {
        self.changes.temp = Some(temp);
        self
    }
    
    /// Set fan speed change
    pub fn fan(mut self, fan: FanSpeed) -> Self {
        self.changes.fan = Some(fan);
        self
    }
    
    /// Set vertical swing change
    pub fn swing_v(mut self, swing_v: SwingV) -> Self {
        self.changes.swing_v = Some(swing_v);
        self
    }
    
    /// Set horizontal swing change
    pub fn swing_h(mut self, swing_h: SwingH) -> Self {
        self.changes.swing_h = Some(swing_h);
        self
    }
    
    /// Set quiet mode change
    pub fn quiet(mut self, on: bool) -> Self {
        self.changes.quiet = Some(on);
        self
    }
    
    /// Set powerful mode change
    pub fn powerful(mut self, on: bool) -> Self {
        self.changes.powerful = Some(on);
        self
    }
    
    /// Set ion filter change
    pub fn ion(mut self, on: bool) -> Self {
        self.changes.ion = Some(on);
        self
    }
    
    /// Build the changes
    pub fn build(self) -> PanasonicAcChanges {
        self.changes
    }
}