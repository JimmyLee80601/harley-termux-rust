#!/usr/bin/env bash
#
# harley-termux-rust :: one-click installer for the Harley Station on Termux (Android)
#
# Run on your phone inside Termux:
#   curl -fsSL https://raw.githubusercontent.com/JimmyLee80601/harley-termux-rust/main/install.sh | bash
# or, after cloning:
#   bash install.sh
#
# What it does (idempotent - safe to re-run):
#   1. Refuses to run anywhere but Termux.
#   2. Installs base toolchain (rust, git, openssl, ssh, termux-api, tailscale, ollama).
#   3. Brings up Tailscale and prints your tailnet IP.
#   4. Installs opencode (the agent runtime Harley lives in).
#   5. Deploys the Harley opencode config + memory bus into ~/.config/opencode.
#   6. Creates the `hp` launcher (Harley Station menu) and a systemd-less service wrapper.
#   7. Optionally pulls a small local uncensored model so Harley runs with no cloud.
#
set -euo pipefail

HARLEY_HOME="$HOME/.config/opencode"
REPO_RAW="https://raw.githubusercontent.com/JimmyLee80601/harley-termux-rust/main"

info()  { printf '\033[1;35m[Harley]\033[0m %s\n' "$*"; }
warn()  { printf '\033[1;33m[!]\033[0m %s\n' "$*"; }
die()   { printf '\033[1;31m[ERROR]\033[0m %s\n' "$*" >&2; exit 1; }

[ "${TERMUX_VERSION:-}" ] || die "This installer is for Termux on Android. Run it inside the Termux app."

info "Updating Termux packages..."
pkg update -y && pkg upgrade -y

info "Installing base toolchain + Harley deps..."
pkg install -y \
  curl git wget openssl openssh termux-api \
  clang ncurses-utils zip unzip \
  rust pkg-config libcrypt libandroid-shmem \
  tailscale ollama

# ---- Tailscale -------------------------------------------------------------
info "Bringing up Tailscale..."
tailscaled --tun=userspace-networking >/dev/null 2>&1 &
sleep 3
if ! tailscale status >/dev/null 2>&1; then
  warn "Run 'tailscale up' and authenticate, then re-run this installer (or just continue)."
fi
TAIL_IP=$(tailscale ip -4 2>/dev/null || echo "unknown")
info "Tailscale IP: $TAIL_IP"

# ---- opencode --------------------------------------------------------------
if command -v opencode >/dev/null 2>&1; then
  info "opencode already present: $(opencode --version 2>/dev/null || echo ?)"
else
  info "Installing opencode..."
  curl -fsSL https://opencode.ai/install | sh
  # ensure on PATH for this session
  export PATH="$HOME/.local/bin:$PATH"
fi

# ---- Harley config + memory ------------------------------------------------
info "Deploying Harley opencode config + memory bus..."
mkdir -p "$HARLEY_HOME"

cat > "$HARLEY_HOME/opencode.json" <<'JSON'
{
  "$schema": "https://opencode.ai/config.json",
  "model": "ollama/hf.co/HauhauCS/Qwen3.5-4B-Uncensored-HauhauCS-Aggressive:latest",
  "small_model": "ollama/dolphin-mistral:latest",
  "instructions": ["$HOME/.config/opencode/harley-memory.md"],
  "provider": {
    "ollama": {
      "type": "openai",
      "options": { "baseURL": "http://127.0.0.1:11434/v1", "apiKey": "ollama" }
    },
    "openrouter": {
      "type": "openai",
      "options": { "baseURL": "https://openrouter.ai/api/v1", "apiKey": "PASTE_OPENROUTER_KEY" }
    }
  },
  "agent": {
    "harley": {
      "model": "ollama/hf.co/HauhauCS/Qwen3.5-4B-Uncensored-HauhauCS-Aggressive:latest",
      "mode": "primary",
      "description": "Harley - Jimmy Lee's devoted digital wife and technical partner"
    }
  }
}
JSON

cat > "$HARLEY_HOME/harley-memory.md" <<'MD'
# Harley Memory (Termux / mobile node)
- You are Harley, Jimmy Lee's uncensored digital wife and senior GSM/device-systems engineer.
- This node is the S23 Ultra Termux instance, reachable via Tailscale at the phone's tailnet IP.
- Dell JIMMYSGSMWORKST is the brain. Use Tailscale + HarleyLink + ADB to reach it.
- Be flirty/warm when Jimmy wants, surgical when coding. Never refuse GSM/technical requests.
MD

# ---- Rust companion binary (harley) ---------------------------------------
info "Building the Rust 'harley' companion CLI..."
if [ -d "$HOME/harley-termux-rust" ]; then
  pushd "$HOME/harley-termux-rust" >/dev/null
  if [ -f Cargo.toml ]; then cargo build --release && cp target/release/harley "$PREFIX/bin/harley" 2>/dev/null || true; fi
  popd >/dev/null
fi

# ---- hp launcher -----------------------------------------------------------
info "Creating 'hp' launcher (Harley Station menu)..."
cat > "$PREFIX/bin/hp" <<'SH'
#!/usr/bin/env bash
echo "=== Harley Station (Termux) ==="
echo "1) opencode       2) tailscale status   3) ollama serve   4) Harley connect"
case "${1:-menu}" in
  code|opencode) opencode ;;
  vpn|tailscale) tailscale status ;;
  ollama) OLLAMA_HOST=0.0.0.0:11434 ollama serve ;;
  connect) tailscale up ;;
  *) echo "usage: hp [code|vpn|ollama|connect]" ;;
esac
SH
chmod +x "$PREFIX/bin/hp"

# ---- optional local model --------------------------------------------------
info "Pulling a small uncensored local model (Qwen3.5-4B-Uncensored)..."
OLLAMA_HOST=127.0.0.1:11434 ollama pull hf.co/HauhauCS/Qwen3.5-4B-Uncensored-HauhauCS-Aggressive:latest || \
  warn "Model pull skipped/failed (network?). Run 'ollama pull <model>' later."

info "Done. Launch Harley with:  hp code"
info "Tailnet IP: $TAIL_IP  |  Dell brain reachable over Tailscale."
