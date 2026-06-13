# rstat — İmplementasyon Planı

> **Rust tabanlı, cross-platform, pipeline-friendly istatistik CLI aracı**
> "scipy.stats + Minitab'ın komut satırı versiyonu — runtime gerektirmeyen profesyonel stats engine."
> Hedef: Solo geliştirici, 1 yıl, $10-15k self-serve gelir.

---

## 0. Yönetici Özeti

**Doğruluk önce, hız ikinci, özellik üçüncü.**

İstatistik yazılımında bir tek yanlış p-değeri ürünün itibarını bitirir. Plan boyunca **R/Python çapraz doğrulama** birinci sınıf vatandaştır.

İki temel mimari karar:
1. **Çekirdek kütüphane (`rstat-core`) + ince CLI (`rstat-cli`) ayrımı.** İleride GUI, web servisi veya FFI gerekirse çekirdek dokunulmaz.
2. **Veri akışı = `Vec<f64>` / `Dataset` soyutlaması.** Her komut aynı parse → validate → compute → format hattını paylaşır.

---

## 1. Proje Yapısı

### Cargo Workspace

```
rstat/
├── Cargo.toml                  # [workspace]
├── Cargo.lock                  # commit edilir (binary projesi)
├── rust-toolchain.toml
├── deny.toml                   # cargo-deny: lisans + güvenlik
├── CLAUDE.md                   # AI bağlamı
│
├── crates/
│   ├── rstat-core/             # Saf hesaplama kütüphanesi
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs        # thiserror → StatError enum
│   │       ├── data/
│   │       │   ├── frame.rs    # Dataset soyutlaması
│   │       │   └── summary.rs  # mean, var, sd, quantile, skew, kurtosis
│   │       ├── dist/
│   │       │   ├── mod.rs      # statrs köprüsü: t, F, chi2, normal
│   │       │   └── pvalue.rs   # tek/çift kuyruk p-değeri
│   │       ├── tests_stat/
│   │       │   ├── ttest.rs    # one-sample, two-sample (Welch+pooled), paired
│   │       │   ├── anova.rs    # one-way ANOVA
│   │       │   └── chisq.rs    # bağımsızlık + uyum iyiliği
│   │       ├── interval/
│   │       │   └── ci.rs       # ortalama, oran, varyans CI
│   │       ├── spc/
│   │       │   ├── xbar_r.rs   # X-bar & R grafikleri + A2/D3/D4/d2 sabitleri
│   │       │   └── rules.rs    # Western Electric / Nelson kuralları
│   │       ├── capability/
│   │       │   └── mod.rs      # Cp, Cpk, Pp, Ppk
│   │       └── result.rs       # TestResult (serde Serialize)
│   │
│   └── rstat-cli/              # İnce kabuk — arg parse + I/O + format
│       └── src/
│           ├── main.rs
│           ├── cli.rs          # clap derive: Cli + Commands
│           ├── io/
│           │   ├── reader.rs   # CSV / stdin / kolon seçimi
│           │   └── detect.rs   # delimiter & header sezgisi
│           ├── render/
│           │   ├── table.rs    # comfy-table
│           │   ├── json.rs     # serde_json
│           │   └── csv_out.rs
│           └── commands/       # ttest, anova, chisq, ci, spc, capability
│
├── tests/                      # CLI entegrasyon testleri (assert_cmd)
│   └── fixtures/               # örnek CSV + beklenen JSON
│
├── benches/                    # criterion benchmark
│
├── scripts/
│   ├── crossvalidate.py        # scipy.stats referans değer üretici
│   └── crossvalidate.R         # R bağımsız referans
│
└── .github/workflows/
    ├── ci.yml                  # test + clippy + fmt + cross-validation
    └── release.yml             # cross-platform binary + GitHub Release
```

### Bağımlılık Yönü (Altın Kural)

```
rstat-cli  ──▶  rstat-core  ──▶  statrs
```

`rstat-core` asla `clap`, `csv`, `stdin` bilmez. Sadece `&[f64]` alır, `TestResult` döner.

---

## 2. Crate Seçimi

