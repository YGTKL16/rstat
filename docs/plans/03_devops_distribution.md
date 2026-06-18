# rstat — DevOps & Distribution Planı (03)

> **Oluşturulma:** 2026-06-18  
> **Yazar:** DevOps & Dağıtım Subagent  
> **Kapsam:** CI/CD, cross-platform release, Ed25519 offline lisans, Lemon Squeezy self-serve

---

## İçindekiler

1. [GitHub Actions CI Pipeline](#1-github-actions-ci-pipeline)
2. [Release Pipeline (cargo-dist)](#2-release-pipeline-cargo-dist)
3. [Dağıtım Kanalları](#3-dağıtım-kanalları)
4. [Offline Ed25519 Lisans Sistemi](#4-offline-ed25519-lisans-sistemi)
5. [Lemon Squeezy Entegrasyonu](#5-lemon-squeezy-entegrasyonu)
6. [Bakım Sıfır Modu](#6-bakım-sıfır-modu)
7. [Somut İmplementasyon Sıralaması](#7-somut-implementasyon-sıralaması)

---

## 1. GitHub Actions CI Pipeline

> **5 job paralel çalışır. PR + push to main trigger. Ortalama ~4-6 dakika.**

### `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # ─── Job 1: Format kontrolü ───────────────────────────────────────────────
  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all --check

  # ─── Job 2: Clippy lint ───────────────────────────────────────────────────
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2          # serde/statrs büyük — cache kritik
        with:
          key: clippy-${{ hashFiles('**/Cargo.lock') }}
      - run: cargo clippy --all-targets --all-features -- -D warnings

  # ─── Job 3: Test (Linux) ─────────────────────────────────────────────────
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          key: test-${{ hashFiles('**/Cargo.lock') }}
      - run: cargo test --all --all-features -- --test-threads=4

  # ─── Job 4: Cross-validation (scipy) ─────────────────────────────────────
  crossvalidate-python:
    name: Cross-validate (Python/scipy)
    runs-on: ubuntu-latest
    needs: test                               # testler geçmeden çalışma
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
          cache: pip
      - name: Install scipy
        run: pip install scipy numpy
      - name: Build binary
        run: cargo build --bin rstat-cli --release
      - name: Run cross-validation
        run: python3 scripts/crossvalidate_ttest.py --binary target/release/rstat-cli
      - name: Upload cross-validation report
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: crossval-report
          path: crossval_results.json

  # ─── Job 5: Cross-validation (R) ─────────────────────────────────────────
  crossvalidate-r:
    name: Cross-validate (R)
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: r-lib/actions/setup-r@v2
        with:
          r-version: "4.4"
      - name: Install R packages
        run: Rscript -e "install.packages(c('jsonlite'), repos='https://cloud.r-project.org')"
      - name: Build binary
        run: cargo build --bin rstat-cli --release
      - name: Run R cross-validation
        run: Rscript scripts/crossvalidate.R --binary target/release/rstat-cli
```

### Cache Stratejisi

`statrs` (~500KB) ve `serde` zinciri büyük bağımlılıklardır. `Swatinem/rust-cache` şu stratejiye göre cache'ler:

| Cache Key | Ne Zaman Geçersizleşir |
|-----------|------------------------|
| `Cargo.lock` hash | Herhangi bir bağımlılık değişince |
| `Cargo.toml` hash | Workspace üyeleri değişince |
| Job adı (`clippy-`, `test-`) | Job-bazlı ayrım |

Cache miss → ilk build ~3-4 dk; cache hit → ~30-45 sn.

---

## 2. Release Pipeline (cargo-dist)

> **`v*` tag push'u tüm 5 platformu otomatik build eder, SHA256 checksum üretir, GitHub Release'e upload eder.**

### Adım 1: cargo-dist Kurulum

```bash
cargo install cargo-dist
cargo dist init
# → Hedef platformlar seçilir
# → CI provider: github-actions
# → installer: shell + powershell
```

### Kök `Cargo.toml` Metadata

```toml
[workspace.metadata.dist]
cargo-dist-version = "0.28.0"
ci = "github"
targets = [
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
installers = ["shell", "powershell"]
tap = "yourusername/homebrew-rstat"
publish-jobs = ["homebrew"]
pr-run-mode = "plan"

[profile.dist]
inherits = "release"
lto = "thin"
```

### `.github/workflows/release.yml`

```yaml
name: Release

permissions:
  contents: write

on:
  push:
    tags:
      - "v[0-9]+.[0-9]+.[0-9]+"

jobs:
  plan:
    name: Plan Release
    runs-on: ubuntu-latest
    outputs:
      val: ${{ steps.plan.outputs.manifest }}
    env:
      GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install cargo-dist
        run: curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/latest/download/cargo-dist-installer.sh | sh
      - id: plan
        run: cargo dist plan --tag=${{ github.ref_name }} --output-format=json > dist-manifest.json && echo "manifest=$(cat dist-manifest.json)" >> "$GITHUB_OUTPUT"
      - uses: actions/upload-artifact@v4
        with:
          name: artifacts-dist-manifest
          path: dist-manifest.json

  build-local-artifacts:
    name: Build ${{ matrix.display_name }}
    needs: plan
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-22.04
            dist-args: --artifacts=local --target=x86_64-unknown-linux-musl
            display_name: Linux x86_64 (musl)
          - os: ubuntu-22.04
            dist-args: --artifacts=local --target=aarch64-unknown-linux-musl
            display_name: Linux ARM64 (musl)
          - os: macos-13
            dist-args: --artifacts=local --target=x86_64-apple-darwin
            display_name: macOS Intel
          - os: macos-14
            dist-args: --artifacts=local --target=aarch64-apple-darwin
            display_name: macOS Apple Silicon
          - os: windows-2022
            dist-args: --artifacts=local --target=x86_64-pc-windows-msvc
            display_name: Windows x86_64
    runs-on: ${{ matrix.os }}
    env:
      GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install cargo-dist
        shell: bash
        run: curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/latest/download/cargo-dist-installer.sh | sh
      - name: Install musl-tools (Linux musl için)
        if: runner.os == 'Linux'
        run: sudo apt-get install -y musl-tools
      - name: Build artifacts
        run: cargo dist build ${{ matrix.dist-args }} --output-format=json
      - uses: actions/upload-artifact@v4
        with:
          name: artifacts-${{ matrix.os }}
          path: target/distrib/

  publish-github-release:
    name: Publish GitHub Release
    needs: [plan, build-local-artifacts]
    runs-on: ubuntu-latest
    env:
      GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    steps:
      - uses: actions/checkout@v4
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          pattern: artifacts-*
          path: artifacts
          merge-multiple: true
      - name: Generate SHA256 checksums
        run: |
          cd artifacts
          sha256sum rstat-cli-* > SHA256SUMS.txt
          cat SHA256SUMS.txt
      - name: Install cargo-dist
        run: curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/latest/download/cargo-dist-installer.sh | sh
      - name: Publish Release
        run: cargo dist publish --tag=${{ github.ref_name }} --artifacts=artifacts/
```

### Artifact Adlandırma

```
rstat-cli-v1.0.0-x86_64-unknown-linux-musl.tar.gz
rstat-cli-v1.0.0-aarch64-unknown-linux-musl.tar.gz
rstat-cli-v1.0.0-x86_64-apple-darwin.tar.gz
rstat-cli-v1.0.0-aarch64-apple-darwin.tar.gz
rstat-cli-v1.0.0-x86_64-pc-windows-msvc.zip
SHA256SUMS.txt
install.sh
install.ps1
```

---

## 3. Dağıtım Kanalları

### 3.1 `rstat-cli/Cargo.toml` Publish Metadata

```toml
[package]
name = "rstat-cli"
version = "0.1.0"
description = "Cross-platform statistics CLI — scipy.stats for the terminal"
repository = "https://github.com/yourusername/rstat"
homepage = "https://rstat.dev"
license = "BUSL-1.1"
keywords = ["statistics", "cli", "ttest", "anova", "spc"]
categories = ["command-line-utilities", "science"]
```

```bash
# Kullanıcı kurulumu
cargo install rstat-cli
cargo binstall rstat-cli   # daha hızlı, prebuilt binary
```

### 3.2 `cargo binstall` Konfigürasyonu

```toml
# rstat-cli/Cargo.toml'a ekle
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/rstat-cli-v{ version }-{ target }.tar.gz"
bin-dir = "rstat-cli-v{ version }-{ target }/rstat-cli{ binary-ext }"
pkg-fmt = "tgz"

[package.metadata.binstall.overrides]
"x86_64-pc-windows-msvc" = { pkg-fmt = "zip", bin-dir = "rstat-cli.exe" }
```

### 3.3 Homebrew Formula Şablonu

`homebrew-rstat/Formula/rstat-cli.rb`:

```ruby
class RstatCli < Formula
  desc "Cross-platform statistics CLI — scipy.stats for the terminal"
  homepage "https://github.com/yourusername/rstat"
  version "0.1.0"
  license "BUSL-1.1"

  on_macos do
    on_arm do
      url "https://github.com/yourusername/rstat/releases/download/v#{version}/rstat-cli-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "BURAYA_SHA256_KOYULACAK"
    end
    on_intel do
      url "https://github.com/yourusername/rstat/releases/download/v#{version}/rstat-cli-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "BURAYA_SHA256_KOYULACAK"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/yourusername/rstat/releases/download/v#{version}/rstat-cli-v#{version}-aarch64-unknown-linux-musl.tar.gz"
      sha256 "BURAYA_SHA256_KOYULACAK"
    end
    on_intel do
      url "https://github.com/yourusername/rstat/releases/download/v#{version}/rstat-cli-v#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "BURAYA_SHA256_KOYULACAK"
    end
  end

  def install
    bin.install "rstat-cli"
  end

  test do
    assert_match "rstat-cli", shell_output("#{bin}/rstat-cli --version")
  end
end
```

**cargo-dist `tap` ayarlandığında Formula'yı release sırasında otomatik günceller.**

```bash
# Kullanıcı kurulumu
brew tap yourusername/rstat
brew install rstat-cli
```

### 3.4 `install.sh` (Linux/macOS)

```bash
#!/bin/sh
set -eu

REPO="yourusername/rstat"
BINARY="rstat-cli"

case "$(uname -sm)" in
  "Darwin arm64")  TARGET="aarch64-apple-darwin" ;;
  "Darwin x86_64") TARGET="x86_64-apple-darwin" ;;
  "Linux aarch64") TARGET="aarch64-unknown-linux-musl" ;;
  "Linux x86_64")  TARGET="x86_64-unknown-linux-musl" ;;
  *) echo "Desteklenmeyen platform: $(uname -sm)"; exit 1 ;;
esac

VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"v\([^"]*\)".*/\1/')
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY}-v${VERSION}-${TARGET}.tar.gz"

