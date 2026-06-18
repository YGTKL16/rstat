# rstat — Ürün Stratejisi & Go-to-Market Belgesi

> **Versiyon:** 1.0 | **Tarih:** Haziran 2026 | **Yazar:** Ürün Stratejisti (AI)  
> **Kapsam:** Pazar analizi, fiyatlandırma, GTM planı, web varlığı, riskler, 12 aylık takvim

---

## ⚡ Özet: 3 Kritik Karar

Belgeyi okumadan önce bilinmesi gereken üç şey:

1. **Fiyatlandırma:** Open-core DEĞİL — **tek seferlik ödeme ($99 bireysel / $249 küçük takım)** bireysel satış hedefi için çok daha fazla gelir getirir.
2. **İlk Hedef Kitle:** Veri mühendisleri değil — **endüstri mühendisleri ve kalite mühendisleri** (SPC/Capability özelliklerinin gerçek alıcıları).
3. **Web:** Landing page zorunlu değil — **GitHub README + tek Netlify sayfası** yeterli, WASM demo sonra.

---

## 1. Pazar Pozisyonu Analizi

### 1.1 Rekabet Haritası

```
                    ÜCRETLİ
                       │
        Minitab ●      │
        ($1800+/yıl,   │
        Windows only)  │
                       │
KAPALI ────────────────┼──────────────────── AÇIK
(Closed)               │                  (Open)
                       │        ● qsv
                       │        (ücretsiz, tanımlayıcı)
                       │        ● datamash
                       │        (temel, sadece özet)
                       │        ● miller
                       │        (format dönüşümü, stats yok)
                    ÜCRETSİZ
```

**rstat'ın hedeflediği boşluk:**
```
              $99-249 (makul fiyat)
                       │
                       │
KAPALI ────────────────┼──────────────────── AÇIK
                  ★ rstat                
              (inferential stats,         
               SPC, Capability,           
               Linux'ta çalışır,          
               pipeline-friendly)         
```

### 1.2 Rakip Analizi — Gerçek Boşluklar

| Rakip | Güçlü | Zayıf | rstat Fırsatı |
|-------|-------|-------|---------------|
| **Minitab** | Kapsamlı, endüstri standardı | Windows-only, $1,800+/yıl, GUI bağımlı | Linux + CLI + 20x ucuz |
| **R (qcc paketi)** | Ücretsiz, güçlü | Runtime gerektirir, pipe'a girmiyor, öğrenme eğrisi | Sıfır kurulum, tek binary |
| **Python scipy** | Ücretsiz, esnek | Runtime, script yazmak gerekiyor, tek satır t-testi bile 5 satır kod | `cat data.csv \| rstat ttest` → tek satır |
| **qsv** | Hızlı, popüler | Tanımlayıcı stats, hipotez testi YOK | Tüm inferential katman |
| **xsv** | Ultra hızlı | Arşivlendi (2025), stats yok | Devam eden bakım + stats |
| **datamash** | Pipeline-friendly | Sadece mean/sum/sd, p-değeri yok | Profesyonel test sonuçları |
| **JASP** | GUI, Minitab alternatifi | GUI, automation yok | CI/CD pipeline entegrasyonu |

> [!IMPORTANT]
> **Gerçek Boşluk:** $0 ile $1,800/yıl arasında **profesyonel inferential statistics + SPC + Capability** yapabilen, **Linux'ta çalışan**, **pipeline-friendly** ve **runtime gerektirmeyen** bir araç YOK.

### 1.3 Hedef Kullanıcı Persona'ları

---

#### 🧑‍🔧 Persona 1: "Fabrika Kedi" — Kalite Mühendisi

**İsim:** Mert, 34, Bursa'daki bir otomotiv Tier-1 tedarikçisinde kalite mühendisi  
**Araçlar:** Minitab (lisans şirket tarafından karşılanıyor ama yeni versiyona geçiş yavaş), Excel, şirkete ait Ubuntu sunucular  
**Acı noktası:** Minitab Windows-only, CI/CD pipeline'ına bağlanamıyor. Fabrika sunucusunda Python yok ve IT departmanı kurulum izni vermiyor. Günde 3 kez X-bar-R grafiği çizmesi gerekiyor.  
**Ne istiyor:** `scp data.csv server: && ssh server "cat data.csv | rstat spc --chart xbar-r --json"` → bitti.  
**Ödeme isteği:** Şirketi ödüyor zaten — $249 ekip lisansı için bütçe tamam.  
**Kanal:** LinkedIn (kalite mühendisliği grupları), r/industrialengineering, iş arkadaşı tavsiyesi.