| Katman | Seçim | Gerekçe |
|---|---|---|
| Dağılımlar | `statrs` | t, F, χ², normal CDF/PDF/PPF — olgun, test edilmiş |
| CSV parse | `csv` crate | Hafif, şeffaf, `Vec<f64>` kontrolü |
| CLI parse | `clap` v4 (derive) | De-facto standart, subcommand, shell completion |
| Tablo çıktı | `comfy-table` | Unicode, otomatik kolon genişliği, aktif bakım |
| Serileştirme | `serde` + `serde_json` | `--format json` için |
| Hata yönetimi | `thiserror` (core) + `anyhow` (cli) | Tipli hata / kolay zincirleme |
| Renk/TTY | `owo-colors` + `anstream` | Sadece TTY'de renk, pipe'ta düz |

### Neden `polars` DEĞİL?
- Binary boyutu: polars → onlarca MB, csv crate → ~yüz KB
- rstat verisi tipik olarak binlerce satır — RAM'e rahat sığar
- polars lazy/SIMD avantajları milyon-satır ölçeğinde; rstat o ligde değil
- Ham `Vec<f64>` üzerinde tam kontrol = hata ayıklanabilirlik

### Neden `hypors`/`rs-stats` çekirdek bağımlılığı değil?
Test istatistiklerinin formülleri (t, F, χ²) **kapalı-form, literatür formülleri** — kendi yazarsak scipy/R ile doğrulama yapılabilir, bağımlılık riski sıfırlanır. Dağılım fonksiyonları (matematiksel olarak zor kısım) için `statrs`'a güvenilir.

---

## 3. Faz Planı (52 Hafta / Solo)

### Faz 0 — Temel & İskele (Hafta 1–3)
- [ ] Workspace kur, CI iskeleti (fmt + clippy + test)
- [ ] `Dataset` ve `TestResult` tiplerini tasarla
- [ ] `error.rs` — `StatError` enum
- [ ] CSV reader + stdin + kolon seçimi
- [ ] Özet istatistikler: mean, var (n-1), sd, min, max, quantile — scipy karşılaştırmalı test
- **Çıktı:** `echo "1,2,3" | rstat` özet istatistik basıyor

### Faz 1 — t-test (Hafta 4–7)
- [ ] `statrs` ile t-dağılımı CDF/PPF köprüsü
- [ ] One-sample, two-sample (Welch + pooled), paired
- [ ] CLI: `rstat ttest` tüm flag'leriyle
- [ ] Cross-validation harness kur (scripts/crossvalidate.py)
- **Çıktı:** scipy ile 1e-10 toleransında eşleşme — projenin doğruluk omurgası

### Faz 2 — ANOVA + Güven Aralıkları (Hafta 8–12)
- [ ] One-way ANOVA (SS, MS, F, p, η²)
- [ ] CI: ortalama (t), oran (Wilson + Wald), varyans (χ²)
- [ ] CLI: `rstat anova`, `rstat ci`
- [ ] R cross-validation ekle (ikinci bağımsız referans)

### Faz 3 — Chi-square (Hafta 13–15)
- [ ] Kontenjans tablosu parse
- [ ] Bağımsızlık testi + Yates düzeltmesi + Cramér's V
- [ ] CLI: `rstat chisq`

### Faz 4 — SPC: Kontrol Grafikleri (Hafta 16–22)
- [ ] X-bar & R: alt grup ort, kontrol limitleri (A2, D3, D4, d2 sabitleri)
- [ ] Western Electric / Nelson kuralları
- [ ] Terminal ASCII grafik + JSON limit çıktısı
- [ ] CLI: `rstat spc`
- **Çıktı: IE diferansiyasyonu başlıyor**

### Faz 5 — Proses Yeterliliği (Hafta 23–26)
- [ ] Cp, Cpk, Pp, Ppk + sigma seviyesi + PPM tahmini
- [ ] Normallik uyarısı
- [ ] CLI: `rstat capability`
- **MVP feature-complete (~6. ay)**

### Faz 6 — Sertleştirme & Dokümantasyon (Hafta 27–34)
- [ ] Entegrasyon testleri (assert_cmd) + fixture'lar
- [ ] Edge case'ler: boş veri, tek değer, NaN
- [ ] `--help` metinleri, örnekli README, formül referans dokümanı
- [ ] Shell completion (bash/zsh/fish/powershell)