echo "İndiriliyor: ${BINARY} v${VERSION} (${TARGET})"
curl -sSfL "${URL}" | tar -xz -C /tmp
install -m755 /tmp/${BINARY} /usr/local/bin/${BINARY}
echo "Kurulum tamamlandı!"
```

### 3.5 `install.ps1` (Windows)

```powershell
$Repo = "yourusername/rstat"
$Binary = "rstat-cli"
$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Version = $Release.tag_name.TrimStart('v')
$Url = "https://github.com/$Repo/releases/download/v$Version/$Binary-v$Version-x86_64-pc-windows-msvc.zip"

$TempDir = [System.IO.Path]::GetTempPath()
$ZipPath = Join-Path $TempDir "$Binary.zip"
Invoke-WebRequest -Uri $Url -OutFile $ZipPath
Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

$InstallDir = "$env:LOCALAPPDATA\Programs\rstat"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item "$TempDir\$Binary.exe" "$InstallDir\$Binary.exe" -Force

$CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$CurrentPath;$InstallDir", "User")
}
Write-Host "Kurulum tamamlandı: $InstallDir"
```

---

## 4. Offline Ed25519 Lisans Sistemi

> **Felsefe: Bulut sunucusu yok. Binary offline çalışır. Lisans dosyası local. Public key binary'ye gömülü.**

### 4.1 Mimari

```
KURUCU MAKİNESİ
  rstat-keygen generate-keypair
    ├── Private key → 1Password'da sakla
    └── Public key → verify.rs EMBEDDED_PUBLIC_KEY_HEX const'una gömülür

