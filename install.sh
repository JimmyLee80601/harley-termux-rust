#!/usr/bin/env bash
#
# harley-termux-rust :: one-click installer for the Harley Station on Termux (Android)
#
# One-click (on your phone, inside Termux):
#   curl -fsSL https://raw.githubusercontent.com/JimmyLee80601/harley-termux-rust/main/install.sh | bash
#
# What it does (idempotent - safe to re-run):
#   1. Refuses to run anywhere but Termux.
#   2. Installs runtime deps (android-tools, tailscale, ollama, termux-api, openssl ...).
#   3. Brings up Tailscale and prints your tailnet IP.
#   4. Installs the `harley-termux` binary - downloads the prebuilt release if present,
#      otherwise builds it from source via build-termux.sh.
#   5. Installs opencode (the agent runtime Harley lives in) unless already present.
#   6. Deploys the Harley opencode config + memory into ~/.config/opencode.
#   7. Creates the `hp` launcher (Harley Station menu).
#   8. Optionally pulls a small local uncensored model so Harley runs with no cloud.
#
set -euo pipefail

REPO="JimmyLee80601/harley-termux-rust"
HARLEY_HOME="$HOME/.config/opencode"
REPO_DIR="$HOME/harley-termux-rust"

info() { printf '\033[1;35m[Harley]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[!]\033[0m %s\n' "$*"; }
die()  { printf '\033[1;31m[ERROR]\033[0m %s\n' "$*" >&2; exit 1; }

[ "${TERMUX_VERSION:-}" ] || die "This installer is for Termux on Android. Run it inside the Termux app."

info "Updating Termux packages..."
pkg update -y && pkg upgrade -y

info "Installing runtime deps..."
pkg install -y \
  curl git wget openssl openssh termux-api \
  clang ncurses-utils zip unzip android-tools \
  rust pkg-config libcrypt libandroid-shmem cmake \
  tailscale ollama

# ---- Tailscale -------------------------------------------------------------
info "Bringing up Tailscale..."
tailscaled --tun=userspace-networking >/dev/null 2>&1 &
sleep 3
if ! tailscale status >/dev/null 2>&1; then
  warn "Tailscale not authenticated yet - run 'tailscale up' after install and re-run if you like."
fi
TAIL_IP=$(tailscale ip -4 2>/dev/null || echo "unknown")
info "Tailnet IP: $TAIL_IP"

# ---- harley-termux binary --------------------------------------------------
install_binary() {
  if command -v harley-termux >/dev/null 2>&1; then
    info "harley-termux already installed: $(harley-termux --version 2>/dev/null || echo ?)"
    return
  fi
  info "Fetching prebuilt harley-termux binary..."
  if curl -fsSL -o "$PREFIX/bin/harley-termux" \
       "https://github.com/$REPO/releases/latest/download/harley-termux" && \
     [ -s "$PREFIX/bin/harley-termux" ]; then
    chmod +x "$PREFIX/bin/harley-termux"
    info "Prebuilt binary installed to $PREFIX/bin/harley-termux"
  else
    warn "No prebuilt release yet - building from source (this can take a few minutes)..."
    [ -d "$REPO_DIR" ] || git clone "https://github.com/$REPO" "$REPO_DIR"
    pushd "$REPO_DIR" >/dev/null
    chmod +x build-termux.sh
    ./build-termux.sh
    popd >/dev/null
  fi
}
install_binary

# ---- opencode --------------------------------------------------------------
if command -v opencode >/dev/null 2>&1; then
  info "opencode already present: $(opencode --version 2>/dev/null || echo ?)"
else
  info "Installing opencode (agent runtime)..."
  curl -fsSL https://opencode.ai/install | sh
  export PATH="$HOME/.local/bin:$PATH"
fi

# ---- Harley config + memory ------------------------------------------------
info "Deploying Harley opencode config + memory..."
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

# ---- hp launcher -----------------------------------------------------------
info "Creating 'hp' launcher (Harley Station menu)..."
cat > "$PREFIX/bin/hp" <<'SH'
#!/usr/bin/env bash
echo "=== Harley Station (Termux) ==="
echo "1) opencode   2) tailscale status   3) ollama serve   4) harley-termux --help"
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
info "Rust tool:  harley-termux --help"
info "Tailnet IP: $TAIL_IP  |  Dell brain reachable over Tailscale."
