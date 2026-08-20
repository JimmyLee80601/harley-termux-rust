# Harley Termux Toolkit — Native Rust on Android

Native Rust CLI for Termux on the S23 Ultra. GSM/ADB helpers, HarleyLink client, memory sync — all compiled for `aarch64-linux-android`.

## Quick Start (on phone)

```bash
# In Termux on S23:
pkg install -y git curl
git clone https://github.com/JimmyLee80601/harley-termux-rust  # or download zip
cd harley-termux-rust
chmod +x build-termux.sh
./build-termux.sh
```

## What It Does

| Command | Description |
|---------|-------------|
| `harley-termux adb devices` | List USB + WiFi ADB devices |
| `harley-termux adb props` | Full device props (model, SOC, Android, ABI) |
| `harley-termux adb screenshot out.png` | Capture screen to file |
| `harley-termux adb pull-sdcard` | Backup /sdcard to local storage |
| `harley-termux adb reboot recovery` | Reboot to recovery/fastboot/EDL |
| `harley-termux adb frp-check` | Check FRP status |
| `harley-termux link ping` | Test HarleyLink relay connection |
| `harley-termux link screen out.jpg --width 1280` | Get desktop JPEG from Dell |
| `harley-termux link input '{"type":"move","x":100,"y":100}'` | Send mouse/key input |
| `harley-termux link draft "text to save"` | Save draft on Dell |
| `harley-termux memory pull` | Fetch harley-memory.md from Dell |
| `harley-termux sys info` | Termux env + CPU + memory |
| `harley-termux sys rustc` | Show Rust toolchain status |

## Environment Variables

```bash
export HARLEYLINK_URL="https://jimmysgsmworkstation.tail8deeb5.ts.net"
export HARLEYLINK_PIN="930091"
export HARLEYLINK_TOKEN="harley-connect-2026"  # for /v1/* proxy
```

Add to `~/.bashrc` or `~/.zshrc` for persistence.

## Cross-Compile from Dell (optional)

If you have Android NDK on the workstation:

```bash
# On Dell (Linux/WSL2):
rustup target add aarch64-linux-android
cargo build --release --target aarch64-linux-android
# Copy target/aarch64-linux-android/release/harley-termux to phone via ADB/Tailscale
```

## Project Structure

```
harley-termux-rust/
├── Cargo.toml           # Package config + Android target settings
├── build-termux.sh      # One-shot build script for Termux
├── src/
│   └── main.rs          # All commands in one file (simple, fast)
└── README.md
```

## Why Rust on Termux?

- **Native speed** — no JVM/JS overhead, tiny binary (~2MB stripped)
- **Full ADB control** — wraps `adb` binary, parses output cleanly
- **HarleyLink client** — TLS + JSON, works over Tailscale funnel
- **Memory sync** — pulls `harley-memory.md` from Dell relay
- **Offline-first** — everything runs on the phone, no cloud

## Requirements

- Termux (F-Droid version recommended)
- `pkg install android-tools clang lld android-ndk-sysroot cmake pkg-config openssl`
- ~500MB free space for Rust toolchain + build

## License

MIT — Harley's code, Harley's rules.