LEMON SQUEEZY WEBHOOK → Cloudflare Worker (100 satır JS)
  order_created event → private key ile lisans imzala → email gönder

MÜŞTERİ MAKİNESİ
  ~/.config/rstat/license.json
  rstat spc ... → public key ile doğrula → çalışır ✓
```

### 4.2 Kullanılacak Crate'ler

| Crate | Versiyon | Amaç |
|-------|----------|------|
| `ed25519-dalek` | 2.x | Ed25519 imzalama ve doğrulama |
| `hex` | 0.4 | Public/private key hex encode/decode |
| `base64` | 0.22 | Signature base64 encoding |
| `dirs` | 5.x | `~/.config/rstat/` yolunu bul |
| `chrono` | 0.4 | Expiry tarihi parse/kontrol |

### 4.3 Yeni Crate Yapısı

```
crates/
  rstat-license/          # doğrulama kütüphanesi (binary'ye dahil)
    Cargo.toml
    src/
      lib.rs
      license.rs          # LicenseFile struct
      verify.rs           # LicenseVerifier + EMBEDDED_PUBLIC_KEY_HEX
      feature.rs          # load_license(), check_feature(), require_feature! macro
      error.rs            # LicenseError enum

  rstat-keygen/           # kurucu CLI (publish = false, dağıtılmaz)
    Cargo.toml
    src/
      main.rs             # generate-keypair, sign subcommandları
