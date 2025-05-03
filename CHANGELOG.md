# Changelog

## [0.2.2] - 2025-05-04

### Added
- Added AArch64 (64-bit ARM) support for Raspberry Pi
- Improved installation script to detect CPU architecture
- Added automatic binary selection based on architecture

## [0.2.1] - 2025-05-04

### Fixed
- Fixed Clippy warnings and compiler issues for CI
- Fixed incorrect enum references in CLI tests
- Updated CI workflow to properly build and test with features
- Improved system dependencies installation in CI
- Fixed binary build on CI

## [0.2.0] - 2025-05-03

### Added
- Improved Raspberry Pi hardware support with better GPIO handling
- Added better hardware abstraction layer for diverse platforms
- Implemented comprehensive error handling for hardware failures
- Created RpiHardware struct for easier initialization of hardware components
- Added new command-line binary for controlling AC units from Raspberry Pi
- Added support for saving and loading presets in binary application
- Added IR signal reception capabilities
- Added comprehensive tests for Raspberry Pi implementation

### Fixed
- Fixed RpiTransmitter to properly use the encoder for correct timing
- Fixed memory management issues with hardware closures

## [0.1.0] - 2025-05-01

### Added
- Initial release
- Core IR protocol support for all major Panasonic AC models
- Complete state management with builder pattern
- Temperature, mode, fan speed, and swing controls
- Timer functionality and special modes
- Basic Raspberry Pi hardware support
- Comprehensive error handling
- IR signal encoding and decoding