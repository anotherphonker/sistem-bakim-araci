# Sistem Bakım Aracı

Windows için **Rust + eframe/egui** ile yazılmış, tek statik `.exe` olarak dağıtılan
bakım ve onarım aracı. Dışa bağımlılık sıfırdır; çizim anlık (immediate mode) yapılır.

## Özellikler

- **Mikro Kod Kontrolü** (ana sayfada öne çıkan kart): Intel 13./14. nesil tespiti.
  SADECE TARAR, HİÇBİR POPUP/ONAY SORMADAN sonucu konsola yazar. Mikrokod eskiyse
  kapatma yapmaz; konsolda **Güç Yönetimi → Turbo Kapat (%99)** yolunu gösterir
  (aktif: Maksimum İşlemci %100→%99 + Processor Performance Boost Mode kapatılır).
  Turbo zaten kapalıysa **Güç Yönetimi → Turbo Aç (%100)** yönlendirmesi verir.
  Windows mikrokodu bildirmediğinde BIOS değeri (Previous Update Revision) okunur.
  Kalıcı çözüm BIOS güncellemesidir.
- **Bilinen Sorunları Uygula**: tarar ve tek onayla düzeltir. Hızlı Başlangıç,
  DLL arama koruması (CWDIllegalInDllSearch), ağ kısıtlaması (NetworkThrottlingIndex),
  GameDVR. Tamamı yerel `reg.exe` / `powercfg` ile; VBS yok, konsol penceresi açılmaz.
- **Temizlik**: Hızlı / Derin / Explorer yeniden başlat / Geri dönüşüm / WU önbelleği /
  Alan Canavarı (en büyük 20 dosya, silmez).
- **Onarım**: Tam Otomatik (DISM zinciri) / SFC / CheckHealth / ScanHealth /
  RestoreHealth / RestoreHealth + ISO / ComponentCleanup / AnalyzeComponentStore.
- **Disk**: chkdsk /scan, birim ve fiziksel disk bilgisi, bileşen deposu analizi.
- **Güç Yönetimi**: güç planları tek tıkla — Güç Tasarrufu, Dengeli, Yüksek Performans
  ve Chris Titus tarzı Ultimate Performance (gizli plan `powercfg -duplicatescheme`
  ile oluşturulur ve dönen GERÇEK plan GUID'i aktifleştirilir — şablon GUID'e
  setactive yapılmaz, o "unsupported setting" hatası verir). Ayrıca Turbo Kapat
  (Maksimum İşlemci 100 yerine 99 +
  boost mode kapatılır) ve Turbo Aç (geri 100 + boost Aggressive). Hepsi `powercfg`
  ile, konsol penceresi açılmadan uygulanır; powercfg yanıt vermezse değer doğrudan
  kayıt defterine (AC/DC SettingIndex) yazılır.
- **Bilgi ve Log**: sistem özeti, CPU / mikro kod durumu, logları masaüstüne kopyalama.
- **Türkçe / İngilizce**: sol panelin en altındaki bayrak butonu ile anında dil değişimi.
- Koyu / açık mod ve 11 rollü renk özelleştirme
  (`%APPDATA%\SistemBakimAraci\config.json`).
- Liste görünümü; konsol alt/sağ konum, F12 ile aç/kapat, sürükleyerek yeniden boyut.
- Çalışan işlemi durdurma, tek örnek koruması (mutex), yönetici yetkisi (UAC + runas).
- Gerçek renkli emojiler exe'nin içine gömülüdür (Twemoji 72x72 PNG); görünüm
  Windows sürümünden bağımsızdır.
- Araç çalıştırınca PowerShell/cmd penceresi açılmaz (`CREATE_NO_WINDOW` bayrağı).

## Geliştirici

