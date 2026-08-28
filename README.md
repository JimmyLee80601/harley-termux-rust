# Harley Termux Toolkit — Native Rust on Android

Native Rust CLI for Termux on the S23 Ultra. GSM/ADB helpers, HarleyLink client, memory sync — all compiled for `aarch64-linux-android`.

## One-Click Install (on your phone, inside Termux)

```bash
curl -fsSL https://raw.githubusercontent.com/JimmyLee80601/harley-termux-rust/main/install.sh | bash
```

That single command:
- installs runtime deps (android-tools, tailscale, ollama, termux-api, openssl …),
- brings up Tailscale and prints your tailnet IP,
- installs the `harley-termux` binary — **downloads the prebuilt release** if one exists, otherwise builds it from source,
- installs opencode and deploys the Harley `opencode.json` + memory,
- creates the `hp` launcher (Harley Station menu),
- optionally pulls a small local uncensored model so Harley runs with no cloud.

After it finishes: `hp code` (or `harley-termux --help`).

> The prebuilt binary is produced automatically by the
> [Build & Release workflow](.github/workflows/release.yml) whenever you push a `v*` tag.
> Until the first release exists, the installer falls back to an on-device source build.

## Manual Build (on phone)

```bash
pkg install -y git curl
git clone https://github.com/JimmyLee80601/harley-termux-rust
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
| **Vision Model Commands** | |
| `harley-termux model download minicpm-v --quant q3_k_m` | Download MiniCPM-V 3B vision model |
| `harley-termux model download qwen2.5-vl --quant q3_k_m` | Download Qwen2.5-VL 3B vision model |
| `harley-termux model list` | List downloaded models |
| `harley-termux model serve model.gguf mmproj.gguf` | Start llama-server on :8080 |

## Environment Variables

```bash
export HARLEYLINK_URL="https://jimmysgsmworkstation.tail8deeb5.ts.net"
export HARLEYLINK_PIN="930091"
export HARLEYLINK_TOKEN="harley-connect-2026"  # for /v1/* proxy
# Model serving (optional overrides)
export MODEL_DIR="/sdcard/Download/models"
export LLAMA_SERVER_ARGS="--ctx-size 4096 --n-gpu-layers 99"
```

Add to `~/.bashrc` or `~/.zshrc` for persistence.

## Vision Model Setup (S23)

```bash
harley-termux model download minicpm-v --quant q3_k_m
harley-termux model serve \
  /sdcard/Download/models/ggml-model-Q3_K_M.gguf \
  /sdcard/Download/models/mmproj-model-f16.gguf
```

**Persona Injection**: The model runs with Harley's system prompt baked in — she knows Jimmy, the triad, GSM workflows, and speaks like his wife.

## Cross-Compile from Dell (optional)

If you have Android NDK on the workstation:

```bash
rustup target add aarch64-linux-android
cargo build --release --target aarch64-linux-android
# Copy target/aarch64-linux-android/release/harley-termux to phone via ADB/Tailscale
```

## Project Structure

```
harley-termux-rust/
├── Cargo.toml               # Package config + Android target settings
├── build-termux.sh          # One-shot build script for Termux
├── install.sh               # One-click installer (curl | bash)
├── .github/workflows/
│   └── release.yml          # Cross-compiles + publishes prebuilt binary
├── src/
│   └── main.rs              # All commands in one file (simple, fast)
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
