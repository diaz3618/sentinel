#!/usr/bin/env bash
set -e

echo "=== Sentinel Installation Script ==="
echo ""

if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
    VERSION=$VERSION_ID
else
    echo "Error: Cannot detect OS"
    exit 1
fi

echo "Detected OS: $OS $VERSION"
echo ""

if [ "$EUID" -ne 0 ]; then 
    echo "Error: This script must be run as root (use sudo)"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "Rust not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust is already installed ($(rustc --version))"
fi

echo ""
echo "Installing system dependencies..."
case "$OS" in
    ubuntu|debian)
        apt-get update
        apt-get install -y build-essential pkg-config libssl-dev
        ;;
    fedora)
        dnf install -y gcc make openssl-devel
        ;;
    centos|rhel)
        yum install -y gcc make openssl-devel
        ;;
    arch|manjaro)
        pacman -Sy --noconfirm base-devel openssl
        ;;
    opensuse*)
        zypper install -y gcc make libopenssl-devel
        ;;
    *)
        echo "Warning: Unsupported distro '$OS'. You may need to install build tools manually."
        ;;
esac

echo ""
echo "Building Sentinel (release mode)..."
cargo build --release --workspace

echo ""
echo "Creating directories..."
mkdir -p /usr/local/bin
mkdir -p /etc

echo "Installing binaries..."
cp target/release/sentinel /usr/local/bin/sentinel
cp target/release/sentinelctl /usr/local/bin/sentinelctl
chmod +x /usr/local/bin/sentinel
chmod +x /usr/local/bin/sentinelctl

if [ ! -f /etc/memsentinel.toml ]; then
    echo "Installing default config..."
    cp packaging/sentinel.example.toml /etc/memsentinel.toml
    chmod 600 /etc/memsentinel.toml
    echo "Config installed at /etc/memsentinel.toml"
else
    echo "Config already exists at /etc/memsentinel.toml (skipping)"
fi

if command -v systemctl &> /dev/null; then
    echo ""
    echo "Installing systemd service..."
    cp packaging/systemd/sentinel.service /etc/systemd/system/sentinel.service
    cp packaging/systemd/sentinel.slice /etc/systemd/system/sentinel.slice
    systemctl daemon-reload
    echo "Systemd service installed (not enabled by default)"
    echo "To enable: sudo systemctl enable --now sentinel"
fi

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Installed:"
echo "  - sentinel daemon: /usr/local/bin/sentinel"
echo "  - sentinelctl CLI: /usr/local/bin/sentinelctl"
echo "  - Config: /etc/memsentinel.toml"
echo ""
echo "Usage:"
echo "  sentinelctl status"
echo "  sentinelctl top --limit 5"
echo "  sudo systemctl enable --now sentinel"
echo ""
echo "For more info, see README.md"