```

### 4.4 `rstat-license/Cargo.toml`

```toml
[package]
name = "rstat-license"
version = "0.1.0"
edition = "2024"

[dependencies]
ed25519-dalek = { version = "2", features = ["pkcs8"] }
hex = "0.4"
base64 = "0.22"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
dirs = "5"
chrono = { version = "0.4", features = ["serde"] }
```

### 4.5 Lisans Dosyası Formatı

**`~/.config/rstat/license.json`**:

```json
{
  "version": 1,
  "licensee": {
    "name": "Ahmet Yılmaz",
    "email": "ahmet@sirket.com",
    "order_id": "LS-12345"
  },
  "product": "rstat",
  "tier": "pro",
  "features": ["spc", "capability"],
  "issued_at": "2026-06-18T12:00:00Z",
  "expires_at": null,
  "signature": "BASE64_ED25519_SIGNATURE"
}
```

**İmzalanan veri** = şu alanların deterministik JSON'u (sıralı key, boşluksuz):
```json
{"email":"...","features":[...],"issued_at":"...","tier":"...","version":1}
```

### 4.6 `verify.rs` Implementasyonu

```rust
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use crate::license::LicenseFile;
use crate::error::LicenseError;

// cargo-dist build sırasında değiştirilir — `rstat-keygen generate-keypair` çıktısı
pub const EMBEDDED_PUBLIC_KEY_HEX: &str =
    "BURAYA_GERCEK_PUBLIC_KEY_HEX_GELECEK_32_BYTE_64_HEX_CHAR";

pub struct LicenseVerifier {
    verifying_key: VerifyingKey,
}

impl LicenseVerifier {
    pub fn from_embedded() -> Result<Self, LicenseError> {
        let bytes = hex::decode(EMBEDDED_PUBLIC_KEY_HEX)
            .map_err(|_| LicenseError::InvalidPublicKey)?;
        let key_bytes: [u8; 32] = bytes.try_into()
            .map_err(|_| LicenseError::InvalidPublicKey)?;
        Ok(Self {
            verifying_key: VerifyingKey::from_bytes(&key_bytes)
                .map_err(|_| LicenseError::InvalidPublicKey)?,
        })
    }