### Faz 7 — Dağıtım & Lansman (Hafta 35–42)
- [ ] Cross-platform release pipeline (cargo-dist)
- [ ] Lisanslama mekanizması (offline Ed25519)
- [ ] Landing page, fiyatlandırma, ödeme (Lemon Squeezy)
- [ ] `cargo install` + `brew tap` + doğrudan binary
- **v1.0 satışta**

### Faz 8 — Tampon & İlk Geri Bildirim (Hafta 43–52)
- [ ] Erken kullanıcı geri bildirimi → düzeltmeler
- [ ] Post-hoc testler (Tukey HSD), non-parametrik (Mann-Whitney, Kruskal-Wallis)
- **Tampon zorunlu — askeri okul yükü gerçekçilikle karşılanmalı**

---

## 4. Her Komut İçin Tasarım

Tüm komutlar:
- Dosya argümanı yoksa **stdin** (pipeline-friendly)
- `--format table|json|csv` (varsayılan: table)
- `--alpha` (varsayılan: 0.05)
- Hata → stderr; sonuç → stdout

### 4.1 `rstat ttest`

```
rstat ttest [FILE]
  --type <one|two|paired>
  --col <NAME|IDX>                 # one-sample
  --col1 <..> --col2 <..>         # two/paired
  --group <COL> --value <COL>      # uzun-format two-sample
  --mu <FLOAT>                     # one-sample referans (varsayılan 0)
  --alt <two-sided|less|greater>
  --var <equal|unequal>            # pooled vs Welch (varsayılan: unequal)
  --alpha <FLOAT>
  --format <table|json|csv>
```

Hesaplamalar: one-sample `t=(x̄−μ)/(s/√n)`, Welch df (Welch-Satterthwaite), paired = farkların one-sample t. Cohen's d etki büyüklüğü.

**Tablo çıktısı:**
```
Two-Sample t-Test (Welch)
─────────────────────────────────────
              Group A    Group B
  n              30         28
  mean          12.43      10.81
  std            2.10       2.55
─────────────────────────────────────
  t = 2.642   df = 52.3   p = 0.0108
  95% CI for diff: [0.39, 2.85]
  Cohen's d = 0.69
  Decision: reject H0 at α=0.05
```

### 4.2 `rstat anova`

```
rstat anova [FILE]
  --value <COL> --group <COL>   # uzun format
  --cols <c1,c2,c3>             # geniş format
  --alpha <FLOAT>
  --posthoc <none|tukey>        # Faz 8'e ertelendi
```

Klasik ANOVA tablosu çıktısı: SS/df/MS/F/p + η².

### 4.3 `rstat chisq`

```
rstat chisq [FILE]
  --test <independence|gof>
  --yates                       # 2x2 Yates düzeltmesi
  --alpha <FLOAT>
```

Gözlenen/beklenen matris + χ²/df/p + Cramér's V. Düşük beklenen frekans (<5) uyarısı.

### 4.4 `rstat ci`

```
rstat ci [FILE]
  --type <mean|proportion|variance>
  --col <..>
  --level <FLOAT>               # 0.95
  --successes <INT> --trials <INT>
  --method <wilson|wald>
```

Mean: t-tabanlı. Proportion: Wilson (varsayılan). Variance: χ²-tabanlı.

### 4.5 `rstat spc`

```
rstat spc [FILE]
  --chart <xbar-r|i-mr>        # MVP: xbar-r
  --subgroup-size <INT>
  --value <COL> --subgroup <COL>
  --rules <we|nelson|none>
  --plot                        # terminal ASCII grafik
```

A2/D3/D4/d2 sabitleri iki kaynaktan doğrulanmış (Montgomery + ASTM). JSON çıktısında: `ucl`, `lcl`, `center_line`, `violations[]`.

### 4.6 `rstat capability`

```
rstat capability [FILE]
  --col <..>
  --lsl <FLOAT> --usl <FLOAT>
  --target <FLOAT>
  --sigma <within|overall>
```

Cp/Cpk/Pp/Ppk + sigma seviyesi + PPM. Normallik uyarısı (çarpıklık kontrolü).

---

## 5. Test Stratejisi

### Katman 1 — Birim Testleri (Rust)
- Literatür örnekleriyle altın değerler
- Edge case: n=1, varyans=0, NaN
- `approx` crate ile `assert_relative_eq!`

