# ⚛️ QuantumBrowser

> **WebView2 (Chromium) tabanlı, Rust çekirdekli, Arc tarzı ferah arayüze sahip, gizlilik saygılı masaüstü tarayıcı.**
> Sıfır telemetri · Yerleşik reklam/izleyici engelleyici · Çoklu profil · Gizli mod · Dikey sekmeler ve sekme grupları

QuantumBrowser, Microsoft Edge'i güçlendiren motorun (Chromium via WebView2) üzerine **kendi kabuğunu** inşa eder: sekmeler, indirme yöneticisi, reklam engelleme, yer imleri, geçmiş ve Temalar — hepsi Rust (`rusqlite`, IPC) ile yazılmıştır. Sıfırdan render motoru yazmaz; bunun yerine **küçük boyut, hız ve gizlilik** üzerine odaklanır.

---

## ✨ Özellikler (v0.1.0 — Windows MVP)

- **Dikey sekme şeridi** — sürükleyip sabitleme, sekme grupları (renkli, daraltılabilir), pinli sekmeler, orta tuşla kapatma, `Ctrl+Tab` döngüsü
- **Arc tarzı ferah arayüz** — aydınlık "Airy" tema + koyu tema + sistem teması; Lucide ikonları, gerçek düğmeler, emoji yok
- **Reklam & izleyici engelleyici** — ~110 bilinen ağ alan adı engelleme (`doubleclick.net`, `pubmatic.com`, `taboola.com`…) + kozmetik filtreleme; sekme rozetinde kaç öğe engellendiği görünür
- **Downloads yöneticisi** — `Downloads/QuantumBrowser` klasörüne yönlendirme, SQLite'a kayıt, tamamlanınca bildirim
- **Yer imleri & geçmiş** — FTS5 (tam metin) arama, gün bazlı geçmiş gruplama, yer imi yıldızı
- **Gizli mod** — WebView2 non-persistent data store; geçmiş/indirme kayıt edilmez
- **Çoklu profil** — her profil ayrı çerez-deposu (`%LOCALAPPDATA%/com.quantum.browser/profiles/<uuid>`)
- **NewTab sayfası** — canlı saat, günaydın/iyi akşamlar karşılaması, arama kutusu, hız kadranı
- **Klavye kısayolları** — `Ctrl+T` yeni sekme · `Ctrl+W` kapat · `Ctrl+L` adres çubuğu · `Ctrl+Tab` sekme değiştirme · `Ctrl+1..9` sekmeye atla · `Ctrl+Shift+N` gizli sekme · `Ctrl+Shift+Y` geçmiş · `Ctrl+B` yer imleri · `Ctrl+J` indirmeler · `Ctrl+,` ayarlar

## 🧱 Teknoloji

| Katman | Teknoloji |
|---|---|
| Kabuk | **Tauri v2** (`unstable` multiwebview, `windows` hedef) · WebView2 evergreen |
| Çekirdek | **Rust** — `rusqlite` (bundled, FTS5), `chrono`, `uuid` |
| Arayüz | **React 18 + TypeScript + Vite**, Lucide ikonlar |
| Kalıcılık | SQLite — `app_data_dir/quantum.db` (profiller, yer imleri, geçmiş, indirmeler, gruplar, ayarlar; FTS5 indeksli) |

### Mimari

```
┌───────────────────────────── 1 pencere ─────────────────────────────┐
│  React kabuğu (titlebar + dikey şerit + adres çubuğu + NewTab)      │
│  ┌──────────────┬───────────────────────────────────────────────┐   │
│  │ TabWebview 1 │  React panelleri (Geçmiş/Ayarlar…)  ← overlay │   │
│  │ TabWebview 2 │  ← katman: aktif sekme önde, diğerleri gizli  │   │
│  └──────────────┴───────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────┘
  Rust çekirdeği: webview yarat/yok et/yerleşim · navigasyon → URL/başlık senkronu
  → history/downloads (SQLite) · WebResourceRequested ile reklam engelleme
```

Kabuk asla sayfa yüklemez; her `<webview>` ayrı bir çocuk webview'dur ve konumu/boyutu Rust'tan `set_bounds` ile eşitlenir. Sayfalar **Edge ile birebir** render olur.

## 🚀 Geliştirme

**Önkoşullar**
- Windows 10/11 · [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (Windows 11 hazır)
- [Rust](https://rustup.rs/) (≥1.90) + MSVC Build Tools (VS 2026/2022 "Desktop development with C++")
- Node.js ≥ 20
- **Önemli (Windows):** *Smart App Control* (Windows Güvenliği → Uygulama ve tarayıcı denetimi) **kapalı** olmalı ya da derleme başka makinede yapılmalı — `cargo` yeni derlediği build-script `.exe`'lerini çalıştıramaz (`os error 4551`). Kapatıldıktan sonra (Windows sıfırlanmadan) tekrar açılamaz; kararı sana ait.

```powershell
git clone <repo>; cd quantum-browser
npm install
npm run tauri dev        # geliştirme (vite + cargo + uygulama)
npm run tauri build      # release: QuantumBrowser-setup-0.1.0.exe (NSIS)
```

## 🔐 Dürüstlük notu (neler bilerek böyle?)

- Chrome/Edge/Firefox tarzı **uzantı desteği yok** (WebView2 mimarisi gereği) — yerleşik engelleme onun yerine geçer.
- **Google ile oturum açma** bazı sitelerde `disallowed_useragent` dönebilir; OAuth akışları v0.2'de sistem tarayıcısına yönlendirme planlanıyor.
- **DRM** (Netflix/Spotify) — Widevine CDM durumu site bazında değişir; YouTube sorunsuz çalışır.
- **Code signing sertifikan yok** — arkadaşların "Windows protected your PC" uyarısı görür → *More info → Run anyway*. Microsoft sertifikası (~$100–400/yıl) v1.0 hedefi.
- Multiwebview API'si Tauri'de **`unstable`** işaretli — Tauri'yi **minor sürümde sabitliyoruz** (bkz. `Cargo.toml`), API stabilize olunca yükseltiriz.

## 🗺️ Yol haritası

- **v0.2** — WebView2 `AcceleratorKeyPressed` kancası (sayfa odağındayken `Ctrl+T/W/L`), Google OAuth sistem-tarayıcı akışı, profil çoklu penceresi, sekmeleri kaydetme/geri yükleme (oturum)
- **v0.3** — Android (`minSdk 24` → cihaz testine göre 26 olasılığı; `targetSdk 36`), dokunmatik mobil UI, APK dağıtımı (`GitHub Releases`)
- **v1.0** — Code signing (SmartScreen itibarı), otomatik güncelleyici, hata telemetrisi **opt-in**, NSIS kurulumcusu + Microsoft Store değerlendirmesi

## 📁 Yapı

```
src/                React arayüz (kabuk) — App.tsx, components/, pages/NewTab.tsx
src-tauri/          Rust çekirdeği — tabs.rs (sekme motoru), blocker.rs, downloads.rs, db.rs (SQLite/FTS5), commands.rs (IPC)
assets/             Logo kaynakları (SVG + PNG)
```

## 🪪 Lisans

MIT — bkz. [LICENSE](LICENSE). Lucide ikonları ISC, Tauri MIT/Apache-2.0, WebView2 Microsoft şartları.
