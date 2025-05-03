#!/bin/bash
# Panasonic AC Control Installation Script for Raspberry Pi
# This script automates the installation of the panasonic-ac CLI tool

set -e  # Exit on error

echo "===== Panasonic AC Controller Installer ====="
echo "This script will install the Panasonic AC controller CLI on your Raspberry Pi."

# Check if running on a Raspberry Pi
if ! grep -q "Raspberry Pi" /proc/device-tree/model 2>/dev/null; then
    echo "WARNING: This doesn't appear to be a Raspberry Pi. The software may not work correctly."
    echo "Do you want to continue anyway? (y/N)"
    read response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 1
    fi
fi

# Install dependencies
echo "Installing required dependencies..."
sudo apt update
sudo apt install -y libudev-dev wget

# Create temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Get latest release version
echo "Checking for latest release..."
LATEST_RELEASE_URL="https://github.com/EmilLindfors/panasonic-ac/releases/latest"
REDIRECT_URL=$(curl -s -L -I -o /dev/null -w '%{url_effective}' "$LATEST_RELEASE_URL")
VERSION=$(echo "$REDIRECT_URL" | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' || echo "v0.2.1")

echo "Latest version: $VERSION"

# Download binary
BINARY_URL="https://github.com/EmilLindfors/panasonic-ac/releases/download/$VERSION/panasonic-rpi-armv7.tar.gz"
echo "Downloading binary from $BINARY_URL..."
wget -q "$BINARY_URL" -O panasonic-rpi-armv7.tar.gz

# Extract binary
echo "Extracting binary..."
tar -xzf panasonic-rpi-armv7.tar.gz

# Find the binary - handle different archive structures
echo "Locating binary..."
if [ -f panasonic-rpi-armv7/panasonic-rpi ]; then
    BINARY_PATH="panasonic-rpi-armv7/panasonic-rpi"
elif [ -f panasonic-rpi ]; then
    BINARY_PATH="panasonic-rpi"
else
    # Search for the binary
    BINARY_PATH=$(find . -type f -name "panasonic-rpi" | head -n 1)
    
    if [ -z "$BINARY_PATH" ]; then
        echo "ERROR: Could not find the binary in the extracted files."
        echo "Contents of the extracted archive:"
        find . -type f | sort
        exit 1
    fi
fi

echo "Found binary at: $BINARY_PATH"

# Make binary executable
echo "Making binary executable..."
chmod +x "$BINARY_PATH"

# Move to /usr/local/bin
echo "Installing binary to /usr/local/bin..."
sudo mv "$BINARY_PATH" /usr/local/bin/

# Clean up
cd - > /dev/null
rm -rf "$TMP_DIR"

echo "Testing installation..."
if command -v panasonic-rpi >/dev/null 2>&1; then
    echo "Panasonic AC controller installed successfully!"
    
    # Test if binary runs
    echo "Testing binary execution..."
    RESULT=$(panasonic-rpi --help 2>&1 || echo "FAILED")
    
    if [[ "$RESULT" == *"FAILED"* ]]; then
        echo "WARNING: The binary was installed but it may not be compatible with your system."
        echo "Error running binary: $RESULT"
        
        # Check architecture
        echo "Your system architecture: $(uname -m)"
        echo "This binary is built for armv7. If your Raspberry Pi is using a different architecture,"
        echo "you may need to build from source instead."
        
        echo ""
        echo "Would you like to build from source instead? (y/N)"
        read response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            echo "Installing Rust and building from source..."
            sudo apt update
            sudo apt install -y build-essential git curl
            
            # Install Rust
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
            
            # Clone repository
            git clone https://github.com/EmilLindfors/panasonic-ac.git
            cd panasonic-ac
            
            # Build binary
            echo "Building panasonic-rpi (this may take several minutes)..."
            cargo build --bin panasonic-rpi --features rpi,cli --release
            
            # Install binary
            echo "Installing binary to /usr/local/bin..."
            sudo cp target/release/panasonic-rpi /usr/local/bin/
            
            echo "Build from source completed!"
        fi
    else
        echo "Binary executes successfully!"
        echo "Usage example: panasonic-rpi send --power true --mode cool --temp 23"
        echo "For more information, run: panasonic-rpi --help"
    fi
else
    echo "ERROR: Installation failed. The binary is not in the PATH."
    exit 1
fi

# Add user to gpio group if it exists
if getent group gpio >/dev/null; then
    echo "Adding user to gpio group for GPIO access..."
    sudo usermod -a -G gpio "$USER"
    echo "NOTE: You may need to log out and log back in for group changes to take effect."
fi

echo ""
echo "To control your Panasonic AC:"
echo "1. Connect an IR LED to GPIO pin 17 (configurable with --tx-pin option)"
echo "2. For receiving IR signals, connect an IR receiver to GPIO pin 27 (--rx-pin option)"
echo ""
echo "Installation complete!"