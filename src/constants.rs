// constants.rs
//! Constants used by the Panasonic AC protocol

// Timing constants (microseconds)
pub const PANASONIC_HDR_MARK: u16 = 3456;
pub const PANASONIC_HDR_SPACE: u16 = 1728;
pub const PANASONIC_BIT_MARK: u16 = 432;
pub const PANASONIC_ONE_SPACE: u16 = 1296;
pub const PANASONIC_ZERO_SPACE: u16 = 432;
pub const PANASONIC_MIN_COMMAND_LENGTH: u32 = 163_296;
pub const PANASONIC_END_GAP: u16 = 5000;
pub const PANASONIC_MIN_GAP: u32 = 74736;

pub const PANASONIC_AC_SECTION_GAP: u16 = 10000;
pub const PANASONIC_AC_SECTION1_LENGTH: u16 = 8;
pub const PANASONIC_AC_MESSAGE_GAP: u32 = 50000; // Default message gap

// Panasonic AC32 timing constants
pub const PANASONIC_AC32_HDR_MARK: u16 = 3543;
pub const PANASONIC_AC32_BIT_MARK: u16 = 920;
pub const PANASONIC_AC32_HDR_SPACE: u16 = 3450;
pub const PANASONIC_AC32_ONE_SPACE: u16 = 2575;
pub const PANASONIC_AC32_ZERO_SPACE: u16 = 828;
pub const PANASONIC_AC32_SECTION_GAP: u16 = 13946;
pub const PANASONIC_AC32_SECTIONS: u8 = 2;
pub const PANASONIC_AC32_BLOCKS_PER_SECTION: u8 = 2;

// Frequency
pub const PANASONIC_FREQ: u32 = 36700; // 36.7 kHz

// State lengths and checksums
pub const PANASONIC_AC_STATE_LENGTH: usize = 27;
pub const PANASONIC_AC_CHECKSUM_INIT: u8 = 0;

// Bit/byte offsets and positions
pub const PANASONIC_AC_POWER_OFFSET: u8 = 0;
pub const PANASONIC_AC_MODE_OFFSET: u8 = 4;
pub const PANASONIC_AC_MODE_SIZE: u8 = 4; // High nibble of byte 13

pub const PANASONIC_AC_TEMP_OFFSET: u8 = 0;
pub const PANASONIC_AC_TEMP_SIZE: u8 = 8;

pub const PANASONIC_AC_SWINGV_OFFSET: u8 = 0;
pub const PANASONIC_AC_SWINGV_SIZE: u8 = 4; // Low nibble of byte 16

pub const PANASONIC_AC_SWINGH_OFFSET: u8 = 0;
pub const PANASONIC_AC_SWINGH_SIZE: u8 = 4; // Low nibble of byte 17

pub const PANASONIC_AC_FAN_OFFSET: u8 = 4;
pub const PANASONIC_AC_FAN_SIZE: u8 = 4; // High nibble of byte 16

// Temperature bounds
pub const PANASONIC_AC_MIN_TEMP: u8 = 16; // Celsius
pub const PANASONIC_AC_MAX_TEMP: u8 = 30; // Celsius
pub const PANASONIC_AC_FAN_MODE_TEMP: u8 = 27; // Celsius

// Mode values
pub const PANASONIC_AC_AUTO: u8 = 0;
pub const PANASONIC_AC_COOL: u8 = 2;
pub const PANASONIC_AC_DRY: u8 = 3;
pub const PANASONIC_AC_HEAT: u8 = 4;
pub const PANASONIC_AC_FAN: u8 = 6;

// Fan speed values
pub const PANASONIC_AC_FAN_AUTO: u8 = 0;
pub const PANASONIC_AC_FAN_MIN: u8 = 1;
pub const PANASONIC_AC_FAN_LOW: u8 = 2;
pub const PANASONIC_AC_FAN_MED: u8 = 3;
pub const PANASONIC_AC_FAN_HIGH: u8 = 4;
pub const PANASONIC_AC_FAN_MAX: u8 = 5;
pub const PANASONIC_AC_FAN_DELTA: u8 = 3;

