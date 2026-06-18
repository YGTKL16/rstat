# rstat — Proje Bağlamı (CLAUDE.md)
> Bu dosya her session başında/sonunda güncellenir. Son güncelleme: 2026-06-13

## Ne Bu?
Rust tabanlı, cross-platform, pipeline-friendly istatistik CLI aracı.
"scipy.stats + Minitab'ın komut satırı versiyonu — runtime gerektirmeyen profesyonel stats engine."

## Kurucu Profili
- Endüstri Mühendisliği 3. sınıf (askeri okul), 1 yıl içinde askeri görev
- Rust bilgisi var, IE + istatistik güçlü
- Solo geliştirici, düşük bakım hedefi, bulut bağımlılığı yok
- Hedef gelir: ~$10-15k

---

## Proje Durumu (Son Güncelleme: 2026-06-18)

### ✅ Faz 0 — Workspace Scaffold (TAMAMLANDI)
- Cargo workspace: `crates/rstat-core` (lib) + `crates/rstat-cli` (bin)
- Summary stats: mean, variance (Welford), std_dev, quantile, 6-summary
- Temel CLI altyapısı: CSV okuma, stdin pipe, format detection

### ✅ Faz 1 — t-test (TAMAMLANDI)
- `rstat ttest`: one-sample, two-sample (Welch/Pooled), paired
- scipy + R çapraz doğrulaması, 1e-10 tolerans
- One-sided CI fix: `ci_bounds()` → Less=[-∞,ub] / Greater=[lb,+∞] / TwoSided=simetrik
- Cohen's d etki büyüklüğü
- **Toplam test: 50/50 geçiyor** (27 core + 14 CLI + 9 summary)

### ✅ Faz 2 — ANOVA + CI (TAMAMLANDI)
- One-way ANOVA (SS, MS, F, p, η², ω² hesaplamaları)
- CI: Ortalama (t), Oran (Wilson, Wald), Varyans (χ²) güven aralıkları
- CLI komutları: `rstat anova` (geniş ve uzun format desteği), `rstat ci`

### ✅ Faz 3 — Chi-Square (TAMAMLANDI)
- Kontenjans tablosu/matrisi okuma (matris ve uzun format desteği)
- Bağımsızlık testi + Yates düzeltmesi + Cramér's V
- Uyum İyiliği (GoF) testi + otomatik frekans ölçeklendirme
- CLI komutu: `rstat chisq`

### ✅ Faz 4 — SPC: Kontrol Grafikleri (TAMAMLANDI)
- X-bar & R grafik limiti hesaplamaları (A2, D3, D4, d2 sabitleriyle)
- Western Electric 1-4 kuralları ihlal kontrolleri
- Çevrimdışı/Yerel lisans kapısı (`RSTAT_PRO` env var denetimi)
- CLI komutu: `rstat spc`

### ✅ Faz 5 — Proses Yeterliliği (TAMAMLANDI)
- Proses yeterlilik indeksleri (Cp, Cpk, Pp, Ppk)
- Normal dağılıma göre kısa/uzun dönem PPM hata tahmini
- Çarpıklığa (skewness) göre normallik uyarısı
- Çevrimdışı/Yerel lisans kapısı (`RSTAT_PRO` env var denetimi)
- CLI komutu: `rstat capability`

### ✅ Faz 6 — Çevrimdışı Lisanslama & Keygen (TAMAMLANDI)
- `rstat-license` doğrulama kütüphanesi (asimetrik Ed25519 anahtarı doğrulaması, payload determinizmi)
- `rstat-keygen` kurucu lisans üretim aracı (anahtar çifti oluşturma ve lisans imzalama komutları)
- CLI entegrasyonu (başlangıçta otomatik lisans yükleme ve testlerde RSTAT_LICENSE_FILE / RSTAT_PRO desteği)

### ✅ Faz 7 — Dağıtım & Release Altyapısı (TAMAMLANDI)
- Kök `Cargo.toml` üzerinde `cargo-dist` meta veri ve profil yapılandırması
- `rstat-cli/Cargo.toml` üzerinde `binstall` ve metadata yapılandırması
- `.github/workflows/ci.yml` (Format, Clippy, Test otomasyonu)
- `.github/workflows/release.yml` (Platformlar arası derleme, checksum ve GitHub Release otomasyonu)
- Kurulum ve entegrasyon şablonları: Cloudflare Worker webhook scripti, Homebrew formula, `install.sh` ve `install.ps1` betikleri

### 📋 Sonraki Fazlar
- Faz 8: Landing page/web sitesi yayını, Lemon Squeezy canlı satış entegrasyonu ve resmi v1.0.0 lansmanı.

