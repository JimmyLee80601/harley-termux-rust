#!/data/data/com.termux/files/usr/bin/bash
# build-termux.sh — Build harley-termux for Android/Termux
# Run this INSIDE Termux on the S23

set -euo pipefail

echo "🔨 Building harley-termux for aarch64-linux-android..."

# 1. Install Rust toolchain for Android
if ! command -v rustup &> /dev/null; then
    echo "📦 Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME/.cargo/env"
fi

# Add Android target
rustup target add aarch64-linux-android

# 2. Install Android NDK linker (via termux packages)
echo "📦 Installing build dependencies..."
pkg update -y
pkg install -y clang lld android-ndk-sysroot cmake pkg-config openssl libsqlite

# 3. Set up cargo config for Android
mkdir -p "$HOME/.cargo"
cat > "$HOME/.cargo/config.toml" << 'EOF'
[target.aarch64-linux-android]
linker = "aarch64-linux-android21-clang"
rustflags = ["-C", "target-feature=+neon,+crypto", "-C", "link-arg=-Wl,--as-needed"]

[build]
target = "aarch64-linux-android"
EOF

# 4. Build the project
echo "🔨 Building release binary..."
cargo build --release --target aarch64-linux-android

# 5. Copy to PATH
BINARY="$HOME/.cargo/bin/harley-termux"
TARGET_BIN="target/aarch64-linux-android/release/harley-termux"

if [ -f "$TARGET_BIN" ]; then
    cp "$TARGET_BIN" "$BINARY"
    chmod +x "$BINARY"
    echo "✅ Installed to $BINARY"
    echo ""
    echo "Test it:"
    echo "  harley-termux --help"
    echo "  harley-termux adb devices"
    echo "  harley-termux link ping"
    echo "  harley-termux memory pull"
else
    echo "❌ Build failed - binary not found at $TARGET_BIN"
    exit 1
fi