// Swing values
pub const PANASONIC_AC_SWING_V_AUTO: u8 = 0xF;
pub const PANASONIC_AC_SWING_V_HIGHEST: u8 = 0x1;
pub const PANASONIC_AC_SWING_V_HIGH: u8 = 0x2;
pub const PANASONIC_AC_SWING_V_MIDDLE: u8 = 0x3;
pub const PANASONIC_AC_SWING_V_LOW: u8 = 0x4;
pub const PANASONIC_AC_SWING_V_LOWEST: u8 = 0x5;

pub const PANASONIC_AC_SWING_H_AUTO: u8 = 0xD;
pub const PANASONIC_AC_SWING_H_MIDDLE: u8 = 0x6;
pub const PANASONIC_AC_SWING_H_FULL_LEFT: u8 = 0x9;
pub const PANASONIC_AC_SWING_H_LEFT: u8 = 0xA;
pub const PANASONIC_AC_SWING_H_RIGHT: u8 = 0xB;
pub const PANASONIC_AC_SWING_H_FULL_RIGHT: u8 = 0xC;

// Feature byte offsets
pub const PANASONIC_AC_QUIET_OFFSET: u8 = 0;
pub const PANASONIC_AC_QUIET_CKP_OFFSET: u8 = 3;
pub const PANASONIC_AC_POWERFUL_OFFSET: u8 = 5;
pub const PANASONIC_AC_POWERFUL_CKP_OFFSET: u8 = 7;
pub const PANASONIC_AC_ION_FILTER_BYTE: usize = 22;
pub const PANASONIC_AC_ION_FILTER_OFFSET: u8 = 0;

// Timer related
pub const PANASONIC_AC_ON_TIMER_OFFSET: u8 = 1;
pub const PANASONIC_AC_OFF_TIMER_OFFSET: u8 = 2;
pub const PANASONIC_AC_TIME_SIZE: u8 = 11;
pub const PANASONIC_AC_TIME_OVERFLOW_SIZE: u8 = 3;
pub const PANASONIC_AC_TIME_SPECIAL: u16 = 0x600;
pub const PANASONIC_AC_TIME_MAX: u16 = 23 * 60 + 59; // 23:59

// Known good initial state (with correct checksum in last byte)
pub const PANASONIC_KNOWN_GOOD_STATE: [u8; PANASONIC_AC_STATE_LENGTH] = [
    0x02, 0x20, 0xE0, 0x04, 0x00, 0x00, 0x00, 0x06, 0x02, 0x20, 0xE0, 0x04, 0x00, 0x00, 0x00, 0x80,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x92, // 0x92 is the correct checksum (lowbyte of sum)
];

// AC32 constants
pub const PANASONIC_AC32_KNOWN_GOOD: u32 = 0x4004_0190;

pub const PANASONIC_AC32_MIN_TEMP: u8 = 16; // Celsius
pub const PANASONIC_AC32_MAX_TEMP: u8 = 30; // Celsius

// AC32 Mode values
pub const PANASONIC_AC32_AUTO: u8 = 0x0;
pub const PANASONIC_AC32_COOL: u8 = 0x1;
pub const PANASONIC_AC32_DRY: u8 = 0x2;
pub const PANASONIC_AC32_HEAT: u8 = 0x3;
pub const PANASONIC_AC32_FAN: u8 = 0x4;

// AC32 Fan speed values
pub const PANASONIC_AC32_FAN_AUTO: u8 = 0x0;
pub const PANASONIC_AC32_FAN_MIN: u8 = 0x1;
pub const PANASONIC_AC32_FAN_LOW: u8 = 0x2;
pub const PANASONIC_AC32_FAN_MED: u8 = 0x3;
pub const PANASONIC_AC32_FAN_HIGH: u8 = 0x4;
pub const PANASONIC_AC32_FAN_MAX: u8 = 0x5;

// AC32 Swing values
pub const PANASONIC_AC32_SWING_V_AUTO: u8 = 0xF;