---

## Teknik Stack & Kararlar

### Crate Yapısı
```
rstat/
  Cargo.toml              # workspace, resolver="2"
  crates/
    rstat-core/           # saf hesaplama, IO yok
      src/
        error.rs          # StatError enum
        result.rs         # SummaryStats, TTestResult, GroupStats
        data/summary.rs   # Welford variance
        dist/pvalue.rs    # p_value(), critical_t(), ci_bounds()
        tests_stat/ttest.rs
    rstat-cli/            # clap v4, CSV, format, render
      src/
        cli.rs            # Cli, Commands, TtestArgs, SummaryArgs
        io.rs             # read_column(), read_two_columns()
        render.rs         # OutputFormat::detect() (atty TTY check)
        commands/ttest.rs
      tests/
        cli_ttest.rs      # 14 integration test (assert_cmd)
        cli_summary.rs    # 9 integration test
        fixtures/         # one_col.csv, two_col.csv, with_na.csv
```

### Key Dependencies
```toml
# rstat-core
statrs = "0.18"      # ContinuousCDF trait: cdf, sf, inverse_cdf
thiserror = "2"
serde = { features = ["derive"] }
# dev: proptest = "1"

# rstat-cli
clap = { version = "4", features = ["derive"] }
csv = "1"
serde_json = "1"
comfy-table = "7"
atty = "0.2"         # TTY detection
anyhow = "1"
# dev: assert_cmd = "2", predicates = "3"
```

### Önemli Mimari Kararlar
| Konu | Karar | Neden |
|------|-------|-------|
| İki örnek varsayılan | Welch (equal_var=False) | scipy uyumluluğu |
| Varyans algoritması | Welford online | Büyük sayılarda cancellation hatası yok |
| Output format | TTY→tablo, pipe→JSON | `OutputFormat::detect()` via atty |
| Infinity JSON | `null` serialize | serde_json sonsuz sayıyı null yapar |
| İş modeli | Open-core | ttest/ci/chisq ücretsiz; SPC+capability lisanslı |
| Lisans | Offline Ed25519 imzalı | Bulut bağımlılığı yok |
| Dağıtım | cargo-dist + musl | Cross-platform, static binary |
| Çapraz doğrulama | scipy + R | scipy birincil, R ikincil |

---

## Komutlar

### Çalıştır
```bash
cargo build                             # derleme
cargo test                              # tüm testler (50/50)
cargo run --bin rstat-cli -- ttest --help
python3 scripts/crossvalidate_ttest.py  # scipy referans değerleri üret
```

### Doğrulama
```bash
echo "value\n1\n2\n3\n4\n5" | cargo run --bin rstat-cli -- ttest --kind one --col value --mu 3
cat crates/rstat-cli/tests/fixtures/one_col.csv | cargo run --bin rstat-cli -- summary --col value
```

---

## Test Yapısı (108/108 Geçiyor)
- **Core unit tests** (44): `pvalue.rs`, `summary.rs`, `ttest.rs`, `anova.rs`, `ci.rs`, `chisq.rs`, `spc/xbar_r.rs`, `capability/mod.rs` içinde `#[cfg(test)]`.
  - scipy-doğrulanmış p-değerleri ve sınırları (1e-9 tolerans)
  - proptest property-based testler (p∈[0,1], CI tutarlılığı, tamamlayıcı p-değerleri, ANOVA)
- **CLI integration** (64):
  - `tests/cli_ttest.rs` (14 entegrasyon testi)
  - `tests/cli_summary.rs` (9 entegrasyon testi)
  - `tests/cli_anova.rs` (9 entegrasyon testi)
  - `tests/cli_ci.rs` (11 entegrasyon testi)
  - `tests/cli_chisq.rs` (11 entegrasyon testi)
  - `tests/cli_spc.rs` (5 entegrasyon testi)
  - `tests/cli_capability.rs` (5 entegrasyon testi)

---

## Bilinen Sınırlamalar & Gelecek İş
- Şu an sadece f64 → Büyük integer veri setleri için precision kaybı olabilir
- NA/NaN handling: paired ve two-sample testlerde satır bazında drop, tek örnek için NaN skip
- Lisanslama: Şu an `RSTAT_PRO` ortam değişkeniyle stub lisans kapısı var. Faz 6'da Ed25519 offline imza doğrulaması ile tam entegre edilecek.

---

## Workflow Kuralı
Planlama → Opus | Implementasyon → Sonnet | Review → Sonnet | Basit görevler → Haiku

**Bu dosyayı her session başında oku, bitişinde güncelle.**
