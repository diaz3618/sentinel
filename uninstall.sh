#!/usr/bin/env bash
set -e

echo "=== Sentinel Uninstallation Script ==="
echo ""

if [ "$EUID" -ne 0 ]; then 
    echo "Error: This script must be run as root (use sudo)"
    exit 1
fi

if command -v systemctl &> /dev/null; then
    if systemctl is-active --quiet sentinel; then
        echo "Stopping sentinel service..."
        systemctl stop sentinel
    fi
    if systemctl is-enabled --quiet sentinel 2>/dev/null; then
        echo "Disabling sentinel service..."
        systemctl disable sentinel
    fi
fi

echo "Removing binaries..."
rm -f /usr/local/bin/sentinel
rm -f /usr/local/bin/sentinelctl

if command -v systemctl &> /dev/null; then
    echo "Removing systemd files..."
    rm -f /etc/systemd/system/sentinel.service
    rm -f /etc/systemd/system/sentinel.slice
    systemctl daemon-reload
fi

echo ""
read -p "Remove config file /etc/memsentinel.toml? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -f /etc/memsentinel.toml
    echo "Config removed"
else
    echo "Config kept at /etc/memsentinel.toml"
fi

echo ""
echo "=== Uninstallation Complete ==="
