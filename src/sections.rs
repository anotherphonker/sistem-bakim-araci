//! Bölümler ve araç tanımları (çift dilli: Türkçe + English).
//! Emojiler `emoji::Emoji` ile çizilir (exe'ye gömülü PNG -> her Windows'ta aynı).

use crate::emoji::Emoji;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SectionId {
    Home,
    Temizlik,
    Onarim,
    Disk,
    Guc,
    Bilgi,
}

/// Tüm araç eylemleri.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    FullRepair,
    QuickClean,
    DeepClean,
    Chkdsk,
    Sfc,
    CheckHealth,
    ScanHealth,
    RestoreHealth,
    RestoreIso,
    ComponentCleanup,
    AnalyzeStore,
    DiskInfo,
    CpuInfo,
    KnownIssues,
    Explorer,
    Recycle,
    WuCache,
    BigFiles,
    SysInfo,
    Logs,
    About,
    PowerSaver,
    PowerBalanced,
    PowerHighPerf,
    PowerUltimate,
    TurboOn,
    TurboOff,
}

#[derive(Clone)]
pub struct Item {
    pub emoji: Emoji,
    pub title: &'static str,
    pub title_en: &'static str,
    pub desc: &'static str,
    pub desc_en: &'static str,
    pub action: Action,
    pub accent: bool,
}

#[derive(Clone)]
pub struct Section {
    pub id: SectionId,
    pub nav_emoji: Emoji,
    pub title: &'static str,
    pub title_en: &'static str,
    pub sub: &'static str,
    pub sub_en: &'static str,
    pub items: Vec<Item>,
}