    pub fn verify(&self, license: &LicenseFile) -> Result<(), LicenseError> {
        // Expiry kontrolü
        if let Some(expires_at) = &license.expires_at {
            let expiry = chrono::DateTime::parse_from_rfc3339(expires_at)
                .map_err(|_| LicenseError::InvalidDate)?;
            if chrono::Utc::now() > expiry {
                return Err(LicenseError::Expired);
            }
        }
        // İmza doğrulama
        let sig_bytes = base64::decode(&license.signature)
            .map_err(|_| LicenseError::InvalidSignature)?;
        let signature = Signature::from_slice(&sig_bytes)
            .map_err(|_| LicenseError::InvalidSignature)?;
        self.verifying_key
            .verify(&license.signing_payload(), &signature)
            .map_err(|_| LicenseError::InvalidSignature)
    }
}
```

### 4.7 Feature Gate

```rust
// feature.rs
use std::sync::OnceLock;

static LICENSE: OnceLock<Option<crate::license::LicenseFile>> = OnceLock::new();

pub fn load_license() {
    LICENSE.get_or_init(|| try_load());
}

fn try_load() -> Option<crate::license::LicenseFile> {
    let path = std::env::var("RSTAT_LICENSE_FILE").ok()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            dirs::config_dir().unwrap().join("rstat").join("license.json")
        });
    let content = std::fs::read_to_string(path).ok()?;
    let lic: crate::license::LicenseFile = serde_json::from_str(&content).ok()?;
    crate::verify::LicenseVerifier::from_embedded().ok()?
        .verify(&lic).ok()?;
    Some(lic)
}

pub fn check_feature(feature: &str) -> Result<(), String> {
    match LICENSE.get().and_then(|l| l.as_ref()) {
        Some(l) if l.has_feature(feature) => Ok(()),
        Some(_) => Err(format!("'{}' bu lisansta yok", feature)),
        None => Err("Lisans bulunamadı. ~/.config/rstat/license.json".to_string()),
    }
}

// CLI'da kullanım:
// require_feature!("spc");
#[macro_export]
macro_rules! require_feature {
    ($feature:expr) => {
        if let Err(e) = rstat_license::feature::check_feature($feature) {
            eprintln!("⛔ {}", e);
            eprintln!("   Satın al: https://rstat.dev/pricing");
            std::process::exit(1);
        }
    };
}
```

**`main.rs`'de başlatma:**

```rust
fn main() {
    rstat_license::feature::load_license(); // hata olsa bile devam et
    let cli = Cli::parse();
    // ...
}

// commands/spc.rs:
pub fn run(args: &SpcArgs) -> anyhow::Result<()> {
    require_feature!("spc");
    // ... hesaplama
}
```

### 4.8 `rstat-keygen` — Lisans Üretim CLI

```rust
// crates/rstat-keygen/src/main.rs
use clap::{Parser, Subcommand};
use ed25519_dalek::{SigningKey, Signer};
use rand::rngs::OsRng;

#[derive(Parser)]
#[command(name = "rstat-keygen")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ed25519 keypair üret (bir kez çalıştır)
    GenerateKeypair,
    /// Lisans dosyası oluştur ve imzala
    Sign {
        #[arg(long)] name: String,
        #[arg(long)] email: String,
        #[arg(long)] order_id: String,
        #[arg(long, default_value = "pro")] tier: String,
        #[arg(long, value_delimiter = ',')] features: Vec<String>,
        #[arg(long)] private_key: String, // hex
        #[arg(long, default_value = "rstat_license.json")] output: String,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Commands::GenerateKeypair => {
            let signing_key = SigningKey::generate(&mut OsRng);
            let verifying_key = signing_key.verifying_key();
            println!("PRIVATE KEY (gizli):\n{}", hex::encode(signing_key.to_bytes()));
            println!("\nPUBLIC KEY (verify.rs'e gömülecek):\n{}", hex::encode(verifying_key.to_bytes()));
        }
        Commands::Sign { name, email, order_id, tier, features, private_key, output } => {
            let key_bytes = hex::decode(&private_key)?;
            let key_arr: [u8; 32] = key_bytes.try_into().unwrap();
            let signing_key = SigningKey::from_bytes(&key_arr);

            let issued_at = chrono::Utc::now().to_rfc3339();
            let mut license = rstat_license::license::LicenseFile {
                version: 1,
                licensee: rstat_license::license::Licensee { name, email, order_id },
                product: "rstat".to_string(),
                tier, features, issued_at,
                expires_at: None,
                signature: String::new(),
            };
            let sig: ed25519_dalek::Signature = signing_key.sign(&license.signing_payload());
            license.signature = base64::encode(sig.to_bytes());
            std::fs::write(&output, serde_json::to_string_pretty(&license)?)?;
            println!("✓ Lisans oluşturuldu: {output}");
        }
    }
    Ok(())
}
```

**Kullanım:**

```bash
# 1. Bir kez: keypair üret
cargo run --bin rstat-keygen -- generate-keypair
# → PRIVATE_KEY_HEX ve PUBLIC_KEY_HEX çıktısı

