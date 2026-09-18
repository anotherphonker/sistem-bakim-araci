//! Araç gövdesi: temizlik / onarım / disk / bilgi komutlarının gerçek çalıştırılması.
//! Tüm uzun işlemler worker ipliğinde çalışır; çıktılar `Con` üzerinden UI'a akar.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::i18n::{pick, tr};

#[cfg(windows)]
pub fn set_no_window_flags(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    // Yalnızca CREATE_NO_WINDOW kullan: konsol penceresi açmaz, çıktı boruyla okunur.
    // ÖNEMLİ: CREATE_NEW_CONSOLE + DETACHED_PROCESS birlikte kullanılamaz;
    // ikisi de verilirse CreateProcess "error 87 - Parametre hatalı" ile başarısız olur
    // ve chkdsk/DISM/SFC/powercfg/reg hiçbiri çalışmaz.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
pub fn set_no_window_flags(_cmd: &mut Command) {}

use crate::console::Con;

// ---------------------------------------------------------------------------
// yardımcılar
// ---------------------------------------------------------------------------

pub fn is_progress(s: &str) -> bool {
    let s = s.trim();
    s.len() < 80 && s.starts_with('[') && s.ends_with(']') && s.contains('%')
}

fn env(var: &str, fallback: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| fallback.to_string())
}

fn windir() -> String {
    env("WINDIR", "C:\\Windows")
}

fn local() -> String {
    env("LOCALAPPDATA", "")
}

