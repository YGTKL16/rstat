# rstat — Proje Bağlamı

## Ürün Özeti
**Rust tabanlı, cross-platform, pipeline-friendly istatistik CLI aracı.**

Tek cümle: *"scipy.stats + Minitab'ın komut satırı versiyonu — Linux dahil her sistemde çalışan, pipeline'a giren, runtime gerektirmeyen profesyonel stats engine."*

---

## Hedef Kullanıcı
- Veri mühendisleri (production istatistik monitoring)
- ML/data science pratisyenleri (A/B test, model validasyon)
- Endüstri mühendisleri (SPC, kalite kontrol)
- Akademik araştırmacılar (tekrarlanabilir CLI iş akışı)
- Linux kullanıcıları (Minitab çalışmıyor, Python kurmak istemiyorlar)

---

## MVP Özellikleri (Bilinen alan — önce bunlar)

```
rstat ttest      → t-test (tek örnek, iki örnek, paired)
rstat anova      → one-way ANOVA
rstat chisq      → chi-square bağımsızlık testi
rstat ci         → güven aralıkları
rstat spc        → X-bar, R kontrol grafikleri
rstat capability → Cp, Cpk proses yeterliliği
```

**Kullanım örneği:**
```bash
cat data.csv | rstat ttest --col1 before --col2 after --output json
cat process.csv | rstat capability --col thickness --usl 10.5 --lsl 9.5
```

---

## Teknik Mimari

```
CSV/TSV/JSON input (stdin veya dosya)
        ↓
[Parser] → polars veya csv crate
        ↓
[Hesaplama] → hypors + rs-stats + statrs (Rust crate'leri)
        ↓
[Output] → terminal tablo / JSON / CSV
```

**Cross-platform:** Tek Rust binary → Linux / macOS / Windows  
**Runtime:** Yok. pip yok, R yok, Java yok.

---

## Sonraki Versiyon (Öğrendikçe eklenecek)
- Bootstrap resampling
- Bayesian alternatifler
- Power analysis / sample size
- Multiple testing correction (Bonferroni, FDR/BH)
- Permutation tests

---

## İş Modeli
- **Fiyat:** $149/yıl bireysel, $499/yıl takım
- **Satış kanalı:** HackerNews, Reddit r/rust r/datascience, GitHub
- **Model:** Self-serve, no negotiation — kalite konuşur

---

## Kısıtlar
- Solo geliştirici
- 1 yıl build süresi (sonra askeri görev)
- Bulut bağımlılığı yok
- Düşük bakım: tarihi data formatları değişmez
- Hedef: araba parası (~$10-15k)

---

## Kurucu Profili
- Endüstri Mühendisliği 3. sınıf (askeri okul)
- Rust bilgisi var
- IE + matematik güçlü
- AI kullanımı iyi
- 1 yıl sonra subay olacak

---

## Pazar Doğrulaması
- xsv (Rust) → Nisan 2025'te arşivlendi
- qsv → tanımlayıcı istatistik var, hipotez testi YOK
- miller, datamash → temel stats, inferential YOK
- Rust'ta `hypors`, `rs-stats` kütüphaneleri var ama CLI yok
- Minitab: Windows only, $1,800+/yıl → Linux boşluğu gerçek

---

## Sonraki Adım
Implementasyon planı yaz → `rstat ttest` ile başla
