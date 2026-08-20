//! Harley Termux Toolkit — native Rust on Android
//! 
//! Build: cargo build --release --target aarch64-linux-android
//! Run on phone: ./harley-termux --help

use std::path::PathBuf;
use std::process::Command;
use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Parser)]
#[command(name = "harley-termux", version, about = "Harley's Termux-native toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// ADB/GSM device helpers
    Adb {
        #[command(subcommand)]
        cmd: AdbCmd,
    },
    /// HarleyLink relay client (connect to Dell workstation)
    Link {
        #[command(subcommand)]
        cmd: LinkCmd,
    },
    /// Memory sync with Dell workstation
    Memory {
        #[command(subcommand)]
        cmd: MemoryCmd,
    },
    /// Model management (download, list, serve)
    Model {
        #[command(subcommand)]
        cmd: ModelCmd,
    },
    /// System info & diagnostics
    Sys {
        #[command(subcommand)]
        cmd: SysCmd,
    },
}

#[derive(Subcommand)]
enum AdbCmd {
    /// List connected devices (USB + WiFi)
    Devices,
    /// Get device props (model, Android version, SOC, etc.)
    Props { serial: Option<String> },
    /// Pull /sdcard to local storage
    PullSdcard { dest: Option<PathBuf> },
    /// Screenshot to file
    Screenshot { output: PathBuf },
    /// Reboot (normal/recovery/bootloader/edl)
    Reboot { mode: Option<String> },
    /// Check FRP status
    FrpCheck { serial: Option<String> },
    /// Battery info
    Battery { serial: Option<String> },
    /// CPU info
    CpuInfo { serial: Option<String> },
}

#[derive(Subcommand)]
enum LinkCmd {
    /// Test connection to HarleyLink relay
    Ping { url: Option<String> },
    /// Get screen capture (JPEG)
    Screen { output: PathBuf, width: Option<u32>, quality: Option<u8> },
    /// Send input (mouse/keyboard)
    Input { json: String },
    /// Get/set draft text
    Draft { text: Option<String> },
    /// Proxy to LM Studio on Dell
    Models,
}

#[derive(Subcommand)]
enum MemoryCmd {
    /// Pull memory from Dell
    Pull { output: Option<PathBuf> },
    /// Push memory to Dell
    Push { file: PathBuf },
    /// Show local memory path
    Path,
}

#[derive(Subcommand)]
enum SysCmd {
    /// Show Termux environment info
    Info,
    /// Check Rust toolchain
    Rustc,
/// List installed packages
        Packages,
    }
}

/// Model management commands
#[derive(Subcommand)]
enum ModelCmd {
    /// Download a vision model from Hugging Face
    Download {
        /// Model name: minicpm-v, qwen2.5-vl, llava
        name: String,
        /// Quantization: q4_k_m, q3_k_m, q2_k (default: q3_k_m)
        #[arg(long, default_value = "q3_k_m")]
        quant: String,
        /// Output directory
        #[arg(long, default_value = "/sdcard/Download/models")]
        out_dir: PathBuf,
    },
    /// List downloaded models
    List {
        #[arg(long, default_value = "/sdcard/Download/models")]
        dir: PathBuf,
    },
    /// Start llama-server with a vision model
    Serve {
        /// Model file path
        model: PathBuf,
        /// MMProj file path
        mmproj: PathBuf,
        /// Port (default: 8080)
        #[arg(long, default_value = "8080")]
        port: u16,
        /// Context size (default: 4096)
        #[arg(long, default_value = "4096")]
        ctx: u32,
        /// GPU layers (default: 99 = all)
        #[arg(long, default_value = "99")]
        ngl: u32,
    },
}

#[derive(Serialize, Deserialize)]
struct DeviceProps {
    model: String,
    manufacturer: String,
    android_version: String,
    soc: String,
    abi: String,
    serial: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Adb { cmd } => handle_adb(cmd).await,
        Commands::Link { cmd } => handle_link(cmd).await,
        Commands::Memory { cmd } => handle_memory(cmd).await,
        Commands::Sys { cmd } => handle_sys(cmd).await,
        Commands::Model { cmd } => handle_model(cmd).await,
    }
}