---

#### 👩‍💻 Persona 2: "Pipe Fanatiği" — Veri Mühendisi

**İsim:** Alara, 28, bir fintech startup'ta veri mühendisi, tam remote  
**Araçlar:** Python, pandas, her şey Linux/macOS terminalde  
**Acı noktası:** A/B testi sonuçları için her seferinde Python scripti kurmak zorunda kalıyor. `scipy.stats.ttest_ind` işe yarıyor ama JSON çıktısı yok, grep ile kimse okuyamıyor.  
**Ne istiyor:** `cat ab_results.csv | rstat ttest --col treatment --col control --format json | jq '.pvalue'` → otomatik alerting pipeline.  
**Ödeme isteği:** $99 bireysel lisans → hızlıca kendi kartından alır, masraf formunu bile doldurmaz.  
**Kanal:** HackerNews "Show HN", r/dataisbeautiful, r/dataengineering.

---

#### 🎓 Persona 3: "Makale Deposu" — Akademik Araştırmacı

**İsim:** Prof. Kemal, 45, makine mühendisliği doçenti, çalışmalarında üretim kalitesi analizi yapıyor  
**Araçlar:** MATLAB (pahalı ama üniversite lisansı var), R (bazen), Linux sunucu  
**Acı noktası:** Tekrarlanabilir araştırma için tüm analizi script'e geçirmesi lazım. MATLAB GUI güzel ama otomasyona girmiyor. R kurulumu kırılgan.  
**Ne istiyor:** Makalede `rstat capability --lsl 9.5 --usl 10.5` çıktısını direkt gösterebilmek. Tek binary = tekrarlanabilir = hakemler de doğrulayabilir.  
**Ödeme isteği:** Üniversite satın alma süreci yavaş ama kendi cebinden $99 verebilir; eğitim indirimi varsa daha iyi.  
**Kanal:** ResearchGate, akademik blog'lar, r/statistics, Stack Overflow.

---

### 1.4 Unique Value Proposition (Tek Cümle)

> **"Minitab'ın kalite analizini Linux pipeline'ına taşıyan, runtime gerektirmeyen tek Rust CLI."**

Alternatif (daha teknik kitleye):
> **"cat data.csv | rstat → t-test, ANOVA, SPC, Capability — runtime yok, kurulum yok, output JSON."**

---

## 2. Fiyatlandırma Kararı

### 2.1 Model Karşılaştırması

#### Model A: Open-Core (Orijinal Plan)
- ttest/ci/chisq ücretsiz
- SPC + Capability ücretli

| Kriter | Değerlendirme |
|--------|---------------|
| **Gelir potansiyeli** | Orta. Ücretsiz kısım viral olur ama ödeyen kullanıcıya geçiş (%1-3 conversion) zayıf. |
| **Bakım yükü** | YÜKSEK. İki farklı binary, lisans sistemi, ücretsiz sürüm şikayetleri. |
| **Risk** | Ücretsiz kısım fork'lanabilir (MIT ise). |
| **Pasif gelir** | Orta. Yeni kullanıcı akışına bağlı. |
| **Tahmini gelir (12 ay)** | 50 kullanıcı × $149 = **$7,450** — hedefin altı. |

#### Model B: Tamamen Ücretli (Kapalı Kaynak)
- Tüm binary ücretli, GitHub sadece README

| Kriter | Değerlendirme |
|--------|---------------|
| **Gelir potansiyeli** | Yüksek per-seat ama keşfedilebilirlik SIFIR. |
| **Bakım yükü** | Düşük (tek binary). |
| **Risk** | Güven sorunu — kimse kod görmeden satın almaz. |
| **Pasif gelir** | İyi — ama kullanıcı bulmak zor. |
| **Tahmini gelir (12 ay)** | 30 kullanıcı × $299 = **$8,970** — güvenilirlik sorunuyla uğraş. |

