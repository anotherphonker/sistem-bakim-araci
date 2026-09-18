#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // GUI: konsol yok

mod actions;
mod app;
mod config;
mod console;
mod emoji;
mod i18n;
mod sections;
mod theme;
mod winutil;

const APP_TITLE: &str = "Sistem Bakım Aracı v0.1";
const WINDOW_TITLE: &str = "Sistem Bakım Aracı";

fn main() -> eframe::Result<()> {
    // Dil tercihini config'ten erkenden yükle (msgbox'lar da doğru dilde olsun).
    let _cfg = config::Config::load();
    i18n::set_en(_cfg.lang == "en");

    // --- Per-Monitor DPI (manifest gömülmediği için programatik) ---
    #[cfg(windows)]
    winutil::enable_per_monitor_dpi();

    // --- Sıra ÖNEMLİ: önce yönetici yükseltme, SONRA tek-örnek (mutex). ---
    // Eski sıralamada tersiydi: yükseltme (runas) ile yeni süreç başlarken
    // orijinal süreç daha kapanmadığı için mutex hâlâ açık görünüyor ve yeni
    // süreç "zaten açık" deyip kendini kapatıyordu -> hiç pencere açılmıyordu.
    if winutil::elevate_if_needed() {
        return Ok(());
    }

    // --- Tek örnek koruması (mutex, süreç ömrü boyunca tutulur) ---
    let _guard = match winutil::SingleInstance::acquire() {
        Some(g) => g,
        None => {
            winutil::bring_to_front(WINDOW_TITLE);
            winutil::msgbox_warn(
                APP_TITLE,
                if i18n::is_en() {
                    "The app is already open.\nTaking you to the open window."
                } else {
                    "Uygulama zaten açık.\nAçık olan pencereye yönlendiriliyorsunuz."
                },
            );
            return Ok(());
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 720.0])
            .with_min_inner_size([960.0, 620.0])
            .with_title(WINDOW_TITLE)
            .with_app_id("sistem-bakim-araci"), // görev çubuğu/ikon kimliği
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(app::SistemBakimApp::new(cc)))),
    )
}