pub fn all() -> Vec<Section> {
    use Action::*;
    let m = |emoji: Emoji,
             title: &'static str,
             title_en: &'static str,
             desc: &'static str,
             desc_en: &'static str,
             action: Action,
             accent: bool| Item {
        emoji,
        title,
        title_en,
        desc,
        desc_en,
        action,
        accent,
    };

    vec![
        Section {
            id: SectionId::Home,
            nav_emoji: Emoji::Home,
            title: "Ana Menü",
            title_en: "Home",
            sub: "Kısayollar. Tüm araçlar soldaki bölümlerde.",
            sub_en: "Shortcuts. All tools live in the sections on the left.",
            items: vec![
                m(Emoji::Shield, "Tam Otomatik Onarım", "Full Auto Repair", "DISM + SFC zinciri, tek tık. En kapsamlı onarım.", "DISM + SFC chain, one click. The most thorough repair.", FullRepair, true),
                m(Emoji::Bolt, "Mikro Kod Kontrolü", "Microcode Check", "Intel 13./14. nesil: mikrokodu kontrol eder; kapat/aç için Güç Yönetimi'ni gösterir.", "Intel 13/14th gen: checks microcode; points to Power for the toggle.", CpuInfo, false),
                m(Emoji::Broom, "Hızlı Temizlik", "Quick Clean", "Temp, önbellek ve geri dönüşüm.", "Temp, cache and recycle bin.", QuickClean, false),
                m(Emoji::Sponge, "Derin Temizlik", "Deep Clean", "Hızlı + WU önbelleği, prefetch, tarayıcı cache.", "Quick + WU cache, prefetch, browser cache.", DeepClean, false),
                m(Emoji::Magnifier, "Disk Taraması", "Disk Scan", "chkdsk /scan ile çevrimiçi disk taraması.", "Online disk scan with chkdsk /scan.", Chkdsk, false),
            ],
        },
        Section {
            id: SectionId::Temizlik,
            nav_emoji: Emoji::Broom,
            title: "Temizlik",
            title_en: "Cleanup",
            sub: "Geçici/gereksiz dosyalar. Kişisel dosyalarına dokunmaz.",
            sub_en: "Temporary/junk files. Never touches personal files.",
            items: vec![
                m(Emoji::Broom, "Hızlı Temizlik", "Quick Clean", "Kullanıcı/Windows Temp + önbellek + geri dönüşüm.", "User/Windows Temp + cache + recycle bin.", QuickClean, false),
                m(Emoji::Sponge, "Derin Temizlik", "Deep Clean", "Hızlı + WU önbelleği, prefetch, simge cache, minidump, tarayıcı cache.", "Quick + WU cache, prefetch, icon cache, minidump, browser cache.", DeepClean, false),
                m(Emoji::Refresh, "Explorer'ı Yeniden Başlat", "Restart Explorer", "Donuk / ikonsuz masaüstünü tazeler.", "Refreshes a frozen / icon-less desktop.", Explorer, false),
                m(Emoji::Recycle, "Geri Dönüşümü Boşalt", "Empty Recycle Bin", "Silinenleri kalıcı temizler. Geri alınamaz.", "Permanently removes deleted items. Cannot be undone.", Recycle, false),
                m(Emoji::Inbox, "WU Önbelleğini Temizle", "Clear WU Cache", "Takılan/hata veren Windows Update'i açar.", "Unsticks a stuck / failing Windows Update.", WuCache, false),
                m(Emoji::Chart, "Alan Canavarı", "Space Hog Finder", "C:'de en büyük 20 dosyayı listeler. Silmez.", "Lists the 20 largest files on C:. Does not delete.", BigFiles, false),
            ],
        },
        Section {
            id: SectionId::Onarim,
            nav_emoji: Emoji::Shield,
            title: "Onarım",
            title_en: "Repair",
            sub: "SFC + DISM. 0x8000ffff gibi hataların ilacı.",
            sub_en: "SFC + DISM. The cure for errors like 0x8000ffff.",
            items: vec![
                m(Emoji::Shield, "TAM OTOMATİK ONARIM", "FULL AUTO REPAIR", "CheckHealth + ScanHealth + RestoreHealth + SFC + Cleanup. 20-90 dk.", "CheckHealth + ScanHealth + RestoreHealth + SFC + Cleanup. 20-90 min.", FullRepair, true),
                m(Emoji::Bolt, "Mikro Kod Kontrolü", "Microcode Check", "Intel 13./14. nesil: mikrokodu kontrol eder; kapat/aç için Güç Yönetimi'ni gösterir.", "Intel 13/14th gen: checks microcode; points to Power for the toggle.", CpuInfo, false),
                m(Emoji::Wrench, "SFC /scannow", "SFC /scannow", "Korunan Windows dosyalarını onarır (5-15 dk).", "Repairs protected Windows files (5-15 min).", Sfc, false),
                m(Emoji::Stethoscope, "CheckHealth", "CheckHealth", "Hızlı imaj durumu. Sadece rapor.", "Quick image health. Report only.", CheckHealth, false),
                m(Emoji::Magnifier, "ScanHealth", "ScanHealth", "Ayrıntılı tarama (5-15 dk). Sadece teşhis.", "Detailed scan (5-15 min). Diagnostic only.", ScanHealth, false),
                m(Emoji::Pill, "RestoreHealth", "RestoreHealth", "Bozukluğu internetten onarır (15-60 dk). %62-63'te durması normal.", "Repairs corruption via internet (15-60 min). Stalling at 62-63% is normal.", RestoreHealth, false),
                m(Emoji::Disk, "RestoreHealth + ISO", "RestoreHealth + ISO", "İnternet yerine ISO'yu kaynak yapar (otomatik mount).", "Uses an ISO as source instead of internet (auto mount).", RestoreIso, false),
                m(Emoji::Trash, "ComponentCleanup", "ComponentCleanup", "Eski güncelleme artıklarını temizler.", "Cleans up leftover update components.", ComponentCleanup, false),
                m(Emoji::Chart, "AnalyzeComponentStore", "AnalyzeComponentStore", "Bileşen deposunun yer kullanımını raporlar.", "Reports component store space usage.", AnalyzeStore, false),
                m(Emoji::Shield, "Bilinen Sorunları Uygula", "Apply Known Fixes", "Hızlı Başlangıç + DLL koruması tarar ve düzeltir.", "Checks and fixes Fast Startup + DLL protection.", KnownIssues, false),
            ],
        },
        Section {
            id: SectionId::Disk,
            nav_emoji: Emoji::Disk,
            title: "Disk",
            title_en: "Disk",
            sub: "Disk tarama ve bilgi. Salt okunur, güvenli.",
            sub_en: "Disk scan and info. Read-only, safe.",
            items: vec![
                m(Emoji::Magnifier, "chkdsk C: /scan", "chkdsk C: /scan", "Çevrimiçi, salt okunur disk taraması.", "Online, read-only disk scan.", Chkdsk, false),
                m(Emoji::Chart, "Disk / Birim Bilgisi", "Disk / Volume Info", "C: birimi ve fiziksel disklerin teknik verileri.", "Technical data for the C: volume and physical disks.", DiskInfo, false),
                m(Emoji::Optical, "Component Yer Analizi", "Component Space Analysis", "Güncelleme havuzunun yer kullanımı.", "Space usage of the update pool.", AnalyzeStore, false),
            ],
        },
        Section {
            id: SectionId::Guc,
            nav_emoji: Emoji::Controls,
            title: "Güç Yönetimi",
            title_en: "Power",
            sub: "Güç planları ve Turbo kontrolü. powercfg ile anında uygulanır.",
            sub_en: "Power plans and Turbo control. Applied instantly with powercfg.",
            items: vec![
                m(Emoji::Battery, "Güç Tasarrufu", "Power Saver", "Minimum enerji, düşük performans (pil ömrü).", "Minimum energy, low performance (battery life).", PowerSaver, false),
                m(Emoji::Scale, "Dengeli", "Balanced", "Windows varsayılan dengesi. Genel kullanım.", "Windows default balance. General use.", PowerBalanced, false),
                m(Emoji::Rocket, "Yüksek Performans", "High Performance", "CPU'yu yüksek frekansta tutar, fan gürültüsü artar.", "Keeps the CPU at high clocks, more fan noise.", PowerHighPerf, false),
                m(Emoji::Bolt, "Ultimate Performance", "Ultimate Performance", "Chris Titus tarzı: gizli en yüksek performans planını açar.", "Chris Titus style: unlocks the hidden max-performance plan.", PowerUltimate, false),
                m(Emoji::Computer, "Turbo Kapat (%99)", "Turbo Off (99%)", "Maksimum İşlemci 100 -> 99. CPU taban hızında, daha serin.", "Max Processor 100 -> 99. CPU at base clock, cooler.", TurboOff, false),
                m(Emoji::Rocket, "Turbo Aç (%100)", "Turbo On (100%)", "Maksimum İşlemci 99 -> 100. Tam hız.", "Max Processor 99 -> 100. Full speed.", TurboOn, false),
            ],
        },
        Section {
            id: SectionId::Bilgi,
            nav_emoji: Emoji::Info,
            title: "Bilgi & Log",
            title_en: "Info & Logs",
            sub: "Sistem özeti ve destek logları.",
            sub_en: "System summary and support logs.",
            items: vec![
                m(Emoji::Chart, "Sistem Bilgisi", "System Info", "İşletim sistemi, boş alan, yönetici durumu.", "OS, free space, admin status.", SysInfo, false),
                m(Emoji::Clipboard, "Logları Masaüstüne Kopyala", "Copy Logs to Desktop", "CBS/DISM loglarını masaüstüne al.", "Copies CBS/DISM logs to the desktop.", Logs, false),
                m(Emoji::Info, "Hakkında", "About", "Sürüm ve derleme bilgisi.", "Version and build info.", About, false),
            ],
        },
    ]
}
