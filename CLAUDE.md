# rstat — Proje Bağlamı (CLAUDE.md)

## Ne Bu?
Rust tabanlı, cross-platform, pipeline-friendly istatistik CLI aracı.
"scipy.stats + Minitab'ın komut satırı versiyonu — runtime gerektirmeyen profesyonel stats engine."

## MVP Komutları
```
rstat ttest      → t-test (tek örnek, iki örnek, paired)
rstat anova      → one-way ANOVA
rstat chisq      → chi-square bağımsızlık testi
rstat ci         → güven aralıkları
rstat spc        → X-bar, R kontrol grafikleri
rstat capability → Cp, Cpk proses yeterliliği
```

## Kullanım
```bash
cat data.csv | rstat ttest --col1 before --col2 after --output json
cat process.csv | rstat capability --col thickness --usl 10.5 --lsl 9.5
```

## Teknik Stack
- Dil: Rust
- Parser: `csv` crate (veya polars)
- Hesaplama: `hypors`, `rs-stats`, `statrs`
- Output: terminal tablo / JSON / CSV
- Cross-platform tek binary, runtime yok

## Kısıtlar
- Solo geliştirici, 1 yıl süre (sonra askeri görev)
- Bulut bağımlılığı yok, düşük bakım hedefi
- Hedef gelir: ~$10-15k

## Kurucu
- Endüstri Mühendisliği 3. sınıf (askeri okul)
- Rust bilgisi var, IE + matematik güçlü

## İlk Adım
`rstat ttest` implementasyonuyla başla.

## Dosyalar
- `rstat-proje-baglam.md` — tam proje notu (Obsidian)