#### Model C: ⭐ Tek Seferlik Lisans + Açık Kaynak Core (ÖNERİLEN)
- `rstat-core` MIT lisansı (GitHub'da, fork edilebilir, kütüphane)
- `rstat-cli` binary → ücretli (kaynak kodu GitHub'da görünür ama binary'yi indirmek ücretli)
- Akademik/öğrenci: %50 indirim
- Küçük takım (3-10): $249 (tek ödeme, birden fazla bilgisayar)

| Kriter | Değerlendirme |
|--------|---------------|
| **Gelir potansiyeli** | Yüksek. Tek ödeme → düşük sürtünme, yüksek conversion. |
| **Bakım yükü** | Düşük. Abonelik yok, faturalandırma yok, churn yok. |
| **Risk** | Kaynak forklama riski var ama kullanıcıların %95'i derleme zahmetine girmez. |
| **Pasif gelir** | ⭐ **Mükemmel.** Askeri görevde de Lemon Squeezy ödeme alır. |
| **Tahmini gelir (12 ay)** | 70 bireysel × $99 + 20 ekip × $249 = **$11,910** 🎯 |

#### Model D: Freemium (Deneme Sürümü)
- 30 günlük deneme → lisans al

| Kriter | Değerlendirme |
|--------|---------------|
| **Gelir potansiyeli** | İyi ama geçici kullanıcıları elde tutmak zor. |
| **Bakım yükü** | YÜKSEK. Deneme süresi kontrolü, lisans expiry logic. |
| **Risk** | 30 gün yeter → "bir projede kullandım, gerek yok" diyebilir. |
| **Tahmini gelir (12 ay)** | Öngörülmesi zor — **reddediyorum**. |

---

### 2.2 ÖNERİLEN Fiyatlandırma Yapısı

```
┌─────────────────────────────────────────────────────────┐
│  rstat — Fiyatlandırma                                  │
├───────────────┬─────────────────┬───────────────────────┤
│  Bireysel     │  Küçük Takım    │  Akademik/Öğrenci     │
│  $99          │  $249           │  $49                  │
│  (tek ödeme)  │  (1-10 kişi)    │  (.edu eposta ile)    │
├───────────────┴─────────────────┴───────────────────────┤
│  ✓ Tüm komutlar (ttest, anova, chisq, ci, spc, cap.)   │
│  ✓ Tüm platformlar (Linux, macOS, Windows)              │
│  ✓ Lifetime lisans — abonelik yok                       │
│  ✓ 1 yıl güncelleme (sonrası $29 güncelleme ücreti)    │
│  ✓ Offline Ed25519 doğrulama — internet bağlantısı yok │
└─────────────────────────────────────────────────────────┘
```

> [!TIP]
> **Neden $99 ve $149/yıl değil?** Tek ödeme → developer topluluğunda "subscription fatigue" yok, alım kararı anlık. Abonelik için güncellemelerin sürekli gelmesi gerekir — askeri görevde bu mümkün değil.

### 2.3 Gelir Senaryoları

| Senaryo | Bireysel | Ekip | Akademik | **Toplam** |
|---------|----------|------|----------|-----------|
| **Kötümser** | 40 × $99 = $3,960 | 8 × $249 = $1,992 | 10 × $49 = $490 | **$6,442** |
| **Gerçekçi** | 80 × $99 = $7,920 | 18 × $249 = $4,482 | 15 × $49 = $735 | **$13,137** ✅ |
| **İyimser** | 130 × $99 = $12,870 | 30 × $249 = $7,470 | 25 × $49 = $1,225 | **$21,565** |

---

## 3. Go-to-Market Planı

### 3.1 İlk 10 Beta Kullanıcıyı Nereden Bulursun?

> [!NOTE]
> Beta: Ücretsiz, süre sınırlı değil. Karşılıklı beklenti: gerçek kullanım + geri bildirim.

**Kanal 1: r/rust (Reddit)**  
- Post: "Show r/rust: rstat — Rust ile yazdım, pipeline-friendly t-test/ANOVA/SPC CLI"
- Ton: Teknik, mütevazı, kodu göster
- Beklentim: 50-150 upvote, 3-8 beta başvurusu
- Zamanlama: MVP feature-complete olunca (SPC dahil)

**Kanal 2: r/dataisbeautiful / r/datascience**  
- Post: Terminal çıktısının güzel ekran görüntüsü + kullanım örneği
- "cat data.csv | rstat spc --plot" terminal ASCII grafiği büyük ilgi çeker
- Beklentim: 5-10 beta başvurusu

**Kanal 3: r/industrialengineering**  
- Post: "Minitab'a alternatif arayan var mı? Linux pipeline'a giren SPC aracı geliştiriyorum"
- Bu topluluk Minitab acısını çok iyi biliyor — doğrudan hedef
- Beklentim: 3-5 kaliteli (IE/kalite müh.) beta kullanıcı

**Kanal 4: LinkedIn (Türkiye IE/Kalite ağı)**  
- Hedef: TOBB bağlantıları, kalite mühendisliği mezunlar
- "Türk yazılımcı olarak Minitab alternatifi üzerinde çalışıyorum" → yerli övünç etkisi
- Beklentim: 2-4 beta kullanıcı + PR etkisi

**Kanal 5: Hacker News "Who's Hiring" / "Ask HN"**  
- "Ask HN: What do you use for statistical tests in shell pipelines?"
- Sorular post olduğu için cevap vermek natural — rstat'ı tanıt
- Beklentim: 1-3 beta kullanıcı, ama kalite çok yüksek

**Kanal 6: Twitter/X — Rust topluluğu**  
- @rustlang, @this_week_in_rust etiketle
- GIF/video: Terminal'de canlı t-test çalışması
- Beklentim: 2-5 beta başvurusu

**Kanal 7: Dev.to yazısı**  
- "Rust ile CLI istatistik aracı yazmak: bir IE öğrencisinin hikayesi"
- SEO değeri uzun vadeli, anlık traffic değil
- Beklentim: Uzun vadede Google search'ten trafik

**Kanal 8: Hacettepe / ODTÜ / İTÜ IE toplulukları**  
- Telegram/Discord grupları, LinkedIn mezun ağları
- "IE öğrencisi CLI stats aracı yazıyor" — üniversite gururu + kullanım ihtiyacı
- Beklentim: 3-5 beta, güçlü feedback

**Kanal 9: GitHub Awesome Lists**  
- awesome-rust, awesome-cli-apps, awesome-statistics listelerine PR
- Organik discovery için en değerli uzun vadeli yatırım
- Zamanlama: v1.0 stabil olduktan sonra

**Kanal 10: Stack Overflow (pasif)**  
- "How to run t-test from command line?" sorularına cevap yaz, rstat'ı göster
- Organik, sürekli trafik kaynağı

---

### 3.2 Lansman Sıralaması

```
Evre 1: Sessiz Beta (SPC tamamlanınca)
  └─ 10 beta kullanıcı, seçilmiş, feedback odaklı

Evre 2: v1.0 Soft Launch (tüm komutlar hazır + sertleştirme)
  └─ Lemon Squeezy hazır, landing page canlı, HN Show HN

Evre 3: Büyüme (1-3 ay sonra)
  └─ Blog yazıları, awesome list PR'ları, akademik indirim
```

**Evre 1: Sessiz Beta (Ay 4-5 sonrası)**
- Hedef: 10 gerçek kullanıcı
- Araç: Google Form başvuru sayfası (Notion veya Tally.so — ücretsiz)
- Kriter: IE, kalite müh., veri müh. — rastgele değil seçilmiş
- Beklenti: Haftalık feedback turu (15 dk görüşme veya Discord)

**Evre 2: v1.0 Soft Launch (MVP feature-complete + sertleştirme)**
- Lemon Squeezy sayfası hazır
- Minimal landing page yayında
- cargo-dist ile tüm platformlarda binary mevcut
- HN "Show HN" postu

**Evre 3: Büyüme (Lansman sonrası 1-3 ay)**
- Blog yazıları: "rstat ile SPC nasıl yapılır?"
- Awesome list PR'ları
- LinkedIn kalite mühendisliği topluluklarına erişim
- Akademik indirim tanıtımı

---

### 3.3 Viral Olabilecek İçerikler

#### 🎬 Viral İçerik 1: "Minitab'ı terminale taşıdım"
```bash
# Minitab'da 5 tıklama gerektiren şey:
cat process_data.csv | rstat capability --lsl 9.5 --usl 10.5
```
Terminal çıktısı ekran görüntüsü → Twitter/LinkedIn → kalite mühendisleri paylaşır.

#### 🎬 Viral İçerik 2: ASCII SPC Grafiği
```
X-bar Chart — Subgroup Mean
─────────────────────────────────────────────────────────
  12.5 ┤                    ● UCL=12.41
  12.0 ┤    ●    ●    ●    ●
  11.5 ┤  ●   ●    ●    ●    ●    ●   ─ CL=11.43
  11.0 ┤●                        ●    ●
  10.5 ┤                              ─ LCL=10.45
─────────────────────────────────────────────────────────
  Violations: Rule 1 at subgroup 7 (point beyond 3σ)
```
Bu görsel Reddit/Twitter'da çok paylaşılır — "terminal'de kontrol grafiği" rare.

#### 🎬 Viral İçerik 3: Benchmark karşılaştırması
```
A/B test analizi:
- Python scipy: pip install + 15 satır kod + json parse = ~2 dk
- rstat: cat results.csv | rstat ttest --format json | jq '.p_value' = 3 saniye
```

#### 🎬 Viral İçerik 4: "Askeri öğrenci CLI tool yazdı" hikayesi
- Personal story angle: IE 3. sınıf askeri okul öğrencisi, askeri göreve gitmeden önce…
- HN topluluğu bu tür hikayeleri sever
- Görünürlük: 1,000-10,000 kişiye ulaşabilir

---

## 4. Web Varlığı Önerisi

### 4.1 Seçenekler ve ROI Analizi

| Seçenek | Maliyet | Süre | Trafik Değeri | Bakım |
|---------|---------|------|---------------|-------|
| Sadece GitHub | $0 | 0 gün | Düşük (keşfedilemezlik) | Minimal |
| Minimal Landing Page | $0-15/ay | 1-2 gün | Orta | Düşük |
| WASM Demo | $0-30/ay | 2-4 hafta | Yüksek | **Orta-Yüksek** |
| Tam site + blog | $30+/ay | 1-2 ay | Çok yüksek | **Çok Yüksek** |

> [!WARNING]
> WASM demo teknik olarak çekici ama yanlış öncelik: Önce ödeyen 10 kullanıcı bul, sonra demo yaz. WASM demo Rust CLI'ı web'e taşımak demek, bu ciddi iş yükü.

### 4.2 ÖNERİLEN: Minimum Viable Web Presence

**Aşama 1 — v1.0 Öncesi (Şimdi):**
- Güçlü bir **GitHub README** (tek sayfa, demo GIF, install komutları, feature listesi)
- Mevcut: Hepsi zaten teknik toplulukta yeterli

**Aşama 2 — v1.0 Lansmanında:**

Tek sayfalık landing page → **Netlify veya GitHub Pages (ücretsiz)**

```
rstat.io (veya rstat-cli.com)
─────────────────────────────
[Hero] "Minitab'ı terminale taşı" 
       + terminal demo animasyonu (asciinema embed)

[Özellikler] ttest / anova / chisq / ci / spc / capability
             → kısa açıklamalar, komut örnekleri

[Install] cargo install rstat
          brew install ...
          curl -L ... | sh

[Fiyat] Bireysel $99 | Ekip $249 | Akademik $49
        [Satın Al →] (Lemon Squeezy link)

[GitHub] 
─────────────────────────────
```

**Araç: Astro + Tailwind CSS**  
Neden: Static, ultra hızlı, Netlify'a 1 komutla deploy, bakım sıfır.

**Domain:** `rstat.rs` (Rust için tematik) veya `rstat.sh` (CLI teması)  
Maliyet: ~$10-15/yıl

**Aşama 3 — Askeri Görev Öncesi (Opsiyonel):**
- asciinema embed ile terminal demo (WASM değil, daha kolay)
- Tek blog yazısı: "rstat nasıl çalışır?"
- SEO değeri: "CLI statistics", "Minitab alternative Linux"

> [!TIP]
> **WASM Demo ne zaman?** Kullanıcılardan "keşke web'de deneyebilseydim" yorumu 5 kez geldiğinde yaz. Şu an öncelik değil.

---

## 5. Riskler ve Azaltma

### 5.1 Risk: Askeri Görevde Ürün Terk Edilmesi

**Senaryo:** 12 ay sonra subay oluyorsun. Kullanıcılar var, ödeme gelmeye devam ediyor, ama yeni özellik yok, bug fix yok.

**Azaltma Planı:**

```
Askeri Görev Öncesi Yapılacaklar (Ay 10-12):
─────────────────────────────────────────────
1. CHANGELOG ve roadmap duyurusu:
   "v1.x stabil, aktif geliştirme durabilir"
   → Dürüstlük = kullanıcı güveni korunur

2. GitHub Issues → "help wanted" etiketleri
   → Community contributions olabilir

3. Lemon Squeezy otopilota devam eder:
   → Satış otomatik, teknik destek gerekmez
   
4. GitHub Discussions veya basit FAQ:
   → Kullanıcılar birbirlerine yardım edebilir

5. "Lifetime license" sözü ver:
   → Yeni versiyon çıkınca $29 ek ödeme
   → Askeri görev dönüşünde v2.0 ile canlan
```

> [!NOTE]
> Rust CLI'ların en büyük avantajı: **bakım minimumdur**. CSV formatı değişmez, istatistik formülleri değişmez. xsv 2025'te arşivlendi ama insanlar hâlâ kullanıyor.

**Açık Mesaj (README'de):** "Bu araç bir Türk askeri üniversite öğrencisi tarafından yapılmıştır. [tarih] itibariyle aktif geliştirme yavaşlayabilir, ama mevcut özellikler tamamen fonksiyonel."  
→ HN topluluğu bu tür hikayeleri sever ve destekler.

---

### 5.2 Risk: Kullanıcı Bulamama

**Senaryo:** Lansman yaptın, 5 kişi indirdi, satış sıfır.

**Erken Uyarı Sinyalleri:**
- Beta başvurusu < 5 kişi → kanal değiştir
- GitHub star 3 ayda < 50 → mesajlaşma değiştir
- v1.0'da 30 günde < 3 satış → fiyat indir veya freemium dene

**Azaltma:**
1. Beta aşamasında 10 kullanıcı bulana kadar lansman yapma
2. Her beta kullanıcısıyla 15 dk görüş — neyi sevdiler, ne eksik
3. İlk satışı yakın çevreden al (üniversite hocası, IE mezun ağı)
4. Eğer hiç çalışmazsa: `rstat-core`'u MIT ile aç → kütüphane olarak ekosistem değeri

---

### 5.3 Risk: Rakip Çıkması

**Senaryo A:** qsv inferential stats ekliyor.  
→ qsv'nin odağı büyük veri wrangling. İnferential stats eklemek mimariye aykırı. **Olasılık: düşük.**

**Senaryo B:** Biri rstat'ı fork'layıp "rstats-free" yapıyor.  
→ Core zaten MIT. Fork edilebilir. Ama binary + destek + marka = senin. Çoğu kullanıcı orijinali seçer.

**Senaryo C:** Büyük bir şirket (Datadog, Grafana Labs) benzer araç çıkarıyor.  
→ Odakları farklı. Ama olursa: niche'i derinleştir (SPC), fiyat düşür.

**Senaryo D:** Minitab CLI versiyonu çıkarıyor.  
→ $1,800'lık ürün $99'a inemez. Linux destekleri en az 2 yıl alır. Zaman avantajın var.

> [!CAUTION]
> **Gerçek risk:** "Kimse Rust stats CLI aramıyor" — yani talep yokluğu. Beta aşamasında 10 gerçek kullanıcı = kanıt. Bu olmadan ilerle değil.

---

### 5.4 Risk: Teknik — Yanlış p-değeri

**Senaryo:** Üretimde bir bug, kullanıcı yanlış kararlar alıyor.  
→ **İtibar bitici.**

**Mevcut Azaltma (zaten yapılıyor):**
- scipy + R çift çapraz doğrulama ✅
- 1e-10 tolerans ✅
- CI gate ✅

**Eklenecek:**
- v1.0 öncesi: Minitab'ın kendi dokümantasyon örnekleri ile el ile doğrulama
- Formül referans dokümanı (hangi formül, hangi kaynak)

---

## 6. 12 Aylık Takvim

### 6.1 Yoğunluk Haritası

| Ay | Odak | Başarı Kriteri |
|----|------|----------------|
| **Temmuz 2026** ████ | ANOVA + Güven Aralıkları (Faz 2) | 15 yeni test, scipy+R uyuşuyor |
| **Ağustos 2026** ███ | Chi-square (Faz 3) | 10 yeni test, Minitab el doğrulaması |
| **Eylül 2026** ████ | SPC başlangıcı (Faz 4) ⭐ | X-bar UCL/LCL, qcc eşleşmesi |
| **Ekim 2026** ███ | SPC tamamlama | ASCII grafiği terminal'de çalışıyor |
| **Kasım 2026** ██ | Capability + Beta açılış | 5 beta başvurusu |
| **Aralık 2026** ██ | Sertleştirme + v1.0 hazırlık | cargo dist release 5 platform |
| **Ocak 2027** ████ | v1.0 LANSMAN 🚀 | 100+ star, ilk 5 satış |
| **Şubat 2027** ███ | Kullanıcı geri bildirimi + hotfix | 10 satış kümülatif |
| **Mart 2027** ██ | Büyüme içeriği, awesome list PR | 30 satış kümülatif |
| **Nisan 2027** █ | Pasif mod başlıyor | Lemon Squeezy otopilot |
| **Mayıs 2027** █ | Pasif satış | — |
| **Haziran 2027** █ | Değerlendirme + v2.0 planı | $10k+ kümülatif |

### 6.2 "Minimum Viable Sprint" Tanımı

> Askeri okulun en yoğun dönemlerinde bile yapılabilecek en küçük anlamlı adım.

**MVS = 90 dakika × haftada 2 kez**

Yoğun dönemde bile yapılabilenler:
- 1 unit test yaz (15 dk)
- 1 edge case fix (30 dk)
- 1 README cümlesi düzelt (10 dk)
- 1 GitHub issue cevapla (15 dk)
- 1 Lemon Squeezy kontrolü (5 dk)

### 6.3 Aylık Detaylı Hedefler

#### Temmuz 2026 — ANOVA + Güven Aralıkları
- [ ] One-way ANOVA: SS/MS/F/p/η² — scipy doğrulamalı
- [ ] CI: ortalama (t-tabanlı), oran (Wilson), varyans (χ²)
- [ ] CLI: `rstat anova`, `rstat ci` tam çalışıyor
- [ ] R çapraz doğrulama eklendi (ikinci referans)

#### Ağustos 2026 — Chi-square
- [ ] Bağımsızlık testi (kontenjans tablosu)
- [ ] Yates düzeltmesi (2x2), Cramér's V
- [ ] CLI: `rstat chisq`

#### Eylül 2026 — SPC Başlangıcı ⭐
- [ ] A2/D3/D4/d2 sabitleri (Montgomery + ASTM çift kaynak)
- [ ] X-bar hesaplama + UCL/LCL
- [ ] Western Electric kuralları (Rule 1-4)

#### Ekim 2026 — SPC Tamamlama
- [ ] I-MR grafik (bireysel ölçüm)
- [ ] Terminal ASCII grafiği (`--plot`)
- [ ] JSON: `ucl`, `lcl`, `center_line`, `violations[]`
- [ ] CLI: `rstat spc` tam çalışıyor

#### Kasım 2026 — Capability + Beta Açılış
- [ ] Cp/Cpk/Pp/Ppk + sigma seviyesi + PPM
- [ ] Normallik uyarısı
- [ ] CLI: `rstat capability`
- [ ] Beta başvuru formu aç (Tally.so)
- [ ] r/rust + r/industrialengineering ilk duyuru

#### Aralık 2026 — Sertleştirme + v1.0 Hazırlık
- [ ] Entegrasyon testleri tüm komutlar (assert_cmd)
- [ ] Edge case'ler: boş veri, tek değer, NaN
- [ ] Shell completion (bash/zsh/fish)
- [ ] cargo-dist pipeline çalışıyor
- [ ] Lemon Squeezy ürün oluşturuldu
- [ ] Minimal landing page canlı (Netlify)
- [ ] 10 beta kullanıcıdan feedback alındı

#### Ocak 2027 — v1.0 LANSMAN 🚀
- [ ] GitHub Release + CHANGELOG
- [ ] HN "Show HN" postu (Salı 09:00 EST)
- [ ] r/rust + r/datascience + r/industrialengineering postu
- [ ] LinkedIn duyurusu (Türkiye IE ağı)
- [ ] Twitter/X + terminal GIF
- [ ] Awesome-rust PR açıldı

#### Şubat 2027 — İlk Geri Bildirim
- [ ] Kullanıcı raporları → hotfix'ler
- [ ] Awesome-cli-apps, awesome-statistics PR'ları
- [ ] Stack Overflow cevapları

#### Mart 2027 — Büyüme İçeriği
- [ ] Dev.to teknik yazı
- [ ] asciinema demo
- [ ] Akademik indirim kodları

#### Nisan-Haziran 2027 — Pasif Mod
- Lemon Squeezy otopilotta
- Kritik bug varsa MVS ile fix
- GitHub Issues açık, community aktif

---

## 7. Bonus: AI/LLM Entegrasyonu (2026-2027 Trend)

rstat'ı LLM aracı olarak çerçevele — ek geliştirme gerekmez:

```bash
# LLM agent: "istatistiksel test yap" dediğinde
cat data.csv | rstat ttest --format json
# → Temiz JSON → LLM parse eder → kullanıcıya anlatır
```

**Neden değerli:**
- LLM'ler hesaplama yapmak yerine araç çağırıyor (MCP trend)
- rstat'ın JSON çıktısı LLM için mükemmel — zaten var
- README'de "Use with AI assistants" bölümü → viral potansiyel

**Ne gerekir:** Hiçbir şey ekstra. Sadece README'de "AI-friendly JSON output" başlığı.

---

## 8. Co-Founder Tavsiyesi

> Sana danışman olarak değil, co-founder olarak konuşuyorum:

### ✅ Yap:

**Fiyatlandırma:** $99 bireysel, tek ödeme. Abonelik değil. Bunu v1.0'dan itibaren koru.

**İlk müşteri:** r/industrialengineering'deki bir kalite mühendisini bul. Endüstri mühendisleri olmak zorundalar, Minitab acısını biliyorlar, ve $99'ı şirketten alabilirler. Bu persona developer'lardan çok daha iyi dönüşüm yapar.

**SPC + Capability'yi erken bitir:** Bu iki özellik senin Minitab'tan farkını yaratıyor. Developer'lar ttest için Python'dan ayrılmaz. Kalite mühendisi X-bar grafiği için ayrılır.

**Lansmanda kişisel hikayeni kullan:** "Askeri üniversite öğrencisi, 12 ay içinde askeri göreve gidecek, bu sürede Minitab'ın CLI versiyonunu yazdı." — Bu HN'de front page'e gider.

### ❌ Yapma:

**WASM demo yazmak için zaman harcama.** Güzel ama yanlış öncelik. Asciinema kaydı yeterli.

**Open-core yapma.** ttest ücretsiz, SPC ücretli demek → kullanıcı ttest yapar, SPC için Python'a döner. Tüm tool'u $99'a ver.

**Abonelik fiyatlandırması.** Askeri görevde churn yönetemezsin.

**Lansmandan önce landing page'i perfekt yapmaya çalışma.** 1 saatlik Astro sayfası + Lemon Squeezy linki = yeterli.

### 🎯 Öncelik Sırası (Bir Sonraki 6 Ay):

```
1. ANOVA + CI + Chisq → bitir (Faz 2-3)
2. SPC → bitir (Faz 4) ← en kritik diferansiyasyon
3. Capability → bitir (Faz 5)
4. Sertleştirme → yeterince iyi (Faz 6)
5. Beta: 10 gerçek kullanıcı bul
6. v1.0 lansmanı
```

**Sonuç:** $10-15k hedef ulaşılabilir. Gerçekçi senaryo $13k, kötümser $6k — ikisi de "araba parası" için makul. Ama bu rakama ulaşmak için **kalite mühendislerine ulaşman gerekiyor** — developer'lar ücretsiz araç ister, IE mühendisleri Minitab için zaten para ödüyor.

---

*Bu belge Haziran 2026 tarihli pazar koşullarına göre hazırlanmıştır.*  
*Kaynak: Pazar araştırması (SPC software market ~$2.8B, CAGR 8-12%), proje dosyaları analizi, rakip analizi.*