async fn handle_adb(cmd: AdbCmd) -> Result<()> {
    let adb = which::which("adb").context("adb not in PATH — install via `pkg install android-tools`")?;
    
    match cmd {
        AdbCmd::Devices => {
            let out = Command::new(&adb).args(["devices", "-l"]).output()?;
            print!("{}", String::from_utf8_lossy(&out.stdout));
            if !out.status.success() {
                eprint!("{}", String::from_utf8_lossy(&out.stderr));
            }
        }
        AdbCmd::Props { serial } => {
            let mut args = vec!["shell", "getprop"];
            if let Some(s) = serial { args.insert(1, "-s"); args.insert(2, &s); }
            let out = Command::new(&adb).args(&args).output()?;
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        AdbCmd::PullSdcard { dest } => {
            let dest = dest.unwrap_or_else(|| PathBuf::from("/sdcard/Download/harley_backup"));
            let mut args = vec!["pull", "/sdcard/", dest.to_str().unwrap()];
            println!("Pulling /sdcard -> {}", dest.display());
            let status = Command::new(&adb).args(&args).status()?;
            if !status.success() { anyhow::bail!("adb pull failed"); }
        }
        AdbCmd::Screenshot { output } => {
            let tmp = "/sdcard/screen.png";
            Command::new(&adb).args(["shell", "screencap", "-p", tmp]).status()?;
            Command::new(&adb).args(["pull", tmp, output.to_str().unwrap()]).status()?;
            println!("Screenshot saved to {}", output.display());
        }
        AdbCmd::Reboot { mode } => {
            let mode = mode.unwrap_or_else(|| "normal".into());
            let arg = match mode.as_str() {
                "recovery" => "reboot recovery",
                "bootloader" | "fastboot" => "reboot bootloader",
                "edl" => "reboot edl",
                _ => "reboot",
            };
            let mut args: Vec<&str> = arg.split_whitespace().collect();
            args.insert(0, "shell");
            Command::new(&adb).args(&args).status()?;
            println!("Reboot command sent: {}", arg);
        }
        AdbCmd::FrpCheck { serial } => {
            let mut args = vec!["shell", "getprop", "ro.frp.pst"];
            if let Some(s) = serial { args.insert(1, "-s"); args.insert(2, &s); }
            let out = Command::new(&adb).args(&args).output()?;
            let frp = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("FRP Status: {}", if frp.is_empty() { "unknown/off" } else { &frp });
        }
        AdbCmd::Battery { serial } => {
            let mut args = vec!["shell", "dumpsys", "battery"];
            if let Some(s) = serial { args.insert(1, "-s"); args.insert(2, &s); }
            let out = Command::new(&adb).args(&args).output()?;
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        AdbCmd::CpuInfo { serial } => {
            let mut args = vec!["shell", "cat", "/proc/cpuinfo"];
            if let Some(s) = serial { args.insert(1, "-s"); args.insert(2, &s); }
            let out = Command::new(&adb).args(&args).output()?;
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
    }
    Ok(())
}

async fn handle_link(cmd: LinkCmd) -> Result<()> {
    let base_url = std::env::var("HARLEYLINK_URL")
        .unwrap_or_else(|_| "https://jimmysgsmworkstation.tail8deeb5.ts.net".into());
    let pin = std::env::var("HARLEYLINK_PIN").unwrap_or_else(|_| "930091".into());
    
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)  // self-signed or Let's Encrypt via funnel
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    
    let auth_header = format!("Bearer {}", pin);
    
    match cmd {
        LinkCmd::Ping { url } => {
            let url = url.unwrap_or_else(|| base_url.clone());
            println!("Pinging {}...", url);
            match client.get(&url).header("Authorization", &auth_header).send() {
                Ok(resp) => println!("✅ {} - {}", resp.status(), resp.status().canonical_reason().unwrap_or("")),
                Err(e) => println!("❌ Failed: {}", e),
            }
        }
        LinkCmd::Screen { output, width, quality } => {
            let mut url = format!("{}/screen", base_url);
            let mut params = vec![];
            if let Some(w) = width { params.push(format!("width={}", w)); }
            if let Some(q) = quality { params.push(format!("quality={}", q)); }
            if !params.is_empty() { url.push('?'); url.push_str(&params.join("&")); }
            
            println!("Fetching screen capture...");
            let resp = client.get(&url).header("Authorization", &auth_header).send()?;
            if !resp.status().is_success() {
                anyhow::bail!("Screen capture failed: {}", resp.status());
            }
            let bytes = resp.bytes()?;
            fs::write(&output, &bytes).await?;
            println!("✅ Saved {} bytes to {}", bytes.len(), output.display());
        }
        LinkCmd::Input { json } => {
            let url = format!("{}/input", base_url);
            let resp = client.post(&url)
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(json)
                .send()?;
            let text = resp.text()?;
            println!("{}", text);
        }
        LinkCmd::Draft { text } => {
            let url = format!("{}/draft", base_url);
            if let Some(t) = text {
                let resp = client.post(&url)
                    .header("Authorization", &auth_header)
                    .header("Content-Type", "application/json")
                    .body(format!(r#"{{"text":{:?}}}"#, t))
                    .send()?;
                println!("{}", resp.text()?);
            } else {
                let resp = client.get(&url).header("Authorization", &auth_header).send()?;
                let data: serde_json::Value = resp.json()?;
                println!("{}", data.get("text").and_then(|v| v.as_str()).unwrap_or(""));
            }
        }
        LinkCmd::Models => {
            let url = format!("{}/v1/models", base_url);
            let token = std::env::var("HARLEYLINK_TOKEN").unwrap_or_else(|_| "harley-connect-2026".into());
            let resp = client.get(&url).header("Authorization", format!("Bearer {}", token)).send()?;
            println!("{}", resp.text()?);
        }
    }
    Ok(())
}

async fn handle_memory(cmd: MemoryCmd) -> Result<()> {
    let base_url = std::env::var("HARLEYLINK_URL")
        .unwrap_or_else(|_| "https://jimmysgsmworkstation.tail8deeb5.ts.net".into());
    let pin = std::env::var("HARLEYLINK_PIN").unwrap_or_else(|_| "930091".into());
    
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    
    match cmd {
        MemoryCmd::Pull { output } => {
            let url = format!("{}/memory", base_url);
            println!("Fetching memory from Dell...");
            let resp = client.get(&url).header("Authorization", format!("Bearer {}", pin)).send()?;
            if !resp.status().is_success() {
                anyhow::bail!("Memory fetch failed: {}", resp.status());
            }
            let text = resp.text()?;
            let path = output.unwrap_or_else(|| PathBuf::from("/sdcard/Documents/Harley/harley-memory.md"));
            fs::write(&path, &text).await?;
            println!("✅ Memory saved to {} ({} bytes)", path.display(), text.len());
        }
        MemoryCmd::Push { file } => {
            // Would need a POST /memory endpoint on relay — not implemented yet
            println!("⚠️ Push not implemented on relay yet. Use git sync instead.");
        }
        MemoryCmd::Path => {
            let path = dirs::home_dir()
                .map(|h| h.join("Documents/Harley/harley-memory.md"))
                .unwrap_or_else(|| PathBuf::from("/sdcard/Documents/Harley/harley-memory.md"));
            println!("{}", path.display());
        }
    }
    Ok(())
}

async fn handle_sys(cmd: SysCmd) -> Result<()> {
    match cmd {
        SysCmd::Info => {
            println!("=== Termux Environment ===");
            println!("HOME: {}", std::env::var("HOME").unwrap_or_default());
            println!("PREFIX: {}", std::env::var("PREFIX").unwrap_or_default());
            println!("PATH: {}", std::env::var("PATH").unwrap_or_default());
            println!("SHELL: {}", std::env::var("SHELL").unwrap_or_default());
            println!("ANDROID_ROOT: {}", std::env::var("ANDROID_ROOT").unwrap_or_default());
            println!("TERMUX_VERSION: {}", std::env::var("TERMUX_VERSION").unwrap_or_default());
            
            // CPU info
            if let Ok(cpu) = fs::read_to_string("/proc/cpuinfo").await {
                let lines: Vec<&str> = cpu.lines().take(20).collect();
                println!("\n=== CPU (first 20 lines) ===");
                for l in lines { println!("{}", l); }
            }
            
            // Memory
            if let Ok(mem) = fs::read_to_string("/proc/meminfo").await {
                let lines: Vec<&str> = mem.lines().take(10).collect();
                println!("\n=== Memory (first 10 lines) ===");
                for l in lines { println!("{}", l); }
            }
        }
        SysCmd::Rustc => {
            let out = Command::new("rustc").args(["--version", "--print", "target-list"]).output()?;
            print!("{}", String::from_utf8_lossy(&out.stdout));
            if !out.status.success() {
                eprint!("{}", String::from_utf8_lossy(&out.stderr));
            }
        }
        SysCmd::Packages => {
            let out = Command::new("pkg").args(["list-installed"]).output()?;
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
    }
    Ok(())
}

async fn handle_model(cmd: ModelCmd) -> Result<()> {
    use std::io::{BufRead, BufReader};
    
    match cmd {
        ModelCmd::Download { name, quant, out_dir } => {
            fs::create_dir_all(&out_dir).await?;
            println!("📥 Downloading {} ({}) to {}", name, quant, out_dir.display());
            
            let (repo, model_file, mmproj_file) = match name.as_str() {
                "minicpm-v" | "minicpm" => {
                    let model = format!("ggml-model-{}.gguf", quant.to_uppercase().replace("_", "-"));
                    ("openbmb/MiniCPM-V-2_6-gguf".to_string(), model, "mmproj-model-f16.gguf".to_string())
                }
                "qwen2.5-vl" | "qwen" => {
                    let model = format!("qwen2.5-vl-3b-instruct-{}.gguf", quant);
                    ("Qwen/Qwen2.5-VL-3B-Instruct-GGUF".to_string(), model, "mmproj-qwen2.5-vl-3b-f16.gguf".to_string())
                }
                "llava" => {
                    let model = format!("llava-phi-3-mini-{}.gguf", quant);
                    ("llava-hf/llava-phi-3-mini-GGUF".to_string(), model, "mmproj-llava-phi-3-mini-f16.gguf".to_string())
                }
                _ => anyhow::bail!("Unknown model: {}. Use minicpm-v, qwen2.5-vl, or llava", name),
            };
            
            let python_script = format!(r#"
from huggingface_hub import hf_hub_download
import os
os.makedirs(r"{}", exist_ok=True)
for f in [r"{}", r"{}"]:
    try:
        print(f"Downloading {{f}}...")
        path = hf_hub_download(repo_id=r"{}", filename=f, local_dir=r"{}", local_dir_use_symlinks=False)
        print(f"  -> {{path}} ({{os.path.getsize(path)/1024/1024:.1f}} MB)")
    except Exception as e:
        print(f"  ERROR: {{e}}")
"#, out_dir.display(), model_file, mmproj_file, repo, out_dir.display());
            
            let status = Command::new("python")
                .args(["-c", &python_script])
                .status()?;
            
            if !status.success() {
                anyhow::bail!("Model download failed");
            }
            println!("✅ Model downloaded to {}", out_dir.display());
        }
        ModelCmd::List { dir } => {
            if !dir.exists() {
                println!("Directory not found: {}", dir.display());
                return Ok(());
            }
            let mut entries = fs::read_dir(&dir).await?;
            println!("Models in {}:", dir.display());
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "gguf") {
                    let size = entry.metadata().await?.len();
                    println!("  {} ({:.1f} MB)", path.file_name().unwrap().to_string_lossy(), size as f64 / 1024.0 / 1024.0);
                }
            }
        }
        ModelCmd::Serve { model, mmproj, port, ctx, ngl } => {
            if !model.exists() { anyhow::bail!("Model not found: {}", model.display()); }
            if !mmproj.exists() { anyhow::bail!("MMProj not found: {}", mmproj.display()); }
            
            println!("🚀 Starting llama-server on port {}", port);
            println!("   Model: {}", model.display());
            println!("   MMProj: {}", mmproj.display());
            println!("   Context: {}, GPU layers: {}", ctx, ngl);
            
            let llama_server = which::which("llama-server")
                .or_else(|_| which::which("llama-server.exe"))
                .context("llama-server not in PATH. Install via `pkg install llama.cpp` or build from source")?;
            
            let mut args = vec![
                "-m", model.to_str().unwrap(),
                "--mmproj", mmproj.to_str().unwrap(),
                "--host", "0.0.0.0",
                "--port", &port.to_string(),
                "-c", &ctx.to_string(),
                "-ngl", &ngl.to_string(),
            ];
            
            println!("Running: {} {}", llama_server.display(), args.join(" "));
            
            let mut child = Command::new(llama_server)
                .args(&args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()?;
            
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    println!("{}", line?);
                }
            }
            if let Some(stderr) = child.stderr.take() {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    eprintln!("{}", line?);
                }
            }
            
            child.wait()?;
        }
    }
    Ok(())
}