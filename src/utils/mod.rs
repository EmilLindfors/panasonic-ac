// utils/mod.rs
//! Utility functions and types

pub mod standard;

use crate::error::{Error, Result};

/// Calculate the sum of bytes
pub(crate) fn sum_bytes(data: &[u8], length: usize, initial: u8) -> u8 {
    let mut sum = initial;
    for &byte in data.iter().take(length.min(data.len())) {
        sum = sum.wrapping_add(byte);
    }
    sum
}

/// Convert minutes to a time string
#[must_use]
pub fn mins_to_string(mins: u16) -> String {
    let hours = mins / 60;
    let minutes = mins % 60;
    format!("{hours:02}:{minutes:02}")
}

/// Pad a string with a given character to a minimum length
#[must_use]
pub fn pad_string(input: &str, pad_char: char, width: usize) -> String {
    if input.len() >= width {
        input.to_string()
    } else {
        let padding = width - input.len();
        let padding_str: String = std::iter::repeat_n(pad_char, padding).collect();
        format!("{input}{padding_str}")
    }
}

/// Parse a time string in HH:MM format to minutes
pub fn parse_time(time_str: &str) -> Result<u16> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::InvalidValue(
            format!("Invalid time format: {time_str}, expected HH:MM")
        ));
    }

    let hours = parts[0]
        .parse::<u16>()
        .map_err(|_| {
            let part = parts[0];
            Error::InvalidValue(format!("Invalid hours: {part}"))
        })?;

    let minutes = parts[1]
        .parse::<u16>()
        .map_err(|_| {
            let part = parts[1];
            Error::InvalidValue(format!("Invalid minutes: {part}"))
        })?;

    if hours > 23 || minutes > 59 {
        return Err(Error::InvalidValue(
            format!("Invalid time: {hours}:{minutes}")
        ));
    }

    Ok(hours * 60 + minutes)
}

/// Format a temperature value with the Celsius symbol
#[must_use]
pub fn format_temp(temp: u8) -> String {
    format!("{temp}°C")
}

/// Convert a boolean value to "On" or "Off"
pub const fn bool_to_onoff(value: bool) -> &'static str {
    if value {
        "On"
    } else {
        "Off"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_bytes() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];
        assert_eq!(sum_bytes(&data, 3, 0), 0x06);
        assert_eq!(sum_bytes(&data, 5, 0), 0x0F);
        assert_eq!(sum_bytes(&data, 3, 0x10), 0x16);
    }

    #[test]
    fn test_mins_to_string() {
        assert_eq!(mins_to_string(0), "00:00");
        assert_eq!(mins_to_string(60), "01:00");
        assert_eq!(mins_to_string(75), "01:15");
        assert_eq!(mins_to_string(1439), "23:59");
    }

    #[test]
    fn test_pad_string() {
        assert_eq!(pad_string("test", ' ', 8), "test    ");
        assert_eq!(pad_string("test", '-', 6), "test--");
        assert_eq!(pad_string("test", ' ', 3), "test");
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("00:00").unwrap(), 0);
        assert_eq!(parse_time("01:30").unwrap(), 90);
        assert_eq!(parse_time("23:59").unwrap(), 1439);
        assert!(parse_time("24:00").is_err());
        assert!(parse_time("23:60").is_err());
        assert!(parse_time("invalid").is_err());
    }

    #[test]
    fn test_format_temp() {
        assert_eq!(format_temp(25), "25°C");
        assert_eq!(format_temp(0), "0°C");
    }

    #[test]
    fn test_bool_to_onoff() {
        assert_eq!(bool_to_onoff(true), "On");
        assert_eq!(bool_to_onoff(false), "Off");
    }
}