### Katman 2 — Çapraz Doğrulama (KRİTİK)
1. `scripts/crossvalidate.py` → scipy.stats referans JSON fixture üretir
2. `scripts/crossvalidate.R` → R bağımsız ikinci referans
3. Rust testleri aynı veriyi rstat'a verir, 1e-9 toleransla karşılaştırır
4. İki referans birbirleriyle de uyuşmalı (varsayılan farkları ele geçirir)

### Katman 3 — CLI Entegrasyon
- `assert_cmd`: gerçek binary, stdin/dosya/flag kombinasyonları
- stdout/stderr ayrımı, JSON şema geçerliliği, exit kodları

### Destekleyici
- `proptest`: "p değeri [0,1]", "CI alt ≤ üst"
- `criterion`: hız regresyonu takibi
- CI gate: test + clippy + fmt + cross-validation geçmeden merge yok

---

## 6. Dağıtım Planı

### Hedef Platformlar (Tier 1)

| Platform | Triple |
|---|---|
| Linux x86_64 | `x86_64-unknown-linux-musl` (statik) |
| Linux ARM64 | `aarch64-unknown-linux-musl` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| Windows x86_64 | `x86_64-pc-windows-msvc` |

**musl** → statik binary, sıfır runtime bağımlılığı, eski Linux'ta çalışır.

### Pipeline
- **`cargo-dist`**: cross-platform build, GitHub Releases, installer script, binstall desteği — solo geliştirici için minimum bakım yükü.
- Git tag (`v*`) → tüm hedefler build → release → SHA256 checksum.

### Kanallar
1. GitHub Releases (birincil)
2. `cargo install` / `cargo binstall`
3. Homebrew tap
4. Scoop / WinGet (Windows, ileri faz)

### Para Kazanma
- **Açık-çekirdek modeli:** ttest/ci/chisq ücretsiz; SPC + capability lisanslı (IE/kalite müh. tam bu ikisine para öder)
- **Lemon Squeezy** (merchant-of-record, vergi halleder, bulut altyapısı gerektirmez)
- **Offline Ed25519 imzalı lisans** (binary içinde public key gömülü — çevrimdışı doğrulama, bulut yok)

---

## 7. Riskler

| Risk | Etki | Önlem |
|---|---|---|
| Yanlış p-değeri / formül varyantı | Ürün-öldüren | Çift çapraz doğrulama (scipy + R), CI gate |
| Float hassasiyeti / overflow | Orta | Welford varyans algoritması; log-uzayı küçük p için |
| SPC sabit tabloları yanlış | Yüksek | A2/D3/D4 iki kaynaktan doğrula + qcc (R) karşılaştırma |
| Kapsam şişmesi | Yüksek | MVP 6 komut SABİT; post-hoc/non-parametrik v2'ye |
| Askeri okul yoğun dönemleri | Orta | Son 10 hafta tampon; her faz bağımsız teslim edilebilir |

### Açık Kararlar (İlk Hafta Netleştir)
1. **Lisans modeli:** açık-çekirdek mi, tamamen kapalı mı?
2. **Varsayılan format:** TTY'de tablo, pipe'ta otomatik JSON mı, yoksa her zaman açık `--format` mı?
3. **Two-sample t varsayılanı:** Welch (önerilen) mi, pooled mi?

---

## Özet — Kritik Kararlar

| Konu | Karar |
|---|---|
| Mimari | 2-crate workspace: `rstat-core` + `rstat-cli` |
| Veri | `csv` crate → `Vec<f64>` (polars değil) |
| Dağılımlar | `statrs` + test formülleri elle yazılır |
| Doğruluk | scipy + R çift çapraz doğrulama, CI gate |
| Dağıtım | `cargo-dist`, musl statik, GitHub Releases |
| Para kazanma | Açık-çekirdek + Lemon Squeezy + offline Ed25519 |
| Diferansiyasyon | SPC + capability — Minitab'ı CLI'a taşımak |
| Zaman | MVP ~6 ay, +6 ay sertleştirme + dağıtım + tampon |

**İlk somut adım → Faz 0: workspace iskelesi + `Dataset`/`TestResult` tipleri + ilk scipy karşılaştırmalı özet istatistik testi.**