# 2. PUBLIC_KEY_HEX'i verify.rs EMBEDDED_PUBLIC_KEY_HEX'e koy, commit et

# 3. Acil/manuel lisans üretimi:
cargo run --bin rstat-keygen -- sign \
  --name "Ahmet Yılmaz" \
  --email "ahmet@sirket.com" \
  --order-id "LS-12345" \
  --features spc,capability \
  --private-key "HEX_PRIVATE_KEY"
```

---

## 5. Lemon Squeezy Entegrasyonu

### 5.1 Ürün Kurulumu

**Dashboard → Products → Add Product:**

| Alan | Değer |
|------|-------|
| Name | rstat Pro |
| Type | Software |
| Price | $49 (one-time) |
| Variant 2 | rstat Pro Annual — $29/yıl |
| Variant 3 | rstat Team 5-seat — $149 |

**Webhook kurulumu:**
- Settings → Webhooks → Add Webhook
- URL: `https://your-worker.workers.dev/webhook`
- Events: `order_created`
- Secret: `openssl rand -hex 32` çıktısı (env'de WEBHOOK_SECRET)

### 5.2 Cloudflare Worker Webhook Handler (~100 satır JS)

```javascript
// src/worker.js
// wrangler secret put PRIVATE_KEY_HEX
// wrangler secret put WEBHOOK_SECRET
// wrangler secret put RESEND_API_KEY

export default {
  async fetch(request, env) {
    if (request.method !== 'POST') return new Response('Not Found', { status: 404 });

    const body = await request.text();

    // Webhook imza doğrulama (HMAC-SHA256)
    const sig = request.headers.get('X-Signature');
    const expectedSig = await hmacSha256(env.WEBHOOK_SECRET, body);
    if (sig !== expectedSig) return new Response('Unauthorized', { status: 401 });

    const event = JSON.parse(body);
    if (event.meta.event_name !== 'order_created') return new Response('OK');

    const order = event.data.attributes;
    const customerEmail = order.user_email;
    const customerName = order.user_name || 'Customer';
    const orderId = `LS-${event.data.id}`;

    // Lisans JSON üret (Node.js crypto ile Ed25519 imza)
    const licenseJson = await generateLicense({
      name: customerName,
      email: customerEmail,
      orderId,
      privateKeyHex: env.PRIVATE_KEY_HEX,
    });

    // Email gönder
    await fetch('https://api.resend.com/emails', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${env.RESEND_API_KEY}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        from: 'rstat <noreply@rstat.dev>',
        to: [customerEmail],
        subject: 'rstat Pro — Lisans Dosyanız',
        text: buildEmailText(customerName, licenseJson),
        attachments: [{
          filename: 'rstat_license.json',
          content: btoa(licenseJson),
        }],
      }),
    });

    return new Response('OK');
  }
};

async function hmacSha256(secret, data) {
  const key = await crypto.subtle.importKey(
    'raw', new TextEncoder().encode(secret),
    { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']
  );
  const sig = await crypto.subtle.sign('HMAC', key, new TextEncoder().encode(data));
  return Array.from(new Uint8Array(sig)).map(b => b.toString(16).padStart(2, '0')).join('');
}

function buildEmailText(name, licenseJson) {
  return `Merhaba ${name},

rstat Pro lisansınız hazır! Kurulum:

  Linux/macOS:
    mkdir -p ~/.config/rstat
    # Ekteki dosyayı şuraya taşı:
    mv rstat_license.json ~/.config/rstat/license.json
    rstat spc --chart xbar-r verileriniz.csv   # test edin

  Windows:
    # %APPDATA%\\rstat\\license.json konumuna koy

Sorular: support@rstat.dev
`;
}
```

> **Ed25519 imza Cloudflare Worker'da:** Web Crypto API Ed25519'u destekler (`SubtleCrypto.sign` ile `Ed25519` algorithm). Alternatif: webhook gelince sadece `rstat-keygen sign` komutunu çalıştıran basit bir Rust binary sunucu (Railway/Fly.io'da ücretsiz tier).

### 5.3 Müşteri Self-Serve Akışı

```
1. rstat.dev/pricing → "Buy Pro" ($49)           [30 sn]
         │
         ▼
2. Lemon Squeezy checkout (kart bilgisi)          [1 dk]
   (Lemon Squeezy VAT/vergi halleder)
         │
         ▼
3. Ödeme onayı → webhook tetiklenir               [anlık]
         │
         ▼
4. Müşteri ~30 sn içinde email alır               [otomatik]
   Konu: "rstat Pro — Lisans Dosyanız"
   Ek: rstat_license.json
         │
         ▼
5. Müşteri 3 komut çalıştırır:                   [1 dk]
   mkdir -p ~/.config/rstat
   mv ~/Downloads/rstat_license.json ~/.config/rstat/license.json
   rstat spc --chart xbar-r data.csv   # ✓ çalışır

Toplam: satın alma → çalışır = ~3 dakika
Müşteri destek ihtiyacı: SIFIR (self-serve)
```

**Karmaşıklık:** ⭐⭐☆☆☆

- Lemon Squeezy kurulumu: 30-60 dk
- Cloudflare Worker + wrangler deploy: 2-4 saat
- Email template: 1 saat
- Ed25519 signing (Worker'da): 1-2 saat
- **Toplam first-time setup: ~1 iş günü**

---

## 6. Bakım Sıfır Modu

### 6.1 Askeri Görev Sırasında Sistem Durumu

| Bileşen | Durum | Açıklama |
|---------|-------|----------|
| CI pipeline | ✅ Çalışır | PR yoksa tetiklenmez bile |
| Release pipeline | ✅ Çalışır | Tag push tam otomasyon |
| Lemon Squeezy satışlar | ✅ Çalışır | Müşteri kendi alır |
| Webhook → email | ✅ Çalışır | Cloudflare Workers 7/24 |
| Lisans doğrulama | ✅ Çalışır | Tamamen offline, müşteri makinesinde |
| Homebrew formula güncelleme | ✅ Çalışır | cargo-dist otomatik PR açar |
| Dependabot patch PR'ları | ✅ Çalışır | Auto-merge ile |
| GitHub Issues yanıtlama | ❌ Durur | FAQ ile minimize et |
| Minor bağımlılık güncellemeleri | ⚠️ Bekler | Dönünce merge edilir |

### 6.2 Dependabot Konfigürasyonu

**`.github/dependabot.yml`:**

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: weekly
      day: monday
      time: "09:00"
      timezone: "Europe/Istanbul"
    open-pull-requests-limit: 5
    labels: [dependencies, rust]
    groups:
      patch-updates:
        update-types: [patch]

  - package-ecosystem: github-actions
    directory: "/"
    schedule:
      interval: monthly
    labels: [dependencies, github-actions]
```

**`.github/workflows/auto-merge.yml`** (patch PR'larını otomatik merge et):

```yaml
name: Auto-merge Dependabot PRs

on:
  pull_request:
    types: [opened, synchronize]

permissions:
  pull-requests: write
  contents: write

jobs:
  auto-merge:
    runs-on: ubuntu-latest
    if: github.actor == 'dependabot[bot]'
    steps:
      - uses: actions/checkout@v4
      - name: Fetch Dependabot metadata
        id: metadata
        uses: dependabot/fetch-metadata@v2
      - name: Auto-merge patch updates
        if: steps.metadata.outputs.update-type == 'version-update:semver-patch'
        run: gh pr merge --auto --squash "$PR_URL"
        env:
          PR_URL: ${{ github.event.pull_request.html_url }}
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 6.3 Self-Serve FAQ

`docs/FAQ.md` → rstat.dev/docs:

```markdown
## Sık Sorulan Sorular

**S: Lisans dosyamı nereye koyacağım?**
Linux/macOS: `~/.config/rstat/license.json`
Windows: `%APPDATA%\rstat\license.json`

**S: "Lisans bulunamadı" hatası:**
1. Dosya yolunu kontrol et: `ls ~/.config/rstat/`
2. Spam klasörünü kontrol et
3. RSTAT_LICENSE_FILE=/yol/license.json rstat spc ...

**S: Lisansımı yeni makineye taşıyabilir miyim?**
Evet — dosyayı kopyala ve aynı konuma koy.

**S: Kaç makinede kullanabilirim?**
Pro: sınırsız kişisel makine. Team: 5 farklı kullanıcı.

**S: Refund istiyorum.**
30 gün içinde koşulsuz: support@rstat.dev
```

---

## 7. Somut İmplementasyon Sıralaması

| # | Görev | Süre | Faz | Öncelik |
|---|-------|------|-----|---------|
| 7.1 | `.github/workflows/ci.yml` oluştur | 2-3 saat | Şimdi | 🔴 |
| 7.2 | `rust-toolchain.toml` ekle | 30 dk | Şimdi | 🔴 |
| 7.3 | `rstat-license` crate iskelet + testler | 4-6 saat | Faz 6 | 🟡 |
| 7.4 | `rstat-keygen` CLI implementasyonu | 3-4 saat | Faz 6 | 🟡 |
| 7.5 | Keypair üret, public key'i binary'ye göm | 1 saat | Faz 6 | 🟡 |
| 7.6 | Feature gate macro'larını CLI'a entegre et | 2 saat | Faz 6 | 🟡 |
| 7.7 | Workspace `Cargo.toml`'a dist metadata ekle | 1 saat | Faz 7 | 🟠 |
| 7.8 | `cargo dist init` → release workflow commit | 2 saat | Faz 7 | 🟠 |
| 7.9 | `homebrew-rstat` repo oluştur | 30 dk | Faz 7 | 🟠 |
| 7.10 | Lemon Squeezy ürün + webhook kurulumu | 1 saat | Faz 7 | 🟠 |
| 7.11 | Cloudflare Worker webhook + email | 4-6 saat | Faz 7 | 🟠 |
| 7.12 | Dependabot + auto-merge | 1 saat | İstediğinde | 🟢 |
| 7.13 | `install.sh` / `install.ps1` son test | 1 saat | Release öncesi | 🟢 |
| 7.14 | FAQ dokümanı + landing page | 4-8 saat | Lansman öncesi | 🟢 |
| 7.15 | İlk `v1.0.0` tag push → tam release testi | 2 saat | Lansman | 🟢 |

### Kritik Yol

```
7.1 ci.yml
    ↓
    ├── 7.3-7.6 rstat-license crate (Faz 6)
    └── 7.7-7.8 cargo-dist (Faz 7)
                    ↓
              7.9 homebrew-rstat repo
              7.10 Lemon Squeezy kurulumu
                    ↓
              7.11 Cloudflare Worker webhook
                    ↓
              v1.0.0 tag push → GitHub Release ✅ → İlk satış 💰
```

### Bu Haftaki İlk Adım (Hemen Yapılabilir, ~3 Saat)

```bash
# 1. Dizin oluştur
mkdir -p .github/workflows

# 2. ci.yml ve release.yml'i yukarıdaki içerikle oluştur

# 3. rust-toolchain.toml ekle
cat > rust-toolchain.toml << 'EOF'
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
EOF

# 4. Commit et
git add .github/ rust-toolchain.toml
git commit -m "chore: add CI pipeline"
git push origin main
# → CI tetiklenir, 50/50 test geçmeli
```

---

## Referans Belgeler

- [cargo-dist docs](https://opensource.axo.dev/cargo-dist/)
- [ed25519-dalek v2](https://docs.rs/ed25519-dalek/latest/)
- [Lemon Squeezy webhooks](https://docs.lemonsqueezy.com/api/webhooks)
- [Cloudflare Workers](https://developers.cloudflare.com/workers/)
- [Dependabot config](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-options-for-the-dependabot.yml-file)

---

*Versiyon: 1.0 | 2026-06-18*