/// Bir komutu çalıştırır, çıktıyı satır satır konsola akıtır.
pub fn run_forward(bus: &Con, stop: &AtomicBool, cmd: &str, args: &[&str]) {
    let mut command = Command::new(cmd);
    command.args(args).stdin(Stdio::null()).stdout(Stdio::piped());
    set_no_window_flags(&mut command);
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            bus.err(pick(
                format!("({cmd}) başlatılamadı: {e}"),
                format!("({cmd}) failed to start: {e}"),
            ));
            return;
        }
    };

    if let Some(out) = child.stdout.take() {
        let mut reader = BufReader::new(out);
        let mut buf: Vec<u8> = Vec::new();
        loop {
            if stop.load(Ordering::Relaxed) {
                let _ = child.kill();
                bus.warn(tr("İşlem durduruldu.", "Process stopped."));
                break;
            }
            buf.clear();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) => break,
                Ok(_) => {
                    let line = decode(&buf)
                        .trim_end_matches(|c: char| c == '\r' || c == '\n')
                        .to_string();
                    if !line.is_empty() {
                        if is_progress(&line) {
                            bus.progress(line);
                        } else {
                            bus.plain(line);
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }
    match child.wait() {
        Ok(s) => {
            if !s.success() {
                let code = s.code().unwrap_or(-1);
                bus.warn(pick(format!("Çıkış kodu: {code}"), format!("Exit code: {code}")));
            }
        }
        Err(e) => bus.err(pick(format!("Bekleme hatası: {e}"), format!("Wait error: {e}"))),
    }
}

/// Çıktıyı sessizce yakalar (mount gibi tek değer döndüren PowerShell için).
pub fn capture(cmd: &str, args: &[&str]) -> String {
    let mut command = Command::new(cmd);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    set_no_window_flags(&mut command);
    match command.output() {
        Ok(o) => decode(&o.stdout),
        Err(_) => String::new(),
    }
}

/// UTF-8 → (değilse) CP1254 → (değilse) lossy çöz.
pub fn decode(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }
    // CP1254 (Türkçe ANSI) denemesi
    let mapped: String = bytes
        .iter()
        .map(|&b| cp1254(b))
        .collect();
    // lossy'den daha kötüyse lossy kullan
    let lossy = String::from_utf8_lossy(bytes);
    if mapped.contains('\u{FFFD}') && !lossy.contains('\u{FFFD}') {
        lossy.into_owned()
    } else {
        mapped
    }
}

fn cp1254(b: u8) -> char {
    if b < 0x80 {
        return b as char;
    }
    const HIGH: [char; 128] = [
        '€', '\u{FFFD}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}',
        '\u{FFFD}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{FFFD}', '\u{FFFD}', 'Ÿ',
        '\u{00A0}', '¡', '¢', '£', '¤', '¥', '¦', '§', '¨', '©', 'ª', '«', '¬', '\u{00AD}', '®', '¯',
        '°', '±', '²', '³', '´', 'µ', '¶', '·', '¸', '¹', 'º', '»', '¼', '½', '¾', '¿',
        'À', 'Á', 'Â', 'Ã', 'Ä', 'Å', 'Æ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï',
        'Ğ', 'Ñ', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', '×', 'Ø', 'Ù', 'Ú', 'Û', 'Ü', 'İ', 'Ş', 'ß',
        'à', 'á', 'â', 'ã', 'ä', 'å', 'æ', 'ç', 'è', 'é', 'ê', 'ë', 'ì', 'í', 'î', 'ï',
        'ğ', 'ñ', 'ò', 'ó', 'ô', 'õ', 'ö', '÷', 'ø', 'ù', 'ú', 'û', 'ü', 'ı', 'ş', 'ÿ',
    ];
    HIGH[(b - 0x80) as usize]
}

fn rmtree_contents(path: &Path, bus: &Con, show: bool) {
    if !path.is_dir() {
        return;
    }
    let mut n = 0usize;
    if let Ok(rd) = std::fs::read_dir(path) {
        for e in rd.flatten() {
            let p = e.path();
            let r = if p.is_dir() {
                std::fs::remove_dir_all(&p)
            } else {
                std::fs::remove_file(&p)
            };
            if r.is_ok() {
                n += 1;
            }
        }
    }
    if show {
        bus.dim(format!(
            "   - {}  ({n} {})",
            path.display(),
            tr("öğe", "items")
        ));
    }
}

fn glob_del(folder: &str, needle: &str) {
    if let Ok(rd) = std::fs::read_dir(folder) {
        for e in rd.flatten() {
            let fname = e.file_name().to_string_lossy().to_string();
            if fname.to_lowercase().contains(&needle.to_lowercase()) {
                let p = e.path();
                let _ = if p.is_dir() {
                    std::fs::remove_dir_all(&p)
                } else {
                    std::fs::remove_file(&p)
                };
            }
        }
    }
}

fn stop_service(name: &str) {
    let mut c = Command::new("net");
    c.args(["stop", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    set_no_window_flags(&mut c);
    let _ = c.status();
}

fn start_service(name: &str) {
    let mut c = Command::new("net");
    c.args(["start", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    set_no_window_flags(&mut c);
    let _ = c.status();
}

fn empty_recycle() {
    let mut c = Command::new("powershell");
    c.args([
        "-NoProfile",
        "-Command",
        "Clear-RecycleBin -Force -ErrorAction SilentlyContinue",
    ])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null());
    set_no_window_flags(&mut c);
    let _ = c.status();
}

/// Kullanıcı onayı. Windows'ta rfd iletişim kutusu; dev ortamında otomatik evet.
#[cfg(windows)]
fn confirm(title: &str, msg: &str) -> bool {
    rfd::MessageDialog::new()
        .set_title(title)
        .set_description(msg)
        .set_buttons(rfd::MessageButtons::YesNo)
        .set_level(rfd::MessageLevel::Warning)
        .show()
        == rfd::MessageDialogResult::Yes
}

#[cfg(not(windows))]
fn confirm(_title: &str, _msg: &str) -> bool {
    true
}

// ---------------------------------------------------------------------------
// temizlik
// ---------------------------------------------------------------------------

pub fn quick_clean(bus: &Con, _stop: &AtomicBool) {
    bus.header(tr("HIZLI TEMİZLİK", "QUICK CLEAN"));
    bus.dim(tr(
        "temp + önbellek + geri dönüşüm (kişisel dosyalara dokunmaz)",
        "temp + cache + recycle bin (never touches personal files)",
    ));

    let t = std::env::temp_dir();
    rmtree_contents(&t, bus, true);
    rmtree_contents(Path::new(&(windir() + "\\Temp")), bus, true);
    let loc = local();
    glob_del(
        &format!("{loc}\\Microsoft\\Windows\\Explorer"),
        "thumbcache_",
    );
    rmtree_contents(
        Path::new(&format!("{loc}\\Microsoft\\Windows\\INetCache")),
        bus,
        true,
    );
    empty_recycle();
    bus.ok(tr("Hızlı temizlik tamamlandı.", "Quick clean finished."));
}

pub fn deep_clean(bus: &Con, _stop: &AtomicBool) {
    if !confirm(
        tr("Derin Temizlik", "Deep Clean"),
        tr(
            "Windows Update önbelleği, prefetch ve cache dosyaları silinecek.\nKişisel dosyalarına dokunulmaz.\n\nDevam edilsin mi?",
            "Windows Update cache, prefetch and cache files will be deleted.\nYour personal files will not be touched.\n\nContinue?",
        ),
    ) {
        bus.warn(tr("İptal edildi.", "Cancelled."));
        return;
    }

    bus.header(tr("DERİN TEMİZLİK", "DEEP CLEAN"));
    let t = std::env::temp_dir();
    rmtree_contents(&t, bus, true);
    rmtree_contents(Path::new(&(windir() + "\\Temp")), bus, true);

    bus.dim(tr(
        "   - Windows Update önbelleği...",
        "   - Windows Update cache...",
    ));
    stop_service("wuauserv");
    stop_service("bits");
    rmtree_contents(
        Path::new(&(windir() + "\\SoftwareDistribution\\Download")),
        bus,
        false,
    );
    start_service("bits");
    start_service("wuauserv");

    glob_del(&(windir() + "\\Prefetch"), ".pf");
    bus.dim(tr(
        "   - Delivery Optimization önbelleği...",
        "   - Delivery Optimization cache...",
    ));
    rmtree_contents(
        Path::new(
            &(windir()
                + "\\ServiceProfiles\\NetworkService\\AppData\\Local\\Microsoft\\Windows\\DeliveryOptimization\\Cache"),
        ),
        bus,
        false,
    );

    let loc = local();
    let _ = std::fs::remove_file(format!("{loc}\\IconCache.db"));
    glob_del(
        &format!("{loc}\\Microsoft\\Windows\\Explorer"),
        "iconcache_",
    );

    let md = format!("{}\\Minidump", windir());
    let mdp = Path::new(&md);
    if mdp.is_dir() && std::fs::read_dir(mdp).map(|mut r| r.next().is_some()).unwrap_or(false) {
        if confirm(
            tr("Minidump", "Minidump"),
            tr(
                "C:\\Windows\\Minidump içinde mavi ekran kayıtları var.\nBunlar arıza TESPİTİ için delildir.\n\nSilinsin mi?",
                "C:\\Windows\\Minidump contains blue-screen records.\nThey are EVIDENCE for troubleshooting.\n\nDelete them?",
            ),
        ) {
            rmtree_contents(mdp, bus, false);
            let _ = std::fs::remove_file(format!("{}\\MEMORY.DMP", windir()));
            bus.warn(tr("   - Minidump silindi.", "   - Minidump deleted."));
        } else {
            bus.dim(tr("   - Minidump korundu (delil).", "   - Minidump kept (evidence)."));
        }
    }

    if confirm(
        tr("Tarayıcı Önbelleği", "Browser Cache"),
        tr(
            "Tarayıcı önbellekleri de temizlensin mi?\n(Tarayıcılar kapalı olmalı.)",
            "Clear browser caches too?\n(Browsers should be closed.)",
        ),
    ) {
        let loc = local();
        for rel in [
            r"Microsoft\Edge\User Data\Default\Cache",
            r"Google\Chrome\User Data\Default\Cache",
            r"BraveSoftware\Brave-Browser\User Data\Default\Cache",
            r"Opera Software\Opera Stable\Cache",
        ] {
            let p = format!("{loc}\\{rel}");
            rmtree_contents(Path::new(&p), bus, false);
        }
        bus.dim(tr(
            "   - Tarayıcı önbellekleri silindi.",
            "   - Browser caches cleared.",
        ));
    }
    bus.ok(tr("Derin temizlik tamamlandı.", "Deep clean finished."));
}

pub fn restart_explorer(bus: &Con, _stop: &AtomicBool) {
    bus.info(tr("Explorer yeniden başlatılıyor...", "Restarting Explorer..."));
    let mut c = Command::new("taskkill");
    c.args(["/f", "/im", "explorer.exe"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    set_no_window_flags(&mut c);
    let _ = c.status();
    std::thread::sleep(std::time::Duration::from_secs(2));
    let mut e = Command::new("explorer.exe");
    set_no_window_flags(&mut e);
    let _ = e.spawn();
    bus.ok(tr("Masaüstü ve görev çubuğu yenilendi.", "Desktop and taskbar refreshed."));
}

pub fn empty_recycle_dialog(bus: &Con, _stop: &AtomicBool) {
    if !confirm(
        tr("Geri Dönüşüm", "Recycle Bin"),
        tr("Geri dönüşüm kutusu boşaltılsın mı?\nBu işlem geri alınamaz.", "Empty the recycle bin?\nThis cannot be undone."),
    ) {
        bus.warn(tr("İptal edildi.", "Cancelled."));
        return;
    }
    empty_recycle();
    bus.ok(tr("Geri dönüşüm kutusu boşaltıldı.", "Recycle bin emptied."));
}

pub fn wu_cache(bus: &Con, _stop: &AtomicBool) {
    bus.info(tr("Windows Update önbelleği temizleniyor...", "Clearing Windows Update cache..."));
    stop_service("wuauserv");
    stop_service("bits");
    rmtree_contents(
        Path::new(&(windir() + "\\SoftwareDistribution\\Download")),
        bus,
        false,
    );
    start_service("bits");
    start_service("wuauserv");
    bus.ok(tr("Windows Update önbelleği temizlendi.", "Windows Update cache cleared."));
}

pub fn big_files(bus: &Con, stop: &AtomicBool) {
    bus.header(tr("ALAN CANAVARI - en büyük 20 dosya", "SPACE HOG FINDER - 20 largest files"));
    let ps = "Get-ChildItem -Path 'C:\\' -Recurse -File -Force -ErrorAction SilentlyContinue | \
              Sort-Object Length -Descending | Select-Object -First 20 \
              @{n='GB';e={[math]::Round($_.Length/1GB,2)}},FullName | Format-Table -AutoSize | \
              Out-String -Width 200";
    run_forward(bus, stop, "powershell", &["-NoProfile", "-Command", ps]);
    bus.dim(tr("(liste bitti - hiçbir dosya silinmedi)", "(list end - no files deleted)"));
}

// ---------------------------------------------------------------------------
// onarım
// ---------------------------------------------------------------------------

pub fn full_repair(bus: &Con, stop: &AtomicBool) {
    if !confirm(
        tr("Tam Otomatik Onarım", "Full Auto Repair"),
        tr(
            "Şu zincir çalışacak:\n\n  1) CheckHealth\n  2) ScanHealth\n  3) RestoreHealth\n  \
             4) SFC /scannow\n  5) ComponentCleanup\n  6) Kontrol\n\nİnternet gerekir, 20-90 dk.\n\
             RestoreHealth %62-63'te uzun süre 'duruyor' görünür, bu normaldir.\n\nBaşla?",
            "This chain will run:\n\n  1) CheckHealth\n  2) ScanHealth\n  3) RestoreHealth\n  \
             4) SFC /scannow\n  5) ComponentCleanup\n  6) Check\n\nInternet required, 20-90 min.\n\
             RestoreHealth may seem 'stuck' at 62-63% - that is normal.\n\nStart?",
        ),
    ) {
        bus.warn(tr("İptal edildi.", "Cancelled."));
        return;
    }

    let steps: Vec<(&str, &str, &str, Vec<&str>)> = vec![
        ("[1/6] CheckHealth", "[1/6] CheckHealth", "dism", vec!["/online", "/cleanup-image", "/checkhealth"]),
        ("[2/6] ScanHealth", "[2/6] ScanHealth", "dism", vec!["/online", "/cleanup-image", "/scanhealth"]),
        ("[3/6] RestoreHealth", "[3/6] RestoreHealth", "dism", vec!["/online", "/cleanup-image", "/restorehealth"]),
        ("[4/6] SFC /scannow", "[4/6] SFC /scannow", "sfc", vec!["/scannow"]),
        ("[5/6] ComponentCleanup", "[5/6] ComponentCleanup", "dism", vec!["/online", "/cleanup-image", "/startcomponentcleanup"]),
        ("[6/6] Kontrol", "[6/6] Check", "dism", vec!["/online", "/cleanup-image", "/checkhealth"]),
    ];

    for (label_tr, label_en, exe, args) in steps {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        bus.big("-".repeat(52));
        bus.info(format!("> {}", tr(label_tr, label_en)));
        bus.dim("-".repeat(52));
        run_forward(bus, stop, exe, &args);
    }
    bus.big("=".repeat(52));
    bus.ok(tr("Tüm işlemler tamamlandı.", "All steps complete."));
    bus.dim("=".repeat(52));
}

pub fn simple_dism(bus: &Con, stop: &AtomicBool, label: &str, flag: &'static str) {
    bus.header(label);
    run_forward(bus, stop, "dism", &["/online", "/cleanup-image", flag]);
    bus.ok(tr("Tamamlandı.", "Done."));
}

pub fn restore_health(bus: &Con, stop: &AtomicBool) {
    if !confirm(
        tr("RestoreHealth", "RestoreHealth"),
        tr(
            "Bozuk sistem bileşenleri internetten indirilip onarılacak.\n\
             15-60 dakika sürebilir; %62-63'te durması normaldir.\n\nBaşla?",
            "Corrupt system components will be repaired via internet.\n\
             May take 15-60 minutes; stalling at 62-63% is normal.\n\nStart?",
        ),
    ) {
        bus.warn(tr("İptal edildi.", "Cancelled."));
        return;
    }
    bus.header("RestoreHealth");
    run_forward(bus, stop, "dism", &["/online", "/cleanup-image", "/restorehealth"]);
    bus.ok(tr("Tamamlandı.", "Done."));
}

pub fn restore_iso(bus: &Con, stop: &AtomicBool, iso: &str) {
    bus.header("RestoreHealth + ISO");
    bus.dim(format!("ISO: {iso}"));

    let drv = mount_iso(iso);
    if drv.is_empty() {
        bus.err(tr("ISO mount edilemedi.", "Could not mount ISO."));
        return;
    }
    let wim = format!("{drv}:\\sources\\install.wim");
    let esd = format!("{drv}:\\sources\\install.esd");
    if Path::new(&wim).is_file() {
        run_forward(
            bus,
            stop,
            "dism",
            &[
                "/online",
                "/cleanup-image",
                "/restorehealth",
                &format!("/source:{drv}:\\sources\\install.wim"),
                "/limitaccess",
            ],
        );
    } else if Path::new(&esd).is_file() {
        run_forward(
            bus,
            stop,
            "dism",
            &[
                "/online",
                "/cleanup-image",
                "/restorehealth",
                &format!("/source:{drv}:\\sources\\install.esd"),
                "/limitaccess",
            ],
        );
    } else {
        bus.err(tr("ISO içinde install.wim / install.esd bulunamadı.", "install.wim / install.esd not found in ISO."));
    }
    dismount_iso(iso);
    bus.ok(tr("ISO onarımı tamamlandı.", "ISO repair finished."));
}

pub fn mount_iso(iso: &str) -> String {
    let ps = format!(
        "try {{ (Mount-DiskImage -ImagePath '{}' -PassThru | Get-Volume).DriveLetter }} catch {{ '' }}",
        iso.replace('\'', "''")
    );
    let out = capture("powershell", &["-NoProfile", "-Command", &ps]);
    out.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .last()
        .unwrap_or("")
        .to_string()
        .trim_end_matches(':')
        .to_string()
}

pub fn dismount_iso(iso: &str) {
    let ps = format!(
        "Dismount-DiskImage -ImagePath '{}' -ErrorAction SilentlyContinue",
        iso.replace('\'', "''")
    );
    let _ = capture("powershell", &["-NoProfile", "-Command", &ps]);
}

pub fn chkdsk(bus: &Con, stop: &AtomicBool) {
    bus.header("chkdsk C: /scan");
    run_forward(bus, stop, "chkdsk", &["C:", "/scan"]);
    bus.ok(tr("Tamamlandı.", "Done."));
}

// ---------------------------------------------------------------------------
// disk / bilgi
// ---------------------------------------------------------------------------

pub fn disk_info(bus: &Con, stop: &AtomicBool) {
    let vol = "Get-Volume -DriveLetter C | Format-List DriveLetter, FileSystemLabel, \
               FileSystem, HealthStatus, SizeRemaining, Size | Out-String -Width 200";
    let dsk = "Get-Disk | Format-Table Number, FriendlyName, BusType, MediaType, \
               HealthStatus, OperationalStatus, PartitionStyle, \
               @{n='SizeGB';e={[math]::Round($_.Size/1GB,1)}} -AutoSize | Out-String -Width 200";
    bus.header(tr("Birim bilgisi (C:)", "Volume info (C:)"));
    run_forward(bus, stop, "powershell", &["-NoProfile", "-Command", vol]);
    bus.header(tr("Fiziksel disk bilgisi", "Physical disk info"));
    run_forward(bus, stop, "powershell", &["-NoProfile", "-Command", dsk]);
    bus.ok(tr("Tamamlandı.", "Done."));
}

pub fn copy_logs(bus: &Con, _stop: &AtomicBool) {
    let windir = windir();
    let desktop = env("USERPROFILE", "C:\\Users\\Public").to_string() + "\\Desktop";
    let dst = PathBuf::from(format!("{desktop}\\bakim_loglari"));
    let _ = std::fs::create_dir_all(&dst);
    let mut copied: Vec<String> = Vec::new();
    for name in ["CBS.log", "DISM.log", "dism.log"] {
        for base in [
            format!("{windir}\\Logs\\CBS\\{name}"),
            format!("{windir}\\Logs\\DISM\\{name}"),
        ] {
            let p = Path::new(&base);
            if p.is_file() {
                if let Some(fname) = p.file_name().and_then(|f| f.to_str()) {
                    if std::fs::copy(p, dst.join(fname)).is_ok() {
                        copied.push(fname.to_string());
                    }
                }
            }
        }
    }
    if copied.is_empty() {
        bus.warn(tr("Log bulunamadı veya erişim engellendi.", "No logs found or access denied."));
    } else {
        bus.ok(pick(
            format!("Kopyalandı: {}", copied.join(", ")),
            format!("Copied: {}", copied.join(", ")),
        ));
        bus.plain(pick(
            format!("  Klasör: {}", dst.display()),
            format!("  Folder: {}", dst.display()),
        ));
    }
}

// ---------------------------------------------------------------------------
// sistem bilgi metni
// ---------------------------------------------------------------------------

pub fn os_reg_value(name: &str) -> String {
    use crate::winutil::reg_get_string;
    let base = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";
    reg_get_string(base, name).unwrap_or_default()
}

pub fn os_build_string() -> String {
    let build = os_reg_value("CurrentBuild");
    // UBR, REG_DWORD'dur. Bunu metin olarak okumak (reg_get_string) 4 baytı
    // UTF-16 gibi yorumlar ve sayının yanında "kare" (tofu) karakteri üretir.
    // Bu yüzden u32 olarak okunur.
    let ubr = crate::winutil::reg_get_dword(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        "UBR",
    );
    if build.is_empty() {
        return String::new();
    }
    match ubr {
        Some(v) => format!("{build}.{v}"),
        None => build,
    }
}

pub fn os_string() -> String {
    let mut prod = os_reg_value("ProductName");
    let disp = os_reg_value("DisplayVersion");
    let build = os_build_string();

    // Windows 10 / 11 ayrımı ProductName'e DEĞİL build numarasına göre yapılır.
    // Bazı SKU'larda (ör. IoT Enterprise LTSC 2024) kayıt defteri yanlışlıkla
    // "Windows 10 ..." yazar ama build 26100 => aslında Windows 11 (24H2) kod
    // tabanıdır. winver ve Ayarlar bunu "Windows 11 IoT Enterprise LTSC" diye
    // gösterir; aynı eşiği kullanıyoruz (>= 22000 = Windows 11).
    if let Ok(b) = build.split('.').next().unwrap_or("0").parse::<u32>() {
        if b >= 22000 && prod.starts_with("Windows 10") {
            prod = prod.replacen("Windows 10", "Windows 11", 1);
        } else if b < 22000 && prod.starts_with("Windows 11") {
            prod = prod.replacen("Windows 11", "Windows 10", 1);
        }
    }

    let mut parts = Vec::new();
    if !prod.is_empty() {
        parts.push(prod);
    }
    if !disp.is_empty() {
        parts.push(disp);
    }
    if !build.is_empty() {
        parts.push(format!("build {build}"));
    }
    if parts.is_empty() {
        "bilinmiyor".to_string()
    } else {
        parts.join(" ")
    }
}

pub fn free_gb() -> Option<f64> {
    crate::winutil::free_bytes("C:\\").map(|b| b as f64 / (1024.0 * 1024.0 * 1024.0))
}

// ---------------------------------------------------------------------------
// CPU / mikrokod (Intel 13/14. nesil yardımı)
// Akış: KONTROL ET -> bulguları yaz -> Güç Yönetimi'ne yönlendir (POPUP YOK).
// Yalnızca Windows'un kendi araçları (powercfg / reg.exe) kullanılır; VBS yok.
// ---------------------------------------------------------------------------

pub mod cpu {
    use crate::console::Con;
    use crate::winutil::reg_get_string;

    use super::{pick, tr};

    const KEY: &str = r"HARDWARE\DESCRIPTION\System\CentralProcessor\0";
    const GUID_SUB_PROCESSOR: &str = "54533251-82be-4824-96c1-47b60b740d00";
    /// Maksimum İşlemci Durumu (PROCTHROTTLEMAX, %0-100). %100 = turbo açık,
    /// %99 ve altı = turbo kapalı (CPU taban hızında çalışır).
    const GUID_PROCTHROTTLEMAX: &str = "bc5038f7-23e0-4960-96da-33abaf5935ec";

    fn value(name: &str) -> String {
        reg_get_string(KEY, name).unwrap_or_default()
    }

    pub fn identifier() -> String {
        value("ProcessorNameString")
    }

    pub fn vendor() -> String {
        value("VendorIdentifier")
    }

    pub fn signature() -> String {
        value("Identifier")
    }

    /// Kayıt defterindeki ham mikro kod değeri ("Update Revision", 8 bayt).
    pub fn microcode_raw() -> Option<u64> {
        crate::winutil::reg_get_bytes(KEY, "Update Revision").map(|b| {
            let mut v: u64 = 0;
            for (i, x) in b.iter().enumerate() {
                if i >= 8 {
                    break;
                }
                v |= (*x as u64) << (8 * i);
            }
            v
        })
    }

    /// BIOS'un başlangıçta yüklediği mikro kod ("Previous Update Revision").
    /// Windows "Update Revision"ı bildirmediğinde gerçek değer genelde buradadır.
    pub fn microcode_prev() -> Option<u64> {
        crate::winutil::reg_get_bytes(KEY, "Previous Update Revision").map(|b| {
            let mut v: u64 = 0;
            for (i, x) in b.iter().enumerate() {
                if i >= 8 {
                    break;
                }
                v |= (*x as u64) << (8 * i);
            }
            v
        })
    }

    /// Mikro kod revizyonu (u32). Hiç yüklenmemişse Windows "0xFFFFFFFF" verir
    /// ve bu bir revizyon DEĞİLDİR; o durumda None döner.
    pub fn microcode_low(raw: u64) -> Option<u32> {
        let v = if (raw >> 32) & 0xffff_ffff == 0xffff_ffff {
            raw & 0xffff_ffff
        } else {
            raw
        };
        if v == 0xffff_ffff || v == 0 {
            None
        } else {
            Some(v as u32)
        }
    }

    pub fn is_gen13_14(name: &str) -> bool {
        let nums: Vec<u64> = name
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|p| p.parse::<u64>().ok())
            .collect();
        nums.iter().any(|&n| (13000..15000).contains(&n))
    }

    fn active_scheme() -> String {
        // powercfg çıktısı yerine registry'den oku (dilden bağımsız, güvenilir).
        crate::winutil::reg_get_string(
            r"SYSTEM\CurrentControlSet\Control\Power\User\PowerSchemes",
            "ActivePowerScheme",
        )
        .unwrap_or_default()
    }

    /// Aktif şemada bu GUID çifti için AC ayar indeksi (REG_DWORD).
    /// None = aktif şema bulunamadı/okunamadı; Some(None) = ayar tanımsız (varsayılan=100).
    fn procthrottle_index() -> Option<Option<u32>> {
        let scheme = active_scheme();
        if scheme.is_empty() {
            return None;
        }
        let base = format!(
            r"SYSTEM\CurrentControlSet\Control\Power\User\PowerSchemes\{scheme}\{GUID_SUB_PROCESSOR}\{GUID_PROCTHROTTLEMAX}"
        );
        use crate::winutil::reg_get_dword;
        if let Some(ac) = reg_get_dword(&base, "ACSettingIndex") {
            return Some(Some(ac));
        }
        if let Some(dc) = reg_get_dword(&base, "DCSettingIndex") {
            return Some(Some(dc));
        }
        // Kullanıcı hiç değiştirmediyse değer yoktur; varsayılan %100 = turbo açık.
        Some(None)
    }

    /// Turbo açık mı? None = okunamadı. index == 100 => açık, diğer değer => kapalı.
    fn turbo_enabled() -> Option<bool> {
        // %100 = turbo açık. 99 (veya başka) = koruma aktif.
        match procthrottle_index() {
            None => None,
            Some(None) => Some(true), // ayar yok = turbo açık
            Some(Some(i)) => Some(i == 100),
        }
    }

    /// TARA -> yaz -> Güç Yönetimi'ne yönlendir. HİÇBİR POPUP/ONAY SORULMAZ.
    pub fn microcode_status(bus: &Con) {
        bus.header(tr("CPU / BIOS - MİKRO KOD KONTROLÜ", "CPU / BIOS - MICROCODE CHECK"));

        let id = identifier();
        let vendor = vendor();
        let sig = signature();

        bus.plain(format!(
            "  {} : {}",
            tr("İşlemci", "Processor"),
            if id.is_empty() {
                tr("okunamadı", "unavailable").to_string()
            } else {
                id.clone()
            }
        ));
        if !sig.is_empty() {
            bus.plain(format!("  {}   : {sig}", tr("Kimlik", "ID")));
        }

        let mc_raw = microcode_raw();
        let mc = mc_raw.and_then(microcode_low);
        // "Update Revision" = Windows'un şu an yüklü mikro kodu.
        // 0xFFFFFFFF / 0x0 = Windows değeri bildirmiyor (revizyon DEĞİLDİR).
        let mc_txt = match mc {
            Some(v) => format!("0x{v:X}"),
            None => tr("bildirilmiyor", "not reported").to_string(),
        };
        bus.plain(format!("  {}  : {mc_txt}", tr("Mikrokod (Windows)", "Microcode (Windows)")));

        // BIOS'un başlangıçta yüklediği revizyon — Windows bildirmese bile çoğu
        // sistemde bu değer doludur ve gerçek mikrokodu gösterir.
        let prev = microcode_prev().and_then(microcode_low);
        match prev {
            Some(v) => bus.plain(format!(
                "  {}     : 0x{v:X}",
                tr("Mikrokod (BIOS)", "Microcode (BIOS)")
            )),
            None => bus.dim(format!(
                "  {}     : {}",
                tr("Mikrokod (BIOS)", "Microcode (BIOS)"),
                tr("okunamadı", "unavailable")
            )),
        };

        let is_intel = vendor.to_lowercase().contains("genuineintel");
        let gen1314 = is_gen13_14(&id);
        if !(is_intel && gen1314) {
            bus.ok(tr(
                "  İşlemci Intel 13./14. nesil değil - mikrokod riski yok.",
                "  CPU is not Intel 13/14th gen - no microcode risk.",
            ));
            return;
        }

        bus.warn(tr("  TESPİT: Intel 13./14. nesil işlemci.", "  DETECTED: Intel 13/14th gen CPU."));

        let te = turbo_enabled();
        match te {
            Some(true) => bus.warn(tr(
                "  Turbo (Maksimum İşlemci): AÇIK (%100).",
                "  Turbo (Max Processor): ON (100%).",
            )),
            Some(false) => bus.ok(tr(
                "  Turbo (Maksimum İşlemci): kapalı (%99) - koruma aktif.",
                "  Turbo (Max Processor): off (99%) - protection active.",
            )),
            None => bus.warn(tr("  Turbo durumu okunamadı.", "  Turbo status unreadable.")),
        }

        // Karar: Windows değeri yoksa BIOS değerini kullan (ikisi de yoksa riskli varsay).
        let effective = mc.or(prev);
        let risky = !matches!(effective, Some(v) if v >= 0x12B);

        match effective {
            Some(v) if v >= 0x12B => {
                bus.ok(tr(
                    "  Mikrokod güncel (0x12B veya üzeri). Ek önlem gerekmiyor.",
                    "  Microcode is current (0x12B or newer). No extra action needed.",
                ));
            }
            Some(v) => {
                bus.warn(pick(
                    format!("  Mikrokod eski (0x{v:X}). Güncel: 0x12B veya üzeri."),
                    format!("  Microcode is old (0x{v:X}). Current: 0x12B or newer."),
                ));
            }
            None => {
                bus.warn(tr(
                    "  Mikrokod okunamadı - işlemci 13./14. nesil olduğu için riskli sayılır.",
                    "  Microcode unreadable - treated as risky since CPU is 13/14th gen.",
                ));
            }
        }

        if risky {
            bus.dim(tr(
                "  - Bu nesilde hatalı üretim voltajı nedeniyle eski mikrokodla",
                "  - On this generation, with old microcode",
            ));
            bus.dim(tr(
                "    yük altında (özellikle Hyper-V / sanal makine) mavi ekran olabilir.",
                "    heavy load (especially Hyper-V / VMs) can cause blue screens.",
            ));
            bus.dim(tr(
                "    Kalıcı çözüm: ana kart üreticisinden güncel BIOS (mikrokod 0x12B+).",
                "    Permanent fix: updated BIOS from the motherboard vendor (microcode 0x12B+).",
            ));
        }

        // Turbo KAPALI ise mikrokoddan bağımsız: POPUP YOK, sadece yönlendir.
        // %99 = CPU taban hızına düşer, oyunlar ağırlaşır.
        if matches!(te, Some(false)) {
            bus.ok(tr(
                "  Turbo kapalı (%99) - CPU taban hızında çalışıyor, oyunlar kasabilir.",
                "  Turbo off (99%) - CPU runs at base clock, games may stutter.",
            ));
            bus.dim(tr(
                "  - Geri açmak için: soldaki 'Güç Yönetimi' bölümü → 'Turbo Aç (%100)'.",
                "  - To re-enable: left panel 'Power' section → 'Turbo On (100%)'.",
            ));
            return;
        }

        // Mikrokod güncel ve turbo açık -> yapılacak bir şey yok.
        if !risky {
            return;
        }

        // Mikrokod eski/okunamadı + turbo açık -> POPUP SORULMAZ; kapatma için
        // Güç Yönetimi'ne yönlendirilir. Kullanıcı 'Turbo Kapat (%99)' ile
        // istediği zaman geçici korumayı kendisi açar.
        bus.warn(tr(
            "  Turbo açık + mikrokod eski. Geçici koruma için kapatman önerilir.",
            "  Turbo on + old microcode. Turning it off is recommended for protection.",
        ));
        bus.dim(tr(
            "  - Kapatmak için: soldaki 'Güç Yönetimi' bölümü → 'Turbo Kapat (%99)'.",
            "  - To turn it off: left panel 'Power' section → 'Turbo Off (99%)'.",
        ));
    }
}

// ---------------------------------------------------------------------------
// Bilinen sorunları tara ve uygula (reg.exe + powercfg - yerel Windows, VBS yok)
// Akış: TARA -> bulguları yaz -> TEK SORU -> UYGULA.
// ---------------------------------------------------------------------------

pub mod fixes {
    use std::process::{Command, Stdio};

    use crate::console::Con;

    fn run_silent(cmd: &str, args: &[&str]) -> bool {
        let mut c = Command::new(cmd);
        c.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        super::set_no_window_flags(&mut c);
        c.status().map(|s| s.success()).unwrap_or(false)
    }

    fn reg_query(key: &str, value: &str) -> String {
        super::capture("reg", &["query", key, "/v", value])
    }

    fn reg_set_dword(key: &str, value: &str, dword: &str) -> bool {
        run_silent(
            "reg",
            &[
                "add", key, "/v", value, "/t", "REG_DWORD", "/d", dword, "/f",
            ],
        )
    }

    const KEY_POWER: &str = r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power";
    const KEY_SM: &str = r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager";
    const KEY_MM: &str = r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
    const KEY_GDVR: &str = r"HKLM\SOFTWARE\Policies\Microsoft\Windows\GameDVR";

    // -- dedektörler (true = düzeltilecek sorun var) --
    fn hfvs_bad() -> bool {
        reg_query(KEY_POWER, "HiberbootEnabled")
            .to_lowercase()
            .contains("0x1")
    }
    fn dll_bad() -> bool {
        !reg_query(KEY_SM, "CWDIllegalInDllSearch")
            .to_lowercase()
            .contains("0xffffffff")
    }
    fn net_bad() -> bool {
        !reg_query(KEY_MM, "NetworkThrottlingIndex")
            .to_lowercase()
            .contains("0xffffffff")
    }
    fn gdv_bad() -> bool {
        !reg_query(KEY_GDVR, "AllowGameDVR")
            .to_lowercase()
            .contains("0x0")
    }

    // -- uygulayıcılar --
    fn hfvs_fix() -> bool {
        reg_set_dword(KEY_POWER, "HiberbootEnabled", "0")
    }
    fn dll_fix() -> bool {
        reg_set_dword(KEY_SM, "CWDIllegalInDllSearch", "0xffffffff")
    }
    fn net_fix() -> bool {
        reg_set_dword(KEY_MM, "NetworkThrottlingIndex", "0xffffffff")
    }
    fn gdv_fix() -> bool {
        reg_set_dword(KEY_GDVR, "AllowGameDVR", "0")
    }

    pub fn known_issues(bus: &Con) {
        use super::{pick, tr};
        bus.header(tr("BİLİNEN SORUN TARAMASI", "KNOWN ISSUES SCAN"));

        // (TR açıklama, EN açıklama, uygulayıcı)
        let mut plan: Vec<(&str, &str, fn() -> bool)> = Vec::new();

        if hfvs_bad() {
            bus.warn(tr("  - Hızlı Başlangıç: AÇIK", "  - Fast Startup: ON"));
            plan.push((
                "Hızlı Başlangıç kapatılacak",
                "Fast Startup will be disabled",
                hfvs_fix as fn() -> bool,
            ));
        } else {
            bus.ok(tr("  - Hızlı Başlangıç: kapalı (iyi)", "  - Fast Startup: off (good)"));
        }

        if dll_bad() {
            bus.warn(tr("  - DLL arama koruması: EKSİK", "  - DLL search protection: MISSING"));
            plan.push((
                "DLL arama koruması açılacak",
                "DLL search protection will be enabled",
                dll_fix as fn() -> bool,
            ));
        } else {
            bus.ok(tr("  - DLL arama koruması: mevcut (iyi)", "  - DLL search protection: present (good)"));
        }

        if net_bad() {
            bus.warn(tr(
                "  - Ağ kısıtlaması (NetworkThrottling): AÇIK",
                "  - Network throttling: ON",
            ));
            plan.push((
                "Ağ kısıtlaması kapatılacak (oyun/akış gecikmesi için)",
                "Network throttling will be disabled (for game/stream latency)",
                net_fix as fn() -> bool,
            ));
        } else {
            bus.ok(tr("  - Ağ kısıtlaması: kapalı (iyi)", "  - Network throttling: off (good)"));
        }

        if gdv_bad() {
            bus.warn(tr("  - GameDVR (arka plan kayıt): AÇIK", "  - GameDVR (background recording): ON"));
            plan.push((
                "GameDVR kapatılacak (kare düşmesi azalır)",
                "GameDVR will be disabled (reduces frame drops)",
                gdv_fix as fn() -> bool,
            ));
        } else {
            bus.ok(tr("  - GameDVR: kapalı (iyi)", "  - GameDVR: off (good)"));
        }

        if plan.is_empty() {
            bus.ok(tr(
                "Bilinen sorun bulunamadı - sistem bu kontrollerde temiz.",
                "No known issues found - system is clean on these checks.",
            ));
            return;
        }

        bus.dim(pick(
            format!("  {}: {} {}", tr("Uygulanacak", "To apply"), plan.len(), tr("düzeltme", "fixes")),
            format!("  To apply: {} fixes", plan.len()),
        ));

        let mut msg = String::from(tr(
            "Şu düzeltmeler uygulansın mı?\n\n",
            "Apply these fixes?\n\n",
        ));
        for (tr_s, en_s, _) in plan.iter() {
            msg.push_str(&format!("  - {}\n", tr(tr_s, en_s)));
        }
        msg.push_str(tr(
            "\nSadece güvenli ayarlar değişir; geri alınabilir.",
            "\nOnly safe settings change; reversible.",
        ));

        if !super::confirm(tr("Bilinen Sorunlar", "Known Issues"), &msg) {
            bus.warn(tr("İptal edildi.", "Cancelled."));
            return;
        }

        for (tr_s, en_s, apply) in plan.iter() {
            if apply() {
                bus.ok(pick(
                    format!("  {}: {}", tr("Uygulandı", "Applied"), tr(tr_s, en_s)),
                    format!("  Applied: {}", tr(tr_s, en_s)),
                ));
            } else {
                bus.err(pick(
                    format!("  {}: {}", tr("Başarısız", "Failed"), tr(tr_s, en_s)),
                    format!("  Failed: {}", tr(tr_s, en_s)),
                ));
            }
        }
        bus.ok(tr("Tamamlandı.", "Done."));
    }
}

// ---------------------------------------------------------------------------
// Güç yönetimi: güç planları + turbo aç/kapat (powercfg, konsol penceresi yok)
// ---------------------------------------------------------------------------

pub mod power {
    use std::process::{Command, Stdio};

    use crate::console::Con;
    use crate::i18n::{tr};

    /// Chris Titus tarafından yaygınlaştırılan Ultimate Performance planı.
    /// Windows'ta gizlidir; powercfg -duplicatescheme ile açılır.
    const ULTIMATE: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";
    const POWER_SAVER: &str = "a1841308-3541-4fab-bc81-f71556f20b4a";
    const BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
    const HIGH_PERF: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
    const GUID_SUB_PROCESSOR: &str = "54533251-82be-4824-96c1-47b60b740d00";
    /// Maksimum İşlemci Durumu (PROCTHROTTLEMAX, %0-100). %99 = turbo kapalı.
    const GUID_PROCTHROTTLEMAX: &str = "bc5038f7-23e0-4960-96da-33abaf5935ec";
    /// İkincil sınırlayıcı: PERFBOOSTMODE (0=Disabled .. 2=Aggressive).
    const GUID_PERFBOOSTMODE: &str = "be337238-0d82-4146-a960-4f3749d470c7";

    /// Sessiz komut (konsol penceresi açmaz) + stderr yakalar.
    fn run_out(cmd: &str, args: &[&str]) -> (bool, String) {
        let mut c = Command::new(cmd);
        c.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        super::set_no_window_flags(&mut c);
        match c.output() {
            Ok(o) => {
                let err = super::decode(&o.stderr).trim().to_string();
                (o.status.success(), err)
            }
            Err(e) => (false, e.to_string()),
        }
    }

    /// stdout + stderr yakalar (powercfg -list / -duplicatescheme çıktısındaki
    /// yeni GUID'i okuyabilmek için).
    fn run_capture(cmd: &str, args: &[&str]) -> (bool, String, String) {
        let mut c = Command::new(cmd);
        c.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        super::set_no_window_flags(&mut c);
        match c.output() {
            Ok(o) => (
                o.status.success(),
                super::decode(&o.stdout).trim().to_string(),
                super::decode(&o.stderr).trim().to_string(),
            ),
            Err(e) => (false, String::new(), e.to_string()),
        }
    }

    /// Metnin ilk GUID kelimesini döndürür (8-4-4-4-12 hex + tire).
    fn looks_like_guid(w: &str) -> bool {
        let b = w.as_bytes();
        b.len() == 36
            && b[8] == b'-'
            && b[13] == b'-'
            && b[18] == b'-'
            && b[23] == b'-'
            && b
                .iter()
                .enumerate()
                .all(|(i, &c)| [8, 13, 18, 23].contains(&i) || c.is_ascii_hexdigit())
    }

    fn extract_guid(text: &str) -> Option<String> {
        for word in text.split_whitespace() {
            let w = word.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '-'));
            if looks_like_guid(w) {
                return Some(w.to_string());
            }
        }
        None
    }

    /// powercfg için GUID: aktif şemayı registry'den döndür; boşsa (okunamadı)
    /// powercfg'in kendi çözdüğü SCHEME_CURRENT alias'ını kullan.
    fn scheme_or_current() -> String {
        let s = active_scheme();
        if s.is_empty() {
            "SCHEME_CURRENT".to_string()
        } else {
            s
        }
    }

    /// Aktif güç planı GUID'i (kayıt defterinden; dilden bağımsız).
    fn active_scheme() -> String {
        crate::winutil::reg_get_string(
            r"SYSTEM\CurrentControlSet\Control\Power\User\PowerSchemes",
            "ActivePowerScheme",
        )
        .unwrap_or_default()
    }

    /// Verilen planı aktifleştirir; yoksa `duplicatescheme` ile oluşturup dönen
    /// YENİ GUID'i aktifleştirir.
    ///
    /// ÖNEMLİ: Ultimate Performance'ın `e9a42b02-...` GUID'i bir ŞABLON'dur ve
    /// `-setactive` ona yazınca "Attempted to write to unsupported setting"
    /// hatası verir. Gerçek plan GUID'i `duplicatescheme` çıktısından gelir.
    /// (Chris Titus'un kendi winutil'inde de aynı bug vardı; çözüm aynıdır.)
    fn set_plan(bus: &Con, guid: &str, name_tr: &'static str, name_en: &'static str) -> bool {
        let cur = active_scheme().to_lowercase();
        if cur == guid.to_lowercase() {
            bus.ok(format!("  {} \"{}\".", tr("Zaten etkin:", "Already active:"), tr(name_tr, name_en)));
            return true;
        }

        let mut target = String::new();
        let mut last_err = String::new();

        // 1) powercfg -list'te bu GUID (veya şablonun kopyası) zaten var mı?
        let (_, list_out, _) = run_capture("powercfg", &["-list"]);
        let low = guid.to_lowercase();
        for line in list_out.lines() {
            if line.to_lowercase().contains(&low) {
                if let Some(g) = extract_guid(line) {
                    target = g;
                    break;
                }
            }
        }

        // 2) Listede yoksa önce şablon GUID ile setactive dene (ucuz test;
        //    bazı planlar için şablon GUID == gerçek GUID'dir ve direkt tutar).
        let mut tried_template = false;
        if target.is_empty() {
            let (ok, e) = run_out("powercfg", &["-setactive", guid]);
            if ok {
                target = guid.to_string();
            } else {
                tried_template = true;
                if !e.is_empty() {
                    last_err = e;
                }
            }
        }

        // 3) Yine yoksa duplicatescheme ile oluştur, çıktıdaki YENİ GUID'i yakala.
        if target.is_empty() {
            let (d, out, e2) = run_capture("powercfg", &["-duplicatescheme", guid]);
            if d {
                if let Some(g) = extract_guid(&out) {
                    target = g;
                } else if !tried_template {
                    // Bazı sürümler çıktıya GUID basmaz; şablonla tekrar dene.
                    target = guid.to_string();
                }
            }
            if target.is_empty() && !e2.is_empty() {
                last_err = e2;
            } else if target.is_empty() && !out.is_empty() {
                last_err = out;
            }
        }

        if target.is_empty() {
            bus.err(format!("  {} \"{}\".", tr("Etkinleştirilemedi:", "Failed to activate:"), tr(name_tr, name_en)));
            if !last_err.is_empty() {
                bus.dim(format!("    powercfg: {last_err}"));
            }
            return false;
        }

        // 4) Bulunan/en oluşturulan GUID'i aktifleştir.
        let (ok, e) = run_out("powercfg", &["-setactive", &target]);
        if ok {
            bus.ok(format!("  {} \"{}\".", tr("Etkinleştirildi:", "Activated:"), tr(name_tr, name_en)));
            true
        } else {
            if !e.is_empty() {
                last_err = e;
            }
            bus.err(format!("  {} \"{}\".", tr("Etkinleştirilemedi:", "Failed to activate:"), tr(name_tr, name_en)));
            if !last_err.is_empty() {
                bus.dim(format!("    powercfg: {last_err}"));
            }
            false
        }
    }

    pub fn power_saver(bus: &Con) {
        bus.header(tr("GÜÇ TASARRUFU", "POWER SAVER"));
        if set_plan(bus, POWER_SAVER, "Güç Tasarrufu", "Power Saver") {
            bus.dim(tr(
                "  - CPU ve ekran gücü kısılır; pil ömrü artar, performans düşer.",
                "  - CPU and screen power are reduced; longer battery life, lower performance.",
            ));
        }
    }

    pub fn balanced(bus: &Con) {
        bus.header(tr("DENGELİ", "BALANCED"));
        if set_plan(bus, BALANCED, "Dengeli", "Balanced") {
            bus.dim(tr(
                "  - Windows varsayılanı: ihtiyaca göre otomatik geçiş.",
                "  - Windows default: switches automatically as needed.",
            ));
        }
    }

    pub fn high_perf(bus: &Con) {
        bus.header(tr("YÜKSEK PERFORMANS", "HIGH PERFORMANCE"));
        if set_plan(bus, HIGH_PERF, "Yüksek Performans", "High Performance") {
            bus.dim(tr(
                "  - CPU yüksek frekansta tutulur; fan gürültüsü artabilir.",
                "  - CPU is kept at high clocks; fan noise may increase.",
            ));
        }
    }

    pub fn ultimate(bus: &Con) {
        bus.header(tr("ULTIMATE PERFORMANCE", "ULTIMATE PERFORMANCE"));
        bus.dim(tr(
            "  - Chris Titus tarzı: gizli en üst performans planı açılır.",
            "  - Chris Titus style: the hidden top-performance plan is unlocked.",
        ));
        if set_plan(bus, ULTIMATE, "Ultimate Performance", "Ultimate Performance") {
            bus.dim(tr(
                "  - Plan yoksa otomatik eklenir (powercfg -duplicatescheme).",
                "  - The plan is auto-added if missing (powercfg -duplicatescheme).",
            ));
        }
    }

    /// Turbo'yu kapat: PROCTHROTTLEMAX 100 -> 99 ve PERFBOOSTMODE -> 0 (Disabled).
    /// powercfg çalışmazsa doğrudan kayıt defterine yazar (yeniden başlatınca
    /// ya da plan yenilenince kesinleşir).
    pub fn turbo_off(bus: &Con) {
        bus.header(tr("TURBO KAPAT (%99)", "TURBO OFF (99%)"));
        let scheme = scheme_or_current();
        let mut any = false;
        let mut last_err = String::new();
        for flag in ["-setacvalueindex", "-setdcvalueindex"] {
            for (guid, val) in [(GUID_PROCTHROTTLEMAX, "99"), (GUID_PERFBOOSTMODE, "0")] {
                let (ok, e) = run_out("powercfg", &[flag, &scheme, GUID_SUB_PROCESSOR, guid, val]);
                any |= ok;
                if !ok && !e.is_empty() {
                    last_err = e;
                }
            }
        }
        let (ok5, e5) = run_out("powercfg", &["-setactive", &scheme]);
        any |= ok5;
        if !ok5 && !e5.is_empty() {
            last_err = e5;
        }
        if any {
            bus.dim(tr(
                "  - (Geçici koruma bu kadarıyla aktif; aşağıda hâlâ hata görünüyorsa önemsizdir.)",
                "  - (Temporary protection is set; any error shown below is harmless.)",
            ));
        }
        if !any {
            // Fallback: kayıt defterine doğrudan yaz (GUID'i temizle).
            let s = active_scheme();
            if !s.is_empty() {
                let base = format!(
                    r"SYSTEM\CurrentControlSet\Control\Power\User\PowerSchemes\{s}\{GUID_SUB_PROCESSOR}\{GUID_PROCTHROTTLEMAX}"
                );
                any |= crate::winutil::reg_set_dword(&base, "ACSettingIndex", 99);
                any |= crate::winutil::reg_set_dword(&base, "DCSettingIndex", 99);
                if any {
                    bus.dim(tr(
                        "  - powercfg yanıt vermedi; değer doğrudan kayıt defterine yazıldı.",
                        "  - powercfg didn't respond; the value was written directly to the registry.",
                    ));
                    bus.dim(tr(
                        "    Yeniden başlatınca (ya da plan yenilenince) kesinleşir.",
                        "    It takes effect after a reboot (or when the plan refreshes).",
                    ));
                }
            }
        }
        if any {
            bus.ok(tr(
                "  Uygulandı: Turbo kapalı (%99). CPU taban hızında, daha serin.",
                "  Applied: Turbo off (99%). CPU at base clock, cooler.",
            ));
            if !last_err.is_empty() {
                bus.dim(format!("    powercfg: {last_err}"));
            }
            bus.dim(tr(
                "  - Oyun oynarken kasma olursa 'Turbo Aç' ile geri alabilirsin.",
                "  - If games stutter, undo with 'Turbo On'.",
            ));
        } else {
            bus.err(tr(
                "  Uygulanamadı. Görünüşe göre komutlar sessizce başarısız oluyor.",
                "  Could not apply. It seems the commands fail silently.",
            ));
            bus.dim(format!("    powercfg: {}", if last_err.is_empty() { tr("(çıktı/ipucu yok)", "(no output/hint)") } else { &last_err }));
            bus.ok(tr(
                "  EL İLE YAP: Yönetici olarak şunu çalıştır:",
                "  MANUAL: Run this as administrator:",
            ));
            bus.big("    powercfg /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR PROCTHROTTLEMAX 99");
            bus.big("    powercfg /setdcvalueindex SCHEME_CURRENT SUB_PROCESSOR PROCTHROTTLEMAX 99");
            bus.big("    powercfg /setactive SCHEME_CURRENT");
        }
    }

    /// Turbo'yu aç: PROCTHROTTLEMAX 99 -> 100 ve PERFBOOSTMODE -> 2 (Aggressive).
    pub fn turbo_on(bus: &Con) {
        bus.header(tr("TURBO AÇ (%100)", "TURBO ON (100%)"));
        let scheme = scheme_or_current();
        let mut any = false;
        let mut last_err = String::new();
        for flag in ["-setacvalueindex", "-setdcvalueindex"] {
            for (guid, val) in [(GUID_PROCTHROTTLEMAX, "100"), (GUID_PERFBOOSTMODE, "2")] {
                let (ok, e) = run_out("powercfg", &[flag, &scheme, GUID_SUB_PROCESSOR, guid, val]);
                any |= ok;
                if !ok && !e.is_empty() {
                    last_err = e;
                }
            }
        }
        let (ok5, e5) = run_out("powercfg", &["-setactive", &scheme]);
        any |= ok5;
        if !ok5 && !e5.is_empty() {
            last_err = e5;
        }
        if any {
            bus.dim(tr(
                "  - (Turbo açıldı; aşağıda hâlâ hata görünüyorsa önemsizdir.)",
                "  - (Turbo is on; any error shown below is harmless.)",
            ));
        }
        if !any {
            let s = active_scheme();
            if !s.is_empty() {
                let base = format!(
                    r"SYSTEM\CurrentControlSet\Control\Power\User\PowerSchemes\{s}\{GUID_SUB_PROCESSOR}\{GUID_PROCTHROTTLEMAX}"
                );
                any |= crate::winutil::reg_set_dword(&base, "ACSettingIndex", 100);
                any |= crate::winutil::reg_set_dword(&base, "DCSettingIndex", 100);
                if any {
                    bus.dim(tr(
                        "  - powercfg yanıt vermedi; değer doğrudan kayıt defterine yazıldı.",
                        "  - powercfg didn't respond; the value was written directly to the registry.",
                    ));
                }
            }
        }
        if any {
            bus.ok(tr(
                "  Uygulandı: Turbo açık (%100). Tam hızda çalışabilirsin.",
                "  Applied: Turbo on (100%). You can run at full speed.",
            ));
            if !last_err.is_empty() {
                bus.dim(format!("    powercfg: {last_err}"));
            }
        } else {
            bus.err(tr(
                "  Uygulanamadı. Görünüşe göre komutlar sessizce başarısız oluyor.",
                "  Could not apply. It seems the commands fail silently.",
            ));
            bus.dim(format!("    powercfg: {}", if last_err.is_empty() { tr("(çıktı/ipucu yok)", "(no output/hint)") } else { &last_err }));
            bus.ok(tr(
                "  EL İLE YAP: Yönetici olarak şunu çalıştır:",
                "  MANUAL: Run this as administrator:",
            ));
            bus.big("    powercfg /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR PROCTHROTTLEMAX 100");
            bus.big("    powercfg /setdcvalueindex SCHEME_CURRENT SUB_PROCESSOR PROCTHROTTLEMAX 100");
            bus.big("    powercfg /setactive SCHEME_CURRENT");
        }
    }
}