- **Geliştirici:** Resul Çelik
- **GitHub:** [github.com/anotherphonker](https://github.com/anotherphonker)

## Klasörler

```
sistem-bakim-araci/
|- src/                 kod (main, app, actions, console, config, theme,
|                       sections, winutil, emoji, i18n)
|- assets/              icon.ico + emoji/*.png (exe'ye gömülür)
|- build.bat            Windows'ta tek tıkla .exe derleyici
|- install_mingw.bat    Tek seferlik mingw-w64 kurulumu
`- Cikti/               SistemBakimAraci.exe (derleme sonrası)
```

## Windows'ta derlemek

1. **Rust kur** (tek seferlik): https://rustup.rs adresinden rustup-init indir ve çalıştır.
   "Visual C++ prerequisites" sorusuna 3 (Don't install) seç.
2. **MinGW kur** (tek seferlik): `install_mingw.bat` çalıştır (winget ile kurar) ya da
   https://winlibs.com adresinden "x86_64-posix-seh" sürümünü indir, `mingw64\bin`
   klasörünü PATH'e ekle.
3. **build.bat** çalıştır. Çıktı: `Cikti\SistemBakimAraci.exe`
   (Visual Studio gerekmez; GNU araç zinciri otomatik ayarlanır).

## Teknik notlar

- **eframe/egui 0.31** — tek statik EXE, glow (OpenGL) renderer.
- Windows 10/11 ayrımı build numarasına göre yapılır (>= 22000 = Windows 11);
  böylece IoT Enterprise LTSC 2024 gibi, kayıt defterinde yanlış "Windows 10"
  yazan SKU'lar winver ile aynı şekilde "Windows 11 IoT Enterprise LTSC" gösterilir.
- Kayıt defteri REG_SZ değerleri null-sonlandırıcıdan arındırılarak okunur; böylece
  aktif güç planı GUID'i bozulmaz ve powercfg komutları doğru parametreyle çalışır.
- Güç komutları başarısız olursa sebep konsola yazılır ve elle çalıştırılacak
  powercfg komutları (SCHEME_CURRENT / PROCTHROTTLEMAX) önerilir.
- Yönetici logosu ve otomatik UAC: `requireAdministrator` manifest'i `.rc`'den
  windres ile exe'ye gömülür. GNU zincirinin otomatik eklediği `default-manifest.o`
  (asInvoker) spec dosyasıyla devre dışı bırakılır; böylece tek manifest kalır ve
  `multiple non-default manifests` uyarısı çıkmaz. İkon da ayrı bir `.rc` ile gömülür.
- Tek örnek: `Global\SistemBakimAraci_v4_8b2e11` mutex; ikinci açılışta mevcut
  pencere öne alınır.
- Çıktılar worker iş parçacığından `mpsc` kanalıyla UI'a akar; arayüz kilitlenmez.
- Çıktı kodlaması: UTF-8, CP1254, sonra lossy (Türkçe ğ/ş/ı garanti).
- CPU / mikro kod okuması kayıt defterinden (FFI) yapılır; harici araç çalıştırmaz.

---

## ENGLISH

**SYSTEM MAINTENANCE TOOL**

A Windows maintenance and repair tool written in **Rust + eframe/egui**, shipped as a
single static `.exe`. Zero external runtime dependencies, immediate-mode drawing.

### Features

- **Microcode Check** (featured card on the home screen): detects Intel 13/14th gen.
  It only SCANS and prints the result to the console — no popup, no confirmation
  dialog, it changes nothing. If the microcode is old it points to **Power →
  Turbo Off (99%)**; if turbo is already off it points to **Power → Turbo On
  (100%)**. Reads the BIOS value (Previous Update Revision) when Windows doesn't
  report the microcode. Permanent fix is a BIOS update.
- **Apply Known Fixes**: scans and fixes with a single confirmation. Fast Startup,
  DLL search protection (CWDIllegalInDllSearch), network throttling
  (NetworkThrottlingIndex), GameDVR. All via local `reg.exe` / `powercfg`;
  no VBS, no console window opens.
- **Cleanup**: Quick / Deep / restart Explorer / empty recycle bin / clear WU cache /
  Space Hog Finder (20 largest files, does not delete).
- **Repair**: Full Auto (DISM chain) / SFC / CheckHealth / ScanHealth / RestoreHealth /
  RestoreHealth + ISO / ComponentCleanup / AnalyzeComponentStore.
- **Disk**: chkdsk /scan, volume and physical disk info, component store analysis.
- **Power Management**: power plans in one click — Power Saver, Balanced, High
  Performance and the Chris Titus-style Ultimate Performance (the hidden plan is
  created via `powercfg -duplicatescheme` and the REAL plan GUID it returns is
  activated — never setactive the template GUID, which errors with "unsupported
  setting"). Also Turbo Off (Maximum Processor
  State 100 -> 99 plus boost mode off) and Turbo On (back to 100 + boost
  Aggressive). All applied with `powercfg`, no console window; if powercfg won't
  respond the value is written straight to the registry (AC/DC SettingIndex).
- **Info & Logs**: system summary, CPU / microcode status, copy logs to desktop.
- **Turkish / English**: instant language switch via the flag button at the very
  bottom of the left panel.
- Dark / light mode and 11-role color customization
  (`%APPDATA%\SistemBakimAraci\config.json`).
- List view; console bottom/right, F12 toggle, drag to resize.
- Stop a running job, single-instance guard (mutex), administrator rights (UAC + runas).
- Real color emojis embedded in the exe (Twemoji 72x72 PNG); rendering is independent
  of the Windows version.
- No PowerShell/cmd window opens when a tool runs (`CREATE_NO_WINDOW` flag).

### Developer

- **Developer:** Resul Çelik
- **GitHub:** [github.com/anotherphonker](https://github.com/anotherphonker)

### Building on Windows

1. **Install Rust** (once): download and run rustup-init from https://rustup.rs.
   Choose 3 (Don't install) for the "Visual C++ prerequisites" question.
2. **Install MinGW** (once): run `install_mingw.bat` (installs via winget), or download
   the "x86_64-posix-seh" build from https://winlibs.com and add `mingw64\bin` to PATH.
3. Run **build.bat**. Output: `Cikti\SistemBakimAraci.exe`
   (Visual Studio is not required; the GNU toolchain is set up automatically).

Windows 10 vs 11 is decided by the build number (>= 22000 means Windows 11), so
SKUs that mislabel themselves in the registry — like IoT Enterprise LTSC 2024,
which reports "Windows 10 IoT Enterprise LTSC" — are shown as
"Windows 11 IoT Enterprise LTSC", the same way winver and Settings do.
Registry REG_SZ values are read trimmed of the trailing NUL, so the active power
scheme GUID stays intact and powercfg commands receive a valid parameter. If a
power command still fails, the reason is printed to the console along with manual
powercfg commands (SCHEME_CURRENT / PROCTHROTTLEMAX).

### License

Emoji artwork (Twemoji) is CC-BY 4.0. The application source is provided under the
LICENSE file in this repository